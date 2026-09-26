//! PolkitPrompt: a program asks for the person's password (design/20-SURFACES.md section 1.10;
//! design/04-COMPONENTS.md section 42). A narrow centred `Sheet` over a modal scrim, entering
//! with `peek-in`: the person's avatar, a bold title, the program's message, "Details" as a hover
//! card, the password in a boxed `Secret` field that shakes once when it was wrong, then Cancel
//! and Authenticate.

use crate::components::avatar::{AvatarFace, AvatarSize, face};
use crate::components::button::{Button, ButtonVariant};
use crate::components::lock_vocab::{CapsLock, LockUser, PromptState};
use crate::components::scrim_strength::ScrimStrength;
use crate::components::secret_entry::{SecretEntry, use_secret_entry};
use crate::components::sheet::{Sheet, SheetPlacement, SheetWidth};
use crate::components::text_input::{Focus, InputVariant, TextInput, TextInputKind};
use crate::components::text_runs::{Text, text};
use crate::components::tooltip::{Shown, Tooltip, TooltipKind};
use crate::components::vocab::Availability;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// The title when the caller gives none.
const AUTHENTICATE: &str = "Authentication Required";

/// A password prompt for a privileged action. `action` is the program's message ("Authentication
/// is required to change the system's time"); `detail` (the action id, the program's path)
/// shows as a hover card on "Details". The password is a `Secret` field (no `value` prop, its
/// text never in the markup): `oninput` hears it, Enter or Authenticate hands it to `onsubmit`,
/// and Cancel, Escape or a click on the scrim call `oncancel`. `state` is the caller's, as for
/// [`crate::LockPrompt`]: `Checking` closes the field and the buttons but Cancel, `Wrong` shakes
/// the field once and empties it, `LockedOut` says when it opens again. `shown` and `on_hidden`
/// are the sheet's own, for a host that unmaps its surface after the exit.
#[component]
pub fn PolkitPrompt(
    #[props(into)] action: Text,
    #[props(default)] detail: Option<Text>,
    #[props(default)] title: Option<String>,
    user: LockUser,
    #[props(default)] state: PromptState,
    #[props(default)] caps: CapsLock,
    oninput: EventHandler<String>,
    onsubmit: EventHandler<String>,
    oncancel: EventHandler<()>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
) -> Element {
    let entry = use_secret_entry(&state, oninput);
    let availability = state.availability();
    let avatar = AvatarFace {
        size: AvatarSize::Size48,
        ..user.avatar
    };
    let go = move || {
        if availability == Availability::Enabled {
            entry.submit(onsubmit);
        }
    };
    let out = match &state {
        PromptState::LockedOut { until } => Some(format!("Too many tries. Try again at {until}.")),
        _ => None,
    };
    rsx! {
        Sheet {
            label: "Authenticate",
            onclose: move |()| oncancel.call(()),
            shown,
            on_hidden,
            placement: SheetPlacement::Centre,
            scrim: ScrimStrength::Modal,
            width: SheetWidth::Narrow,
            div { class: "ds-polkit", "data-state": state.slug(),
                {face(avatar)}
                div { class: "ds-polkit-title", {title.unwrap_or_else(|| AUTHENTICATE.to_owned())} }
                div { class: "ds-polkit-action", {text(&action)} }
                if let Some(detail) = detail {
                    div { class: "ds-polkit-details",
                        Tooltip { kind: TooltipKind::Card, text: detail.plain_text(),
                            span { class: "ds-polkit-details-word", "Details" }
                        }
                    }
                }
                div { class: "ds-polkit-user", "{user.name}" }
                {field(entry, availability, caps, EventHandler::new(move |()| go()))}
                if let Some(line) = out {
                    div { class: "ds-polkit-hint", "{line}" }
                }
                div { class: "ds-polkit-actions",
                    Button { variant: ButtonVariant::Secondary, label: "Cancel", onclick: move |_| oncancel.call(()) }
                    Button { variant: ButtonVariant::Primary, label: "Authenticate", availability, onclick: move |_| go() }
                }
            }
        }
    }
}

/// The boxed secret field, shaking as one with its caps mark. Escape is left to the sheet,
/// which cancels.
fn field(
    entry: SecretEntry,
    availability: Availability,
    caps: CapsLock,
    submit: EventHandler<()>,
) -> Element {
    let pulse = entry.pulse.attrs();
    let class = match &pulse {
        Some((anim, _)) => format!("ds-polkit-field {anim}"),
        None => "ds-polkit-field".to_owned(),
    };
    let alias = pulse.map(|(_, alias)| alias);
    rsx! {
        div { class, "data-pulse": alias,
            for round in [entry.key()] {
                TextInput {
                    key: "{round}",
                    variant: InputVariant::Boxed,
                    kind: TextInputKind::Secret,
                    label: "Password",
                    value: "",
                    placeholder: "Password",
                    availability,
                    focus: Focus::OnMount,
                    oninput: move |next: String| entry.input(next),
                    onkey: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter {
                            submit.call(());
                        }
                    },
                }
            }
            if caps == CapsLock::On {
                span { class: "ds-polkit-caps", role: "img", "aria-label": "Caps Lock is on",
                    Glyph { icon: Icon::CapsLock, size: IconSize::Compact }
                }
            }
        }
    }
}
