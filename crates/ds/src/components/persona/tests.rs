//! Every part combination draws: each creature, head, top, eye, brow, mouth and accessory in
//! every mood at both levels of detail gives paths with finite numbers inside the canvas's
//! reach, and at least a head, two eyes and a mouth.

use super::face;
use super::head;
use super::mood::{Detail, Mood};
use super::spec::{Accessory, Brows, Creature, Eyes, HeadShape, Mouth, PersonaSpec, Top};

const CREATURES: [Creature; 5] = [
    Creature::Person,
    Creature::Bear,
    Creature::Cat,
    Creature::Bunny,
    Creature::Blob,
];
const HEADS: [HeadShape; 4] = [
    HeadShape::Round,
    HeadShape::Soft,
    HeadShape::Tall,
    HeadShape::Wide,
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
const EYES: [Eyes; 4] = [Eyes::Dot, Eyes::Oval, Eyes::Shine, Eyes::Wide];
const BROWS: [Brows; 3] = [Brows::Hidden, Brows::Soft, Brows::Straight];
const MOUTHS: [Mouth; 4] = [Mouth::Smile, Mouth::Cat, Mouth::Small, Mouth::Grin];
const WORN: [Accessory; 4] = [
    Accessory::Plain,
    Accessory::Glasses,
    Accessory::Freckles,
    Accessory::Bow,
];

/// Every number in a path, which must be finite and within -30..130 (a whisker or an ear may
/// leave the 100-unit canvas a little; nothing may fly off).
fn sane(d: &str) -> bool {
    d.split(|c: char| c.is_ascii_alphabetic() || c == ' ')
        .filter(|word| !word.is_empty())
        .all(|word| {
            word.parse::<f64>()
                .is_ok_and(|n| n.is_finite() && (-30.0..=130.0).contains(&n))
        })
}

#[test]
fn every_combination_draws() {
    let mut count = 0;
    for creature in CREATURES {
        for head_shape in HEADS {
            for top in TOPS {
                for (index, eyes) in EYES.into_iter().enumerate() {
                    let spec = PersonaSpec {
                        creature,
                        head: head_shape,
                        top,
                        eyes,
                        brows: BROWS[index % BROWS.len()],
                        mouth: MOUTHS[index],
                        accessory: WORN[index],
                        ..PersonaSpec::default()
                    };
                    for detail in [Detail::Fine, Detail::Coarse] {
                        let still = head::still(&spec, detail);
                        assert!(!still.is_empty(), "{spec:?}");
                        for mood in Mood::ALL {
                            let eyes = face::eyes(&spec, mood, detail);
                            let mouth = face::mouth(&spec, mood, detail);
                            assert!(eyes.len() >= 2 && !mouth.is_empty(), "{spec:?} {mood:?}");
                            let brows = face::brows(&spec, mood, detail);
                            for mark in still.iter().chain(&eyes).chain(&mouth).chain(&brows) {
                                assert!(sane(&mark.d), "{spec:?} {mood:?}: {}", mark.d);
                            }
                            count += 1;
                        }
                        for mark in face::worn(&spec) {
                            assert!(sane(&mark.d), "{spec:?}: {}", mark.d);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 5 * 4 * 7 * 4 * 2 * 5);
}
