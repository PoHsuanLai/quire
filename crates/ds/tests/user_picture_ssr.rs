//! The user's picture as markup (design/25-EMOJI.md section 7): the letter, an emoji at rest, a
//! photo and the picker match goldens under `tests/snapshots/user_picture/`, lint clean, and use
//! only `ds-` classes the stylesheet styles. A sheet's `data:` URI is replaced in the goldens by
//! its length, so a golden stays readable and still notices a different sheet. The stored
//! choice round-trips through serde by its stable names, and `resolve_picture` follows its table.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test user_picture_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, EmojiId, FaceFile,
    ImageSource, Inject, Material, PictureChoice, PictureSize, Theme, UserPicture,
    UserPicturePicker, UserPortrait, person_hue, resolve_picture,
};

fn letter() -> AvatarFace {
    AvatarFace {
        initial: 'D',
        size: AvatarSize::Size64,
        tone: AvatarTone::Person(person_hue("dana")),
        shape: AvatarShape::Round,
    }
}

fn photo() -> ImageSource {
    ImageSource("data:image/png;base64,AAAA".to_owned())
}

fn root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn portrait(picture: UserPicture) -> Element {
    root(rsx! { UserPortrait { picture, size: PictureSize::Medium } })
}

/// A named specimen and how to build it.
type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("letter", || portrait(letter().into())),
    ("emoji-rest", || portrait(EmojiId::HeartEyes.into())),
    ("photo", || portrait(photo().into())),
    ("picker", || {
        root(rsx! {
            UserPicturePicker {
                letter: letter(),
                choice: PictureChoice::Emoji(EmojiId::Fox),
                onpick: |_| {},
            }
        })
    }),
];

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// `html` with every `data:` URI but the stand-in photo's replaced by its length.
fn redacted(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find("data:image/png;base64,") {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find(['"', '&', ')']).unwrap_or(tail.len());
        match &tail[..end] {
            "data:image/png;base64,AAAA" => out.push_str(&tail[..end]),
            _ => out.push_str(&format!("data:image/png;base64,({} chars)", end)),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn by_name(name: &str) -> String {
    let make = SPECIMENS
        .iter()
        .find(|(named, _)| *named == name)
        .unwrap_or_else(|| panic!("a specimen {name}"))
        .1;
    render(make)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make)| {
            golden::check(
                &format!("user_picture/{name}.html"),
                &redacted(&render(*make)),
            )
            .err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make) in SPECIMENS {
        let html = render(*make);
        for offence in markup(&html, sheet, &LintConfig::default()) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Each kind draws in the same wrapper, in the same 64 px place: the letter as the avatar's
/// disc (its look unchanged), the emoji at its rest frame, the photo cropped round.
#[test]
fn each_kind_is_drawn_in_the_same_place() {
    let letter = by_name("letter");
    assert!(letter.contains("class=\"ds-user-picture\""), "{letter}");
    assert!(
        letter.contains("class=\"ds-avatar\"") && letter.contains("data-size=\"64\""),
        "{letter}"
    );
    let emoji = by_name("emoji-rest");
    for want in [
        "class=\"ds-user-picture\"",
        "class=\"ds-emoji\" data-size=\"64\" data-mood=\"idle\"",
        "data-emoji=\"heart-eyes\"",
        "data-frame=\"0\"",
    ] {
        assert!(emoji.contains(want), "{want} in {}", redacted(&emoji));
    }
    let photo = by_name("photo");
    assert!(
        photo.contains("class=\"ds-user-photo\" data-size=\"64\""),
        "{photo}"
    );
}

/// The picker offers the letter and the whole set, still, with the current choice checked.
#[test]
fn the_picker_offers_the_letter_and_every_emoji_still() {
    let picker = by_name("picker");
    assert_eq!(
        picker.matches("role=\"radio\"").count(),
        EmojiId::ALL.len() + 1
    );
    assert_eq!(picker.matches("aria-checked=\"true\"").count(), 1);
    assert!(
        picker.contains("aria-checked=\"true\" aria-label=\"Fox\""),
        "{}",
        redacted(&picker)
    );
    assert!(picker.contains("aria-label=\"Letter D\""));
    assert_eq!(
        picker.matches("data-playback=\"still\"").count(),
        EmojiId::ALL.len()
    );
}

#[test]
fn the_choice_is_stored_by_stable_names() {
    let cases = [
        (PictureChoice::Auto, r#"{"kind":"auto"}"#),
        (PictureChoice::Letter, r#"{"kind":"letter"}"#),
        (
            PictureChoice::Emoji(EmojiId::HeartEyes),
            r#"{"kind":"emoji","v":"heart-eyes"}"#,
        ),
        (PictureChoice::Photo, r#"{"kind":"photo"}"#),
    ];
    for (choice, json) in cases {
        assert_eq!(
            serde_json::to_string(&choice).expect("serialise"),
            json,
            "{choice:?}"
        );
        let back: PictureChoice = serde_json::from_str(json).expect("deserialise");
        assert_eq!(back, choice, "{json}");
    }
    for emoji in EmojiId::ALL {
        let choice = PictureChoice::Emoji(emoji);
        let json = serde_json::to_string(&choice).expect("serialise");
        assert!(json.contains(emoji.slug()), "{json}");
        let back: PictureChoice = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back, choice);
    }
    assert_eq!(PictureChoice::default(), PictureChoice::Auto);
}

#[test]
fn resolve_follows_its_table() {
    let found = || FaceFile::Found(photo());
    let cases = [
        (PictureChoice::Auto, found(), UserPicture::Photo(photo())),
        (PictureChoice::Auto, FaceFile::Missing, letter().into()),
        (PictureChoice::Letter, found(), letter().into()),
        (PictureChoice::Letter, FaceFile::Missing, letter().into()),
        (
            PictureChoice::Emoji(EmojiId::Wink),
            found(),
            UserPicture::Emoji(EmojiId::Wink),
        ),
        (
            PictureChoice::Emoji(EmojiId::Wink),
            FaceFile::Missing,
            UserPicture::Emoji(EmojiId::Wink),
        ),
        (PictureChoice::Photo, found(), UserPicture::Photo(photo())),
        (PictureChoice::Photo, FaceFile::Missing, letter().into()),
    ];
    for (choice, face, want) in cases {
        let label = format!("{choice:?} {face:?}");
        assert_eq!(resolve_picture(choice, face, letter()), want, "{label}");
    }
}
