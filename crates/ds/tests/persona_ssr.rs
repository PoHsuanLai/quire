//! The persona as markup (design/24-PERSONA.md): three seeds and the default spec in every mood
//! match their goldens under `tests/snapshots/persona/`, lint clean, and use only `ds-` classes
//! the stylesheet styles; `UserPortrait` draws either kind of picture.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test persona_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, Inject, Material, Mood,
    Persona, PersonaFinish, PersonaSize, PersonaSpec, Theme, UserPicture, UserPortrait,
};

/// The seeds with goldens.
const SEEDS: [u64; 3] = [3, 17, 42];

#[derive(Props, Clone, PartialEq)]
struct SpecimenProps {
    spec: PersonaSpec,
    mood: Mood,
    size: PersonaSize,
    theme: Theme,
    finish: PersonaFinish,
}

#[allow(non_snake_case)]
fn Specimen(props: SpecimenProps) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: props.theme, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Host,
            Persona { spec: props.spec, size: props.size, mood: props.mood, finish: props.finish }
        }
    }
}

fn render(props: SpecimenProps) -> String {
    let mut dom = VirtualDom::new_with_props(Specimen, props);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn large(spec: PersonaSpec, mood: Mood) -> SpecimenProps {
    SpecimenProps {
        spec,
        mood,
        size: PersonaSize::Large,
        theme: Theme::Light,
        finish: PersonaFinish::Colour,
    }
}

/// Every golden: its name and its props.
fn specimens() -> Vec<(String, SpecimenProps)> {
    let mut all = Vec::new();
    for seed in SEEDS {
        for mood in Mood::ALL {
            all.push((
                format!("seed-{seed}-{}", mood.slug()),
                large(PersonaSpec::from_seed(seed), mood),
            ));
        }
    }
    all.push((
        "default-idle".into(),
        large(PersonaSpec::default(), Mood::Idle),
    ));
    all.push((
        "seed-17-small-dark".into(),
        SpecimenProps {
            size: PersonaSize::Small,
            theme: Theme::Dark,
            ..large(PersonaSpec::from_seed(17), Mood::Idle)
        },
    ));
    all.push((
        "seed-42-muted".into(),
        SpecimenProps {
            finish: PersonaFinish::Muted,
            ..large(PersonaSpec::from_seed(42), Mood::Idle)
        },
    ));
    all
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = specimens()
        .into_iter()
        .filter_map(|(name, props)| {
            golden::check(&format!("persona/{name}.html"), &render(props)).err()
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

/// Each mood draws its own face: no two moods of one seed render the same markup, and the
/// sleeping `z` is drawn only asleep.
#[test]
fn every_mood_draws_a_different_face() {
    for seed in SEEDS {
        let faces: Vec<String> = Mood::ALL
            .iter()
            .map(|mood| render(large(PersonaSpec::from_seed(seed), *mood)))
            .collect();
        for (index, face) in faces.iter().enumerate() {
            for other in &faces[index + 1..] {
                assert_ne!(face, other, "seed {seed}: two moods look alike");
            }
            let asleep = Mood::ALL[index] == Mood::Asleep;
            assert_eq!(
                face.contains("ds-persona-z"),
                asleep,
                "seed {seed} {:?}",
                Mood::ALL[index]
            );
        }
    }
}

#[allow(non_snake_case)]
fn Pictures() -> Element {
    let letter = AvatarFace {
        initial: 'P',
        size: AvatarSize::Size34,
        tone: AvatarTone::Ink,
        shape: AvatarShape::Round,
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host,
            UserPortrait { picture: UserPicture::Face(letter), size: PersonaSize::Medium }
            UserPortrait { picture: UserPicture::Persona(PersonaSpec::default()), size: PersonaSize::Medium, mood: Mood::Happy }
        }
    }
}

#[test]
fn a_user_picture_draws_either_kind() {
    let mut dom = VirtualDom::new(Pictures);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("class=\"ds-avatar\""), "{html}");
    assert!(html.contains(">P</span>"), "{html}");
    assert!(html.contains("data-mood=\"happy\""), "{html}");
    assert!(html.contains("data-size=\"64\""), "{html}");
}
