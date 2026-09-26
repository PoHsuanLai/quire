//! Animated emoji as markup (design/25-EMOJI.md): a pick at each size, each mood, with and
//! without its disc, in both schemes, match goldens under `tests/snapshots/emoji/`, lint clean,
//! and use only `ds-` classes the stylesheet styles. The sheet's `data:` URI is replaced in the
//! goldens by its length, so a golden stays readable and still notices a different sheet.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test emoji_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    AnimatedEmoji, Appearance, Backdrop, Ds, EmojiDisc, EmojiId, Inject, Material, Mood,
    PersonaSize, Theme,
};

#[derive(Props, Clone, PartialEq)]
struct SpecimenProps {
    emoji: EmojiId,
    size: PersonaSize,
    mood: Mood,
    disc: EmojiDisc,
    theme: Theme,
}

#[allow(non_snake_case)]
fn Specimen(props: SpecimenProps) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: props.theme, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Host,
            AnimatedEmoji { emoji: props.emoji, size: props.size, mood: props.mood, disc: props.disc }
        }
    }
}

fn render(props: SpecimenProps) -> String {
    let mut dom = VirtualDom::new_with_props(Specimen, props);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// `html` with every `data:` URI replaced by its length.
fn redacted(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find("data:image/png;base64,") {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find(['"', '&', ')']).unwrap_or(tail.len());
        out.push_str(&format!("data:image/png;base64,({} chars)", end));
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn large(emoji: EmojiId, mood: Mood) -> SpecimenProps {
    SpecimenProps {
        emoji,
        size: PersonaSize::Large,
        mood,
        disc: EmojiDisc::None,
        theme: Theme::Light,
    }
}

fn specimens() -> Vec<(String, SpecimenProps)> {
    let mut all: Vec<(String, SpecimenProps)> = Mood::ALL
        .iter()
        .map(|mood| {
            (
                format!("default-{}", mood.slug()),
                large(EmojiId::default(), *mood),
            )
        })
        .collect();
    all.push((
        "wink-small".into(),
        SpecimenProps {
            size: PersonaSize::Small,
            ..large(EmojiId::Wink, Mood::Idle)
        },
    ));
    all.push((
        "heart-eyes-medium-disc".into(),
        SpecimenProps {
            size: PersonaSize::Medium,
            disc: EmojiDisc::Tinted(Backdrop::Teal),
            ..large(EmojiId::HeartEyes, Mood::Idle)
        },
    ));
    all.push((
        "fox-disc-dark".into(),
        SpecimenProps {
            disc: EmojiDisc::Tinted(Backdrop::Plum),
            theme: Theme::Dark,
            ..large(EmojiId::Fox, Mood::Idle)
        },
    ));
    all
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = specimens()
        .into_iter()
        .filter_map(|(name, props)| {
            golden::check(&format!("emoji/{name}.html"), &redacted(&render(props))).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, props) in specimens() {
        let html = render(props);
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

/// Asleep draws the sleeping face; every other mood draws the pick at its rest frame (the
/// first render, before any timer).
#[test]
fn a_mood_picks_the_face_at_rest() {
    for mood in Mood::ALL {
        let html = render(large(EmojiId::Wink, mood));
        let face = if mood == Mood::Asleep {
            "sleeping"
        } else {
            "wink"
        };
        assert!(
            html.contains(&format!("data-emoji=\"{face}\"")),
            "{mood:?}: {}",
            redacted(&html)
        );
        assert!(html.contains("data-frame=\"0\""), "{mood:?}");
        assert!(
            html.contains("background-position:0% 0%"),
            "{}",
            redacted(&html)
        );
    }
}
