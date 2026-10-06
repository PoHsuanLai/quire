//! The controlled shape on a real Blitz document: the caller holds the value, the component
//! shows it and says what the person asked for. A secure field the caller holds shows one dot
//! per character, refuses copy and cut and takes paste; a plain field submits on Enter alone and
//! shows its error text; a list's cursor is a key the caller holds and survives a reorder; a
//! pop-up button's open state is the caller's; a sheet's Return presses its default button.

use dioxus::prelude::*;
use ds::components::controls::button_model::{Answers, ButtonFocus};
use ds::components::fields::text_field_model::Invalid;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::motion::detail::stamp::EventStamp;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn harness(app: fn() -> Element) -> Harness {
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(c)));
        harness.advance(ms(20));
    }
}

fn chord(harness: &mut Harness, key: char) {
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char(key)));
    harness.advance(ms(20));
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect(selector);
    harness.send(Input::click(at));
    harness.advance(ms(30));
}

/// A secure field and a plain one, the caller holding both texts.
#[allow(non_snake_case)]
fn Secret() -> Element {
    let mut secret = use_signal(String::new);
    let mut plain = use_signal(|| "visible".to_owned());
    let mut sent = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "secret", style: "display:flex; width:300px; padding:12px",
                TextField {
                    kind: FieldKind::Secure,
                    text: FieldText::Caller,
                    label: "Password",
                    value: secret(),
                    oninput: move |text: String| secret.set(text),
                    onsubmit: move |text: String| {
                        sent.with_mut(|all| all.push(text));
                        secret.set(String::new());
                    },
                }
            }
            div { id: "plain", style: "display:flex; width:300px; padding:12px",
                TextField {
                    label: "Name",
                    value: plain(),
                    oninput: move |text: String| plain.set(text),
                }
            }
            p { class: "held", {secret()} }
            p { class: "sent", {sent().join(",")} }
        }
    }
}

fn dots(harness: &Harness) -> usize {
    harness
        .text_of("#secret .ds-input-mask")
        .map_or(0, |text| text.chars().count())
}

#[test]
fn a_secure_field_the_caller_holds_masks_its_value_and_keeps_it_from_the_clipboard() {
    let mut harness = harness(Secret);
    click(&mut harness, "#secret input");
    type_text(&mut harness, "abc");
    assert_eq!(harness.text_of(".held").as_deref(), Some("abc"));
    assert_eq!(
        dots(&harness),
        3,
        "one dot per character of the caller's value"
    );
    assert_eq!(
        harness.attr("#secret input", "value"),
        None,
        "never the text"
    );
    let markup = harness.html();
    let field = markup
        .split("id=\"secret\"")
        .nth(1)
        .and_then(|rest| rest.split("id=\"plain\"").next())
        .expect("the secure field's markup");
    assert!(!field.contains("abc"), "{field}");

    chord(&mut harness, 'a');
    chord(&mut harness, 'c');
    assert_eq!(harness.clipboard_text(), None, "copy is refused");
    chord(&mut harness, 'x');
    assert_eq!(harness.clipboard_text(), None, "cut is refused");
    assert_eq!(
        harness.text_of(".held").as_deref(),
        Some("abc"),
        "and cuts nothing"
    );

    harness.set_clipboard_text("zz");
    chord(&mut harness, 'v');
    assert_eq!(
        harness.text_of(".held").as_deref(),
        Some("zz"),
        "paste replaces"
    );
    assert_eq!(dots(&harness), 2);

    // The same chords copy out of the plain field beside it.
    click(&mut harness, "#plain input");
    chord(&mut harness, 'a');
    chord(&mut harness, 'c');
    assert_eq!(harness.clipboard_text().as_deref(), Some("visible"));
}

#[test]
fn submitting_hands_over_the_text_and_a_value_the_caller_empties_empties_the_field() {
    let mut harness = harness(Secret);
    click(&mut harness, "#secret input");
    type_text(&mut harness, "hunter2");
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".sent").as_deref(), Some("hunter2"));
    assert_eq!(dots(&harness), 0, "the caller emptied it");
    harness.advance(ms(100));
    assert_eq!(
        harness.focus_of("#secret input"),
        FocusState::Focused,
        "the caret stays in the field it was in"
    );
    type_text(&mut harness, "q");
    assert_eq!(
        harness.text_of(".held").as_deref(),
        Some("q"),
        "the next keystroke starts from nothing, not from the old text"
    );
}

/// A plain field: its submits, its commits and its error.
#[allow(non_snake_case)]
fn Named() -> Element {
    let mut name = use_signal(String::new);
    let mut log = use_signal(Vec::<String>::new);
    let validity = if name().is_empty() {
        Validity::Valid
    } else {
        Validity::Invalid(Invalid {
            message: TextLine::from("Names are for people"),
            stamp: EventStamp(1),
        })
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "name", style: "display:flex; width:300px; padding:12px",
                TextField {
                    label: "Name",
                    value: name(),
                    validity,
                    oninput: move |text: String| name.set(text),
                    onsubmit: move |text: String| log.with_mut(|all| all.push(format!("submit:{text}"))),
                    onchange: move |text: String| log.with_mut(|all| all.push(format!("change:{text}"))),
                }
            }
            div { id: "other", style: "display:flex; width:300px; padding:12px",
                TextField { label: "Other", value: "", oninput: |_| {} }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn a_plain_field_submits_on_enter_alone_and_shows_its_error_under_it() {
    let mut harness = harness(Named);
    assert_eq!(harness.text_of("#name .ds-text-field-help"), None);
    click(&mut harness, "#name input");
    type_text(&mut harness, "ab");
    assert_eq!(
        harness.text_of("#name .ds-text-field-help").as_deref(),
        Some("Names are for people")
    );
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(30));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("change:ab,submit:ab")
    );
    click(&mut harness, "#other input");
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("change:ab,submit:ab,change:ab"),
        "the caret leaving commits and does not submit"
    );
}

const PROVIDERS: [(&str, &str); 4] = [
    ("anthropic", "Anthropic"),
    ("google", "Google"),
    ("icloud", "iCloud"),
    ("imap", "Other IMAP"),
];

/// A provider list the caller holds the selection of, by key; `Reorder` swaps the first two.
#[allow(non_snake_case)]
fn Providers() -> Element {
    let mut picked = use_signal(|| None::<&'static str>);
    let mut flipped = use_signal(|| false);
    let mut order: Vec<(&'static str, &'static str)> = PROVIDERS.to_vec();
    if flipped() {
        order.swap(0, 1);
    }
    let items: Vec<ListItem<&'static str>> = order
        .into_iter()
        .map(|(key, label)| {
            let item = ListItem::row(key, label, rsx! { div { id: "p-{key}", "{label}" } });
            if key == "icloud" {
                item.with(Availability::Disabled)
            } else {
                item
            }
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            List::<&'static str> {
                label: "Providers",
                items,
                cursor: picked(),
                onselect: move |key| picked.set(Some(key)),
            }
            Button { label: "Reorder", onclick: move |_| flipped.set(true), common: Common { id: Some("flip".into()), ..Common::default() } }
            p { class: "picked", {picked().unwrap_or("none")} }
        }
    }
}

#[test]
fn a_list_cursor_is_a_key_the_caller_holds_and_follows_the_key_through_a_reorder() {
    let mut harness = harness(Providers);
    click(&mut harness, "#p-anthropic");
    let press = |harness: &mut Harness, key: ShortcutKey, want: &str, why: &str| {
        harness.send(Input::key(key));
        harness.advance(ms(20));
        assert_eq!(harness.text_of(".picked").as_deref(), Some(want), "{why}");
    };
    press(
        &mut harness,
        ShortcutKey::Down,
        "anthropic",
        "no cursor yet: the first row",
    );
    press(&mut harness, ShortcutKey::Down, "google", "next");
    press(
        &mut harness,
        ShortcutKey::Down,
        "imap",
        "the disabled row is passed over",
    );
    press(&mut harness, ShortcutKey::Home, "anthropic", "Home");
    press(
        &mut harness,
        ShortcutKey::Char('o'),
        "imap",
        "type-ahead: the label starting with o",
    );
    press(&mut harness, ShortcutKey::Home, "anthropic", "Home again");
    press(&mut harness, ShortcutKey::Down, "google", "next");

    // Anthropic and Google swap places. The cursor is a key, so it is still on Google, which is
    // now first: Down reaches Anthropic.
    click(&mut harness, "#flip");
    click(&mut harness, "#p-google");
    press(
        &mut harness,
        ShortcutKey::Down,
        "anthropic",
        "the cursor followed the key",
    );
}

/// A pop-up the caller opens and closes, logging what the person asked.
#[allow(non_snake_case)]
fn Driven() -> Element {
    let mut open = use_signal(|| Shown::Hidden);
    let mut asked = use_signal(Vec::<String>::new);
    let mut value = use_signal(|| 1u8);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PopUpButton::<u8> {
                items: vec![MenuItem::new(1, "Work"), MenuItem::new(2, "Home")],
                value: Some(value()),
                onpick: move |next| value.set(next),
                open: Some(open()),
                on_open_change: move |to: Shown| asked.with_mut(|all| all.push(to.slug().to_owned())),
            }
            Button { label: "Open", onclick: move |_| open.set(Shown::Visible), common: Common { id: Some("open".into()), ..Common::default() } }
            p { class: "asked", {asked().join(",")} }
            p { class: "value", "{value()}" }
        }
    }
}

#[test]
fn a_pop_up_opens_when_the_caller_says_and_only_asks_when_the_person_presses() {
    let mut harness = harness(Driven);
    assert!(harness.rect(".ds-menu").is_none());
    click(&mut harness, "#open");
    assert!(harness.rect(".ds-menu").is_some(), "the caller opened it");
    assert_eq!(
        harness.text_of(".asked").as_deref(),
        Some(""),
        "and was not asked"
    );
    let second = harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the second item");
    harness.send(Input::click(second));
    settle_until(&mut harness, |h| {
        h.text_of(".asked").as_deref() == Some("hidden")
    });
    assert_eq!(
        harness.text_of(".asked").as_deref(),
        Some("hidden"),
        "a pick asks for the menu to close, and the caller was only asked"
    );
    assert_eq!(harness.text_of(".value").as_deref(), Some("2"));
}

/// A sheet with a field, a default button and a cancel.
#[allow(non_snake_case)]
fn Asking() -> Element {
    let mut name = use_signal(String::new);
    let mut log = use_signal(Vec::<String>::new);
    let ready = !name().is_empty();
    let mut add = move || log.with_mut(|all| all.push(format!("add:{}", name())));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Sheet {
                label: "Add account",
                attach: Attach::Centre,
                onclose: move |()| log.with_mut(|all| all.push("cancel".to_owned())),
                on_return: ready.then(|| EventHandler::new(move |()| add())),
                div { style: "display:flex; flex-direction:column; gap:8px; padding:12px",
                    div { id: "who", style: "display:flex",
                        TextField {
                            label: "Account",
                            value: name(),
                            focus: FieldFocus::OnMount,
                            oninput: move |text: String| name.set(text),
                        }
                    }
                    Button {
                        label: "Add",
                        answers: Answers::Return,
                        availability: if ready { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| add(),
                        common: Common { id: Some("add".into()), ..Common::default() },
                    }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn return_in_a_sheet_presses_its_default_button_while_that_is_enabled() {
    let mut harness = harness(Asking);
    harness.advance(ms(600));
    assert_eq!(
        harness.focus_of("#who input"),
        FocusState::Focused,
        "first field"
    );
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(30));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some(""),
        "disabled: Return does nothing"
    );
    type_text(&mut harness, "ada");
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".log").as_deref(), Some("add:ada"));
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".log").as_deref(), Some("add:ada,cancel"));
}

/// A sheet whose first stop is its default button.
#[allow(non_snake_case)]
fn Confirming() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Sheet { label: "Remove", attach: Attach::Centre, onclose: |()| {},
                Button {
                    label: "Remove",
                    answers: Answers::Return,
                    focus: ButtonFocus::OnMount,
                    onclick: |_| {},
                    common: Common { id: Some("default".into()), ..Common::default() },
                }
            }
        }
    }
}

#[test]
fn a_default_button_can_take_the_keyboard_as_its_sheet_opens() {
    let mut harness = harness(Confirming);
    harness.advance(ms(600));
    assert_eq!(harness.focus_of("#default"), FocusState::Focused);
}

/// A sheet whose only stop is a toggle row, with a default button.
#[allow(non_snake_case)]
fn Switching() -> Element {
    let mut on = use_signal(|| Check::Off);
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Sheet {
                label: "Options",
                attach: Attach::Centre,
                onclose: |()| {},
                on_return: EventHandler::new(move |()| log.with_mut(|all| all.push("default".to_owned()))),
                div { id: "options", style: "width:320px",
                    Row {
                        title: "Sync mail",
                        accessory: Accessory::Toggle {
                            value: on(),
                            on_toggle: EventHandler::new(move |next: Check| on.set(next)),
                        },
                    }
                }
            }
            p { class: "on", "{on().slug()}" }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn a_toggle_row_flips_on_space_and_leaves_return_to_the_sheets_default_button() {
    let mut harness = harness(Switching);
    harness.advance(ms(600));
    click(&mut harness, "#options .ds-toggle");
    let flipped = harness.text_of(".on");
    harness.send(Input::key(ShortcutKey::Space));
    harness.advance(ms(30));
    assert_ne!(harness.text_of(".on"), flipped, "Space flips the toggle");
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(30));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("default"),
        "Return reached the sheet"
    );
}
