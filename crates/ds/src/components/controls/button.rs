//! Button: a labelled action in six variants (design/04-COMPONENTS.md section 1).
//! Markup: `button.ds-button[data-variant]`, `aria-pressed` only for a toggle Mini; `title`,
//! `aria-label` and `aria-expanded` only when the caller gives them (or a mark face names it);
//! the consumer's own `data-*` and classes after quire's.

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::TextLine;
use crate::components::controls::button_face::{
    ButtonFace, FaceMark, Leading, Trailing, leading as leading_mark, spoken_label,
    trailing as trailing_mark,
};
use crate::components::controls::button_size::{ButtonSize, disabled};
use crate::components::controls::press::{PressListeners, Propagation, use_pressing};
use crate::core::press::Press;
use crate::core::vocab::{Availability, Check, Shown};
use crate::core::word::Word;
use crate::root::common::Common;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// Which button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ButtonVariant {
    /// Accent fill: Send, Compose.
    Primary,
    /// Surface-2 with a line: the ghost button.
    Secondary,
    /// Small, surface-2: Reply, RSVP, hover-card actions.
    Mini,
    /// Text only: the quiet link-button.
    Quiet,
    /// As Mini at rest; red only on hover.
    Danger,
    /// Words on the Space frame: the sidebar item's chrome, `--f-ink-soft` on
    /// nothing at rest, `--f-ink` on `--f-pill-hover` under the pointer, on `--f-pill` while
    /// pressed or held down. A frame word's contrast is the frame's, which no surface token
    /// guarantees, so the other variants' `--ink` family does not belong there.
    Frame,
}

impl ButtonVariant {
    /// The icon size: 14 for every variant that the doc sizes. TODO(O-3): the Quiet icon is
    /// "not specified" in design/04-COMPONENTS.md section 1; it takes the same 14.
    fn icon_size(self) -> IconSize {
        IconSize::Compact
    }
}

/// A labelled action. `icon` is a glyph or an external icon (an `Icon` or `Option<Icon>`
/// converts). `id` is written as the element's `id`, so a popup can anchor to it by id.
/// `onclick` hears the primary, secondary (right-click) and middle buttons, and the keyboard as
/// primary. `mounted` hands over the element once it is in the document, so a floating
/// component can anchor to it (`Anchor::Mounted`).
///
/// `title` is the hover hint. `aria_label` names the button to assistive technology in place of
/// its visible label, for a label that is a symbol or too terse to stand alone ("+" or "All").
/// `expanded` says whether the menu or panel this button opens is showing (`aria-expanded`);
/// leave it `None` on a button that opens nothing.
///
/// `trailing` puts a mark after the label: `Trailing::Caret` for a dropdown showing its value,
/// or a glyph. `leading` puts one before it: `Leading::Mark` holds an element
/// such as a `ProviderMark`, for a From dropdown whose value shows the account's provider;
/// `Leading::Glyph` a glyph (for a lone glyph, `icon` is the same thing). `face` draws the label as a styled letter (`ButtonFace::Bold` is a bold `B`)
/// and then names the button by `label` through `aria-label`, unless `common.aria_label` says
/// otherwise.
///
/// `label` is a [`Text`]: a `String` or `&str` as before, or runs in their tones (a quoted
/// message's head, "who" strong and "when" faint), drawn inside the label's span. A
/// label of runs names the button by its characters (`Text::plain_text`) through `aria-label`,
/// unless `aria_label` says otherwise.
///
/// `propagation: Propagation::Stop` keeps the press at the button: its ancestors never hear
/// the click (a header action inside a `<summary>` leaves the `<details>` as it was).
///
/// `common` puts the consumer's own `id`, `data-*` attributes and classes on the button itself,
/// so it needs no wrapping `span`: `data-folder` for a drag that reads the place off the element
/// under the pointer, a class for the consumer's own reveal or layout rule (see [`Common`]); its
/// `aria_label` names the button where `label` does not, and `mounted` hands over the element for
/// a menu or popover anchored to it.
///
/// `size` draws the variant at another size: `Some(ButtonSize::Regular)` gives a
/// Danger the Primary's geometry, so Restart sits level with Shut Down and Cancel beside it.
/// `None` keeps the variant's own size (Danger and Mini are Mini-sized) and writes nothing.
///
/// `availability: Availability::Disabled` writes `aria-disabled` and `disabled`,
/// draws the button at .35 with no hover and no press (design/13 section 13.3.3's disabled
/// item), and drops every press: `onclick` never runs.
#[component]
pub fn Button(
    variant: ButtonVariant,
    #[props(default)] size: Option<ButtonSize>,
    #[props(into)] label: TextLine,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] pressed: Option<Check>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] title: Option<String>,
    #[props(default)] expanded: Option<Shown>,
    #[props(default)] trailing: Option<Trailing>,
    #[props(default)] leading: Option<Leading>,
    #[props(default)] face: ButtonFace,
    #[props(default)] propagation: Propagation,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-button");
    let data = common.data_attributes();
    let aria_label = common
        .aria_label
        .clone()
        .or_else(|| spoken_label(face, &label));
    let pressed = pressed.map(|state| state.aria());
    let expanded = expanded.map(Shown::aria);
    let listen = PressListeners::new(onclick).with_propagation(propagation);
    let pressing = use_pressing();
    let live = availability == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            id: common.id.clone(),
            class,
            "data-variant": variant.slug(),
            title,
            "aria-label": aria_label,
            "aria-pressed": pressed,
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            disabled: disabled(availability),
            "data-size": size.map(ButtonSize::slug),
            "data-pressed": if live { pressing.attr() } else { None },
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: move |_| pressing.released(),
            onkeydown: move |event| pressing.key_down(&event),
            onkeyup: move |_| pressing.released(),
            onblur: move |_| pressing.released(),
            onclick: move |event| {
                if live {
                    listen.click(&event);
                }
            },
            oncontextmenu: move |event| {
                if live {
                    listen.context_menu(&event);
                }
            },
            onmouseup: move |event| {
                pressing.released();
                if live {
                    listen.mouse_up(&event);
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: move |event| common.mounted(event),
            // The consumer's own `data-*`, last: a spread follows the named
            // attributes.
            ..data,
            if let Some(mark) = leading {
                {leading_mark(mark, variant.icon_size())}
            }
            if let Some(icon) = icon {
                IconView { source: icon, size: variant.icon_size() }
            }
            FaceMark { face, label }
            if let Some(mark) = trailing {
                {trailing_mark(mark, variant.icon_size())}
            }
        }
    }
}
