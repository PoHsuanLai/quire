use super::{Control, control_of};
use ds_settings::schema::{KeyKind, TogglePair};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_owned()).collect()
}

fn toggle(a: &str, b: &str) -> KeyKind {
    KeyKind::Toggle {
        variants: [a.to_owned(), b.to_owned()],
    }
}

fn switch(on: &str, off: &str) -> Control {
    Control::Switch(TogglePair {
        on: on.to_owned(),
        off: off.to_owned(),
    })
}

#[test]
fn each_kind_gets_the_control_detent_gives_it() {
    let cases = [
        (toggle("on", "off"), switch("on", "off")),
        (toggle("off", "on"), switch("on", "off")),
        (toggle("show", "hide"), switch("show", "hide")),
        (
            toggle("traditional", "natural"),
            switch("natural", "traditional"),
        ),
        (toggle("standard", "reduced"), switch("reduced", "standard")),
        (toggle("system", "reduced"), switch("reduced", "system")),
        (toggle("disabled", "enabled"), switch("enabled", "disabled")),
        // A choice between two things is not a switch.
        (
            toggle("icons", "letters"),
            Control::Segments(words(&["icons", "letters"])),
        ),
        (
            toggle("off", "none"),
            Control::Segments(words(&["off", "none"])),
        ),
        (
            KeyKind::Segmented {
                variants: words(&["a", "b", "c"]),
            },
            Control::PopUp(words(&["a", "b", "c"])),
        ),
        (
            KeyKind::Segmented {
                variants: words(&["a", "b"]),
            },
            Control::Segments(words(&["a", "b"])),
        ),
        (
            KeyKind::Menu {
                variants: words(&["a", "b"]),
            },
            Control::PopUp(words(&["a", "b"])),
        ),
        (KeyKind::Text, Control::Shown),
        (KeyKind::Colour, Control::Shown),
        (KeyKind::Shortcut, Control::Shown),
        (
            KeyKind::Provided {
                capability: "look".to_owned(),
            },
            Control::Shown,
        ),
    ];
    for (kind, expect) in cases {
        assert_eq!(control_of(&kind), expect, "{kind:?}");
    }
}
