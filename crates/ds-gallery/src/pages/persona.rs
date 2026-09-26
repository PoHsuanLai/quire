//! The Persona page (design/24-PERSONA.md): 48 seeds at Large, every mood at Large for three
//! seeds, the three sizes side by side, and a user picture of each kind. Snapshots are CSS time
//! only, so each persona shows its mood's resting face; the motion is in the frame strips
//! (`--persona-frames`).

use super::{Caption, Section};
use dioxus::prelude::*;
use ds::{
    AvatarFace, AvatarShape, AvatarSize, AvatarTone, Creature, Eyes, HeadShape, Mood, Persona,
    PersonaFinish, PersonaSize, PersonaSpec, Tone, Top, UserPicture, UserPortrait, person_hue,
};

/// The seeds the moods and sizes are shown with.
pub const SHOWN: [u64; 3] = [3, 17, 42];

/// The page.
#[component]
pub fn PersonaPage() -> Element {
    rsx! {
        Section { title: "48 seeds", note: "PersonaSpec::from_seed(0..48) at Large, Idle: people, bears, cats, rabbits and blobs, every part a small enum, every colour derived from the icon palette's eight hues and five skin tones.",
            div { class: "g-persona-grid",
                for seed in 0..48u64 {
                    div { key: "{seed}", class: "g-col g-persona-cell",
                        Persona { spec: PersonaSpec::from_seed(seed), size: PersonaSize::Large }
                        Caption { name: format!("seed {seed}") }
                    }
                }
            }
        }
        Section { title: "Two finishes", note: "For the user's pick. A, Colour: the palette as derived (fur and discs at OKLCh chroma .12, brighter than the icons). B, Muted: every colour at .55 of its chroma, as the icons' Muted style mutes them. Seeds 0 to 15 in each.",
            div { class: "g-persona-pair",
                for (name, finish) in [("A  Colour", PersonaFinish::Colour), ("B  Muted", PersonaFinish::Muted)] {
                    div { key: "{name}", class: "g-col",
                        span { class: "g-name", "{name}" }
                        div { class: "g-persona-half",
                            for seed in 0..16u64 {
                                Persona { key: "{seed}", spec: PersonaSpec::from_seed(seed), size: PersonaSize::Medium, finish }
                            }
                        }
                    }
                }
            }
        }
        Section { title: "Every top on every creature", note: "Explicit specs may put any top on any creature (from_seed keeps hair for people): each row a creature, each column Bare, Tuft, Crop, Fringe, Bob, Bun and Curly, with the head shapes and eyes cycling.",
            div { class: "g-col",
                for (row, creature) in CREATURES.into_iter().enumerate() {
                    div { key: "{row}", class: "g-row",
                        for (column, top) in TOPS.into_iter().enumerate() {
                            Persona { key: "{column}", spec: combo(row, column, creature, top), size: PersonaSize::Medium }
                        }
                    }
                }
            }
        }
        Section { title: "Moods", note: "Idle, Attentive (eyes down toward the field), Wince (squint, worried brows, wobbly mouth), Happy (open smile, brows up) and Asleep (closed eyes, leaning, one z). The default spec first.",
            div { class: "g-col",
                MoodRow { spec: PersonaSpec::default(), name: "default".to_string() }
                for seed in SHOWN {
                    MoodRow { key: "{seed}", spec: PersonaSpec::from_seed(seed), name: format!("seed {seed}") }
                }
            }
        }
        Section { title: "Sizes", note: "Small 28, Medium 64, Large 128. Small drops brows, whiskers, freckles and eye flecks and draws the eyes and mouth a little heavier.",
            div { class: "g-row",
                for seed in SHOWN {
                    div { key: "{seed}", class: "g-row g-persona-sizes",
                        Persona { spec: PersonaSpec::from_seed(seed), size: PersonaSize::Small }
                        Persona { spec: PersonaSpec::from_seed(seed), size: PersonaSize::Medium }
                        Persona { spec: PersonaSpec::from_seed(seed), size: PersonaSize::Large }
                    }
                }
            }
        }
        Section { title: "User picture", note: "UserPicture::Face(AvatarFace) and UserPicture::Persona(PersonaSpec), each through UserPortrait: what the lock prompt will take.",
            div { class: "g-row",
                UserPortrait { picture: UserPicture::Face(letter()), size: PersonaSize::Medium }
                UserPortrait { picture: UserPicture::Persona(PersonaSpec::from_seed(SHOWN[0])), size: PersonaSize::Medium }
            }
        }
    }
}

const CREATURES: [Creature; 5] = [
    Creature::Person,
    Creature::Bear,
    Creature::Cat,
    Creature::Bunny,
    Creature::Blob,
];
const TOPS: [Top; 7] = [
    Top::Bare,
    Top::Tuft,
    Top::Crop,
    Top::Fringe,
    Top::Bob,
    Top::Bun,
    Top::Curly,
];

/// A spec for the combinations grid: `creature` wearing `top`, the rest cycling by position.
fn combo(row: usize, column: usize, creature: Creature, top: Top) -> PersonaSpec {
    let base = PersonaSpec::from_seed((row * 7 + column) as u64 + 100);
    let heads = [
        HeadShape::Round,
        HeadShape::Soft,
        HeadShape::Tall,
        HeadShape::Wide,
    ];
    let eyes = [Eyes::Dot, Eyes::Oval, Eyes::Shine, Eyes::Wide];
    let tone = match creature {
        Creature::Person => [
            Tone::Peach,
            Tone::Sand,
            Tone::Honey,
            Tone::Umber,
            Tone::Cocoa,
        ][column % 5],
        Creature::Bear | Creature::Cat | Creature::Bunny | Creature::Blob => base.tone,
    };
    let tone = match (creature, tone) {
        (Creature::Person, tone) => tone,
        (_, Tone::Peach | Tone::Sand | Tone::Honey | Tone::Umber | Tone::Cocoa) => Tone::Sage,
        (_, fur) => fur,
    };
    PersonaSpec {
        creature,
        top,
        tone,
        head: heads[(row + column) % 4],
        eyes: eyes[column % 4],
        ..base
    }
}

/// A letter disc, for the user picture row.
fn letter() -> AvatarFace {
    AvatarFace {
        initial: 'P',
        size: AvatarSize::Size34,
        tone: AvatarTone::Person(person_hue("po")),
        shape: AvatarShape::Round,
    }
}

/// One spec in every mood.
#[component]
fn MoodRow(spec: PersonaSpec, name: String) -> Element {
    rsx! {
        div { class: "g-row",
            span { class: "g-name g-persona-name", "{name}" }
            for mood in Mood::ALL {
                div { key: "{mood.slug()}", class: "g-col g-persona-cell",
                    Persona { spec, size: PersonaSize::Large, mood }
                    Caption { name: mood.slug().to_string() }
                }
            }
        }
    }
}
