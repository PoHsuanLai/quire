//! IconButton: an icon-only action, `aria-label` mandatory (design/04-COMPONENTS.md section 2).

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::controls::button_size::disabled;
use crate::components::controls::pass_through::{DataAttr, ExtraClass, attributes, class_list};
use crate::components::controls::press::{PressListeners, Propagation};
use crate::core::press::Press;
use crate::core::vocab::{Availability, Check, Shown};
use crate::core::word::Word;
use crate::motion::detail::{cue::Cue, first_show::FirstShow, once::use_nudge};
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// Which icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum IconButtonVariant {
    /// Reader and composer tools, 28 x 28 (a Large control).
    Tool,
    /// Sidebar foot, 22 x 22 (a Regular control), on the frame.
    Foot,
    /// A row's hover strip, 26 x 26 round (mailo's, off the ladder until the app side is audited).
    Strip,
    /// A square tile on the frame (account tiles).
    Pin,
    /// A bar status item on the frame (design/13-BEHAVIOUR-menus-windows.md section 13.3.1):
    /// a slot `--bar-status-w` (30) wide and `--bar-status-box` tall holding a glyph of `--bar-status-glyph`, both set by the
    /// consumer from its settings ([`StatusMetrics`]); `--f-ink-soft` at rest, `--f-ink` on
    /// `--f-pill-hover` under the pointer, on `--f-pill` pressed or while its menu is open.
    Status,
}

impl IconButtonVariant {
    /// The glyph size from the section's geometry table: Tool and Foot 16, Strip 14. The Pin's
    /// content is the account tile's (section 27); a bare glyph on a Pin takes the base 16. A
    /// Status glyph is written at the base 16 and sized by `--bar-status-glyph` in the sheet.
    fn icon_size(self) -> IconSize {
        match self {
            IconButtonVariant::Tool
            | IconButtonVariant::Foot
            | IconButtonVariant::Pin
            | IconButtonVariant::Status => IconSize::Base,
            IconButtonVariant::Strip => IconSize::Compact,
        }
    }
}

/// An icon-only action. `icon` is a glyph or an external icon (an `Icon` converts). `id` is
/// written as the element's `id`, so a popup can anchor to it by id. `onclick` hears the
/// primary, secondary (right-click) and middle buttons, and the keyboard as primary.
/// `mounted` hands over the element once it is in the document, so a floating component can
/// anchor to it (`Anchor::Mounted`). `propagation: Propagation::Stop` keeps the press at the
/// button, so a glyph inside a `<summary>` does not toggle its `<details>`.
/// `availability: Availability::Disabled` writes `aria-disabled` and `disabled` and draws the
/// button at .35 with no hover and no press; `onclick` never runs.
///
/// `data` and `extra_class` put the consumer's own `data-*` attributes and
/// classes on the button itself, as on a `Button`, so it needs no wrapping `span`: `data-folder` for a drag that
/// reads the place off the element under the pointer, a class for the consumer's own reveal or
/// layout rule. Both are checked when built ([`DataName::parse`], [`ExtraClass::parse`]): a
/// `ds-` name or class, or a `data-*` name quire writes itself, is refused, so nothing added
/// here can restyle the button through quire's rules.
///
/// A status item's glyph can be a layered status glyph: `icon: IconSource::Status(state)` (a
/// `StatusState` converts) draws `StatusGlyph` in the same box, ink, pill and label as an `Icon`,
/// sized by `--bar-status-glyph`; hand it the state every render and it plays its own
/// moments. `first` is the glyph's first frame (`Still`, the default, on bar chrome).
///
/// `nudge` is an attention cue (`use_detail(..).cue()` of a state whose table names
/// `Moment::Attention`, such as a low battery crossing into its threshold, design/26): each
/// new Attention cue lifts the glyph once (`nudge-up`, `use_nudge`), never the pill, and nothing
/// under Reduced. Pass it for the button's whole life (`None` to `Some` remounts the glyph).
///
/// [`DataName::parse`]: crate::DataName::parse
/// [`ExtraClass::parse`]: crate::ExtraClass::parse
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    #[props(into)] icon: IconSource,
    label: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] pressed: Option<Check>,
    #[props(default)] expanded: Option<Shown>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] id: Option<String>,
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)] propagation: Propagation,
    #[props(default)] data: Vec<DataAttr>,
    #[props(default)] extra_class: Option<ExtraClass>,
    #[props(default)] first: FirstShow,
    #[props(default)] nudge: Option<Cue>,
) -> Element {
    let class = class_list("ds-icon-button", extra_class.as_ref());
    let glyph = glyph_slot(icon, variant.icon_size(), first, nudge);
    let data = attributes(&data);
    let pressed = pressed.map(|state| state.aria());
    let expanded = expanded.map(|state| state.aria());
    let listen = PressListeners::new(onclick).with_propagation(propagation);
    let live = availability == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            id,
            class,
            "data-variant": variant.slug(),
            "aria-label": "{label}",
            title: tooltip,
            "aria-pressed": pressed,
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            disabled: disabled(availability),
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
                if live {
                    listen.mouse_up(&event);
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: move |event| {
                if let Some(mounted) = mounted {
                    mounted.call(event);
                }
            },
            // The consumer's own `data-*`, last: a spread follows the named
            // attributes.
            ..data,
            {glyph}
        }
    }
}

/// The button's glyph, inside the nudge's lifting wrapper when it has an attention cue.
fn glyph_slot(source: IconSource, size: IconSize, first: FirstShow, nudge: Option<Cue>) -> Element {
    match nudge {
        None => rsx! {
            IconView { source, size, first }
        },
        Some(cue) => rsx! {
            Nudged { cue,
                IconView { source, size, first }
            }
        },
    }
}

/// `span.ds-icon-nudge` around a glyph: `nudge-up` once per new Attention cue (R6).
#[component]
fn Nudged(cue: Cue, children: Element) -> Element {
    let (class, alias) = match use_nudge(cue).attrs() {
        Some((anim, alias)) => (format!("ds-icon-nudge {anim}"), Some(alias)),
        None => ("ds-icon-nudge".to_owned(), None),
    };
    rsx! {
        span { class, "data-pulse": alias, {children} }
    }
}
