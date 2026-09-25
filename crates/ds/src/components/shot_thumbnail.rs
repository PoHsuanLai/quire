//! ShotThumbnail: the floating thumbnail after a screenshot (design/04-COMPONENTS.md section 39,
//! design/20 section 1.13; sill Q181).
//!
//! The picture sits letterboxed in a card of the Toast material ([`shot_frame`]), inside a
//! transparent Toast scope, so the card paints the material wherever its root is (as
//! `NotificationCard` does). Showing and hiding are the caller's (`shown`), since it owns the
//! hold (`screenshot.thumbnail_hold_ms`) and knows when a dismissal happened; the card rises in
//! (`Anim::ShotIn`), slides out to the right (`Anim::ShotOut`) and calls `on_hidden` once that
//! exit has settled, so the host can unmap the surface. The pointer's arrival and departure go
//! to `onhover`, so the caller's hold can pause while the card is looked at, and show the
//! actions. A press on the picture opens it on its click, or, once it has travelled
//! [`crate::DRAG_THRESHOLD`], asks the host to start a drag ([`DragStart`]) and does not open.
//!
//! `swipe: Swipe::Dismiss(..)` lets a drag or a horizontal scroll to the right dismiss the card
//! exactly as it dismisses a `NotificationCard` (`notification_swipe`, with the same
//! `notifications.swipe_*` metrics): alone it flies out itself and reports at the flight's
//! settle; in a `BannerStack` row the row carries the flight. A press heading mostly right is
//! then the swipe's and never a drag out. The card places nothing itself (no margin, no
//! anchor), so a `BannerStack` or the caller's surface decides where it sits.

use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::image_source::{ImageSize, ImageSource};
use crate::components::notification_parts::Hover;
use crate::components::notification_swipe::{CardSwipe, Swipe, use_card_swipe};
use crate::components::osd_phase::OsdPhase;
use crate::components::press::Propagation;
use crate::components::shot_frame::{picture_style, shot_frame};
use crate::components::shot_press::{DragLane, DragStart, PressInput, ShotPress};
use crate::components::shown_phase::{Alias, use_shown_phase};
use crate::components::text_runs::Text;
use crate::components::tooltip::Shown;
use crate::components::vocab::StaggerIndex;
use crate::geometry::{Point, Px};
use crate::icon::Icon;
use crate::material::Material;
use crate::motion::anim::Anim;
use crate::motion::swipe::SwipeMetrics;
use crate::root::chrome::RootChrome;
use crate::root::surface::Surface;
use dioxus::prelude::*;

/// One action the thumbnail offers on hover: Delete now; Mark Up and Copy Text once the shell
/// has them. Its press stays at its button: it never also opens the picture.
#[derive(Debug, Clone, PartialEq)]
pub struct ThumbAction {
    /// Its glyph.
    pub icon: Icon,
    /// Its name, read as the button's label.
    pub label: Text,
    /// What a press does.
    pub onpress: EventHandler<()>,
}

/// The screenshot thumbnail. `image` is drawn at `width` (240 by default, sill's surface) in a
/// box of `size`'s ratio held between 2:1 and 16:10. `id` names the card for the host's input
/// and blur region (`Element("thumb")`). `swipe` turns swipe to dismiss on, with
/// `swipe_metrics` from the `notifications.swipe_*` keys.
#[component]
pub fn ShotThumbnail(
    image: ImageSource,
    size: ImageSize,
    shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default = Px(240.0))] width: Px,
    #[props(default)] actions: Vec<ThumbAction>,
    #[props(default)] onopen: Option<EventHandler<()>>,
    #[props(default)] ondrag: Option<EventHandler<DragStart>>,
    #[props(default)] onhover: Option<EventHandler<Hover>>,
    #[props(default)] id: Option<String>,
    #[props(default)] swipe: Swipe,
    #[props(default)] swipe_metrics: SwipeMetrics,
) -> Element {
    let (phase, alias) = use_shown_phase(shown, on_hidden, Anim::ShotIn, Anim::ShotOut);
    rsx! {
        Surface { material: Material::Toast, chrome: RootChrome::Transparent,
            // Keyed by the entrance's alias, which flips on each fresh showing: a card shown
            // again after a swipe flew it out starts with a new swipe (the machine's `Gone` is
            // final, as a notification is dropped after it), a new press and no hover.
            ShotCard {
                key: "{alias.slug()}",
                phase,
                alias,
                image,
                size,
                width,
                actions,
                onopen,
                ondrag,
                onhover,
                id,
                swipe,
                swipe_metrics,
            }
        }
    }
}

/// One showing's card: its swipe, its press and its hover.
#[component]
fn ShotCard(
    phase: OsdPhase,
    alias: Alias,
    image: ImageSource,
    size: ImageSize,
    width: Px,
    actions: Vec<ThumbAction>,
    onopen: Option<EventHandler<()>>,
    ondrag: Option<EventHandler<DragStart>>,
    onhover: Option<EventHandler<Hover>>,
    id: Option<String>,
    swipe: Swipe,
    swipe_metrics: SwipeMetrics,
) -> Element {
    let lane = match swipe {
        Swipe::Dismiss(_) => DragLane::NotRight,
        Swipe::Off => DragLane::Any,
    };
    let swiper = use_card_swipe(&swipe, swipe_metrics);
    let press = use_signal(ShotPress::default);
    let mut hover = use_signal(Hover::default);
    let mut point = move |next: Hover| {
        if *hover.peek() != next {
            hover.set(next);
            if let Some(onhover) = onhover {
                onhover.call(next);
            }
        }
    };
    let frame = shot_frame(width, size);
    rsx! {
        div {
            class: "ds-shot",
            "data-shown": phase.shown().slug(),
            "data-presence": phase.presence(),
            "data-pulse": alias.slug(),
            "data-hover": hover().slug(),
            "data-swipe": swiper.look(),
            style: card_style(frame.card.width, swiper.style()),
            onpointerenter: move |_| point(Hover::Over),
            onpointerleave: move |_| {
                point(Hover::Away);
                swiper.up();
            },
            onpointerdown: move |event| swiper.down(&event),
            onpointermove: move |event| swiper.moved(&event),
            onpointerup: move |_| swiper.up(),
            onwheel: move |event| swiper.wheel(&event),
            div {
                class: "ds-shot-plate",
                id,
                role: "group",
                "aria-label": "Screenshot",
                style: "height:{frame.card.height.0}px",
                {picture(image, picture_style(frame.picture), Press { press, swiper, lane, onopen, ondrag })}
                if !actions.is_empty() {
                    {action_row(actions)}
                }
            }
        }
    }
}

/// The card's inline width, and its swipe offset while it is off its place.
fn card_style(width: Px, swipe: Option<String>) -> String {
    match swipe {
        Some(offset) => format!("width:{}px;{offset}", width.0),
        None => format!("width:{}px", width.0),
    }
}

/// What the picture's press needs: its machine, the card's swipe (whose drag must not also open),
/// the lane a drag out may take, and who hears the open and the drag.
#[derive(Clone, Copy)]
struct Press {
    press: Signal<ShotPress>,
    swiper: CardSwipe,
    lane: DragLane,
    onopen: Option<EventHandler<()>>,
    ondrag: Option<EventHandler<DragStart>>,
}

impl Press {
    fn feed(self, input: PressInput) {
        let mut press = self.press;
        let (next, start) = press.peek().clone().step(input);
        press.set(next);
        if let (Some(start), Some(ondrag)) = (start, self.ondrag) {
            ondrag.call(start);
        }
    }

    fn open(self) {
        if let Some(onopen) = self.onopen {
            onopen.call(());
        }
    }

    /// A click opens only when neither a drag out nor a swipe ended with it.
    fn click(self) {
        let swiped = !self.swiper.click_passes();
        if self.press.peek().opens() && !swiped {
            self.open();
        }
    }
}

/// The picture and its press target.
fn picture(image: ImageSource, style: String, press: Press) -> Element {
    rsx! {
        div {
            class: "ds-shot-picture",
            role: "button",
            tabindex: "0",
            "aria-label": "Open screenshot",
            onpointerdown: move |event| press.feed(PressInput::Down(client(&event))),
            onpointermove: move |event| press.feed(PressInput::Moved(client(&event), press.lane)),
            onpointerup: move |_| press.feed(PressInput::Up),
            onpointercancel: move |_| press.feed(PressInput::Up),
            onclick: move |_| press.click(),
            onkeydown: move |event| {
                if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                    event.prevent_default();
                    press.open();
                }
            },
            img { class: "ds-shot-image", alt: "", src: image.0, style, draggable: "false" }
        }
    }
}

/// A pointer event's client point.
fn client(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// The hover row: strip buttons that pop in one after another (`--j`), each keeping its press.
fn action_row(actions: Vec<ThumbAction>) -> Element {
    rsx! {
        div { class: "ds-shot-actions", role: "toolbar", "aria-label": "Screenshot actions",
            for (j , action) in actions.into_iter().enumerate() {
                span { key: "{j}", class: "ds-shot-action", style: "--j:{StaggerIndex::new(j).get()}",
                    IconButton {
                        variant: IconButtonVariant::Strip,
                        icon: action.icon,
                        label: action.label.plain_text(),
                        propagation: Propagation::Stop,
                        onclick: move |_| action.onpress.call(()),
                    }
                }
            }
        }
    }
}
