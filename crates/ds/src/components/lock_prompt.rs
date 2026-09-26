//! LockPrompt: who is asked, and the password field (design/20-SURFACES.md section 1.9;
//! design/04-COMPONENTS.md section 42). The avatar, the name, a pill field of flat white glass
//! with an enter arrow inside it, a caps-lock mark, and a hint line under it.

use crate::components::avatar::{AvatarFace, AvatarSize, face};
use crate::components::lock_vocab::{CapsLock, LockLook, LockUser, PromptState};
use crate::components::secret_entry::{Filled, SecretEntry, use_secret_entry};
use crate::components::spinner::{Spinner, SpinnerKind};
use crate::components::text_input::{Focus, InputVariant, TextInput, TextInputKind};
use crate::components::text_runs::{Text, text};
use crate::components::vocab::Availability;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// The placeholder when the caller gives none.
const ENTER_PASSWORD: &str = "Enter Password";

/// The lock screen's prompt. The password is a `Secret` field: its text never reaches the
/// markup, and there is no `value` prop. `oninput` hears every change (an empty string when the
/// prompt empties the field itself); Enter or the arrow hands the text to `onsubmit`; Escape
/// empties the field. `state` is the caller's: `Checking` spins the arrow and closes the field,
/// `Wrong` shakes the field once and empties it when the shake settles, `LockedOut` closes the
/// field and says when it opens. `caps` marks caps lock; `hint` is the line under the field
/// ("Touch the key or enter your password"). The field takes the keyboard as it mounts.
#[component]
pub fn LockPrompt(
    user: LockUser,
    #[props(default)] state: PromptState,
    #[props(default)] caps: CapsLock,
    #[props(default)] look: LockLook,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] hint: Option<Text>,
    oninput: EventHandler<String>,
    onsubmit: EventHandler<String>,
) -> Element {
    let entry = use_secret_entry(&state, oninput);
    let avatar = AvatarFace {
        size: AvatarSize::Size64,
        ..user.avatar
    };
    let line = hint_line(&state, hint);
    rsx! {
        div {
            class: "ds-lock-prompt",
            "data-look": look.slug(),
            "data-state": state.slug(),
            {face(avatar)}
            div { class: "ds-lock-name", "{user.name}" }
            {lock_field(entry, &state, caps, placeholder.unwrap_or_else(|| ENTER_PASSWORD.to_owned()), onsubmit)}
            if let Some(line) = line {
                div { class: "ds-lock-hint", {text(&line)} }
            }
        }
    }
}

/// The line under the field: the lock-out's time while locked out, else the caller's hint.
fn hint_line(state: &PromptState, hint: Option<Text>) -> Option<Text> {
    match state {
        PromptState::LockedOut { until } => Some(Text::from(format!("Try again at {until}"))),
        _ => hint,
    }
}

/// The pill: the secret field, the caps mark and the enter button, shaking as one.
fn lock_field(
    entry: SecretEntry,
    state: &PromptState,
    caps: CapsLock,
    placeholder: String,
    onsubmit: EventHandler<String>,
) -> Element {
    let availability = state.availability();
    let pulse = entry.pulse.attrs();
    let class = match &pulse {
        Some((anim, _)) => format!("ds-lock-field {anim}"),
        None => "ds-lock-field".to_owned(),
    };
    let alias = pulse.map(|(_, alias)| alias);
    let go = move || {
        if availability == Availability::Enabled {
            entry.submit(onsubmit);
        }
    };
    rsx! {
        div {
            class,
            "data-pulse": alias,
            "data-filled": entry.filled().slug(),
            "aria-disabled": availability.aria_disabled(),
            for round in [entry.key()] {
                TextInput {
                    key: "{round}",
                    variant: InputVariant::Inline,
                    kind: TextInputKind::Secret,
                    label: "Password",
                    value: "",
                    placeholder: placeholder.clone(),
                    availability,
                    focus: Focus::OnMount,
                    oninput: move |next: String| entry.input(next),
                    onkey: move |event: KeyboardEvent| match event.key() {
                        Key::Enter => go(),
                        Key::Escape => {
                            event.prevent_default();
                            entry.clear();
                        }
                        _ => {}
                    },
                }
            }
            if caps == CapsLock::On {
                span { class: "ds-lock-caps", role: "img", "aria-label": "Caps Lock is on",
                    Glyph { icon: Icon::CapsLock, size: IconSize::Compact }
                }
            }
            {go_button(state, entry.filled(), EventHandler::new(move |()| go()))}
        }
    }
}

/// The enter arrow inside the pill: a spinner while the password is tried.
fn go_button(state: &PromptState, filled: Filled, onpress: EventHandler<()>) -> Element {
    let availability = state.availability();
    let checking = *state == PromptState::Checking;
    rsx! {
        button {
            r#type: "button",
            class: "ds-lock-go",
            "aria-label": "Unlock",
            "data-filled": filled.slug(),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": checking.then_some("true"),
            onclick: move |_| onpress.call(()),
            if checking {
                span { class: "ds-lock-busy",
                    Spinner { kind: SpinnerKind::Spin }
                }
            } else {
                Glyph { icon: Icon::ArrowRight, size: IconSize::Small }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::hint_line;
    use crate::components::lock_vocab::PromptState;
    use crate::components::text_runs::Text;

    #[test]
    fn a_lock_out_speaks_over_the_hint() {
        let hint = || Some(Text::from("Enter your password"));
        let out = PromptState::LockedOut {
            until: "9:52".into(),
        };
        assert_eq!(
            hint_line(&out, hint()).map(|line| line.plain_text()),
            Some("Try again at 9:52".to_owned())
        );
        assert_eq!(
            hint_line(&PromptState::Wrong, hint()).map(|line| line.plain_text()),
            Some("Enter your password".to_owned())
        );
        assert_eq!(hint_line(&PromptState::Idle, None), None);
    }
}
