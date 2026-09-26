//! A pleasant random persona from a number, and its blink rhythm (design/24-PERSONA.md
//! sections 3.3 and 4). SplitMix64: small, well mixed, the same on every machine.

use super::spec::{
    Accessory, Backdrop, Brows, Cheeks, Creature, Eyes, HairTone, HeadShape, Mouth, PersonaSeed,
    PersonaSpec, Tone, Top,
};
use std::time::Duration;

/// The shortest gap between two blinks.
pub const BLINK_MIN: Duration = Duration::from_millis(3000);
/// The longest gap between two blinks.
pub const BLINK_MAX: Duration = Duration::from_millis(6000);

/// A SplitMix64 stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stream(u64);

impl Stream {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// One of `items`, each as likely as the others.
    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        let len = u64::try_from(items.len()).unwrap_or(1).max(1);
        let index = usize::try_from(self.next() % len).unwrap_or_default();
        items[index.min(items.len() - 1)]
    }

    /// One of `items`, weighted.
    fn weighted<T: Copy>(&mut self, items: &[(T, u64)]) -> T {
        let total: u64 = items.iter().map(|(_, weight)| weight).sum::<u64>().max(1);
        let mut roll = self.next() % total;
        for &(item, weight) in items {
            if roll < weight {
                return item;
            }
            roll -= weight;
        }
        items[0].0
    }
}

const SKIN: [Tone; 5] = [
    Tone::Peach,
    Tone::Sand,
    Tone::Honey,
    Tone::Umber,
    Tone::Cocoa,
];
const FUR: [Tone; 8] = [
    Tone::Clay,
    Tone::Ochre,
    Tone::Sage,
    Tone::Jade,
    Tone::Teal,
    Tone::Slate,
    Tone::Plum,
    Tone::Rose,
];
const BACKDROPS: [Backdrop; 8] = [
    Backdrop::Clay,
    Backdrop::Ochre,
    Backdrop::Sage,
    Backdrop::Jade,
    Backdrop::Teal,
    Backdrop::Slate,
    Backdrop::Plum,
    Backdrop::Rose,
];

impl PersonaSpec {
    /// A pleasant random persona: the same `seed` always gives the same one. People take a skin
    /// tone and mostly natural hair; animals and blobs take a fur hue, and their disc is never
    /// within 90 degrees of it, so the character always stands off its ground.
    pub fn from_seed(seed: u64) -> Self {
        let mut stream = Stream(seed);
        let creature = stream.weighted(&[
            (Creature::Person, 4),
            (Creature::Bear, 2),
            (Creature::Cat, 2),
            (Creature::Bunny, 2),
            (Creature::Blob, 2),
        ]);
        let head = stream.pick(&[
            HeadShape::Round,
            HeadShape::Soft,
            HeadShape::Tall,
            HeadShape::Wide,
        ]);
        let tone = match creature {
            Creature::Person => stream.pick(&SKIN),
            Creature::Bear | Creature::Cat | Creature::Bunny | Creature::Blob => stream.pick(&FUR),
        };
        let top = top_for(creature, &mut stream);
        let hair = stream.weighted(&[
            (HairTone::Ink, 3),
            (HairTone::Cocoa, 3),
            (HairTone::Auburn, 2),
            (HairTone::Honey, 2),
            (HairTone::Silver, 1),
            (HairTone::Rose, 1),
            (HairTone::Teal, 1),
            (HairTone::Plum, 1),
        ]);
        let backdrop = backdrop_for(tone, &mut stream);
        PersonaSpec {
            creature,
            head,
            tone,
            eyes: stream.pick(&[Eyes::Dot, Eyes::Oval, Eyes::Shine, Eyes::Wide]),
            brows: stream.weighted(&[(Brows::Hidden, 2), (Brows::Soft, 2), (Brows::Straight, 1)]),
            mouth: stream.pick(&[Mouth::Smile, Mouth::Cat, Mouth::Small, Mouth::Grin]),
            top,
            hair,
            cheeks: stream.weighted(&[(Cheeks::Blush, 3), (Cheeks::Plain, 1)]),
            accessory: stream.weighted(&[
                (Accessory::Plain, 5),
                (Accessory::Glasses, 2),
                (Accessory::Freckles, 2),
                (Accessory::Bow, 2),
            ]),
            backdrop,
            seed: PersonaSeed(seed),
        }
    }

    /// The gaps between blinks after a wake, each between [`BLINK_MIN`] and [`BLINK_MAX`],
    /// the same sequence for the same seed.
    pub fn blink_gaps(&self) -> impl Iterator<Item = Duration> {
        let mut stream = Stream(self.seed.0 ^ 0xB11C_B11C_B11C_B11C);
        let spread = u64::try_from((BLINK_MAX - BLINK_MIN).as_millis()).unwrap_or(3000);
        std::iter::repeat_with(move || {
            BLINK_MIN + Duration::from_millis(stream.next() % (spread + 1))
        })
    }
}

/// A top that suits `creature`: people take hair (never bare: a bald head read as unfinished
/// in the 48-seed grid), animals a tuft or nothing (hair on an animal read as a wig), a blob its
/// sprout.
fn top_for(creature: Creature, stream: &mut Stream) -> Top {
    match creature {
        Creature::Person => stream.weighted(&[
            (Top::Crop, 2),
            (Top::Fringe, 3),
            (Top::Bob, 3),
            (Top::Bun, 2),
            (Top::Curly, 2),
        ]),
        Creature::Blob => Top::Tuft,
        Creature::Bear | Creature::Cat | Creature::Bunny => {
            stream.weighted(&[(Top::Bare, 3), (Top::Tuft, 2)])
        }
    }
}

/// A disc for `tone`: any hue behind skin; behind fur, a hue at least 90 degrees away.
fn backdrop_for(tone: Tone, stream: &mut Stream) -> Backdrop {
    let Some(fur) = super::palette::fur_hue(tone) else {
        return stream.pick(&BACKDROPS);
    };
    let far: Vec<Backdrop> = BACKDROPS
        .into_iter()
        .filter(|backdrop| hue_gap(super::palette::backdrop_hue(*backdrop), fur) >= 90.0)
        .collect();
    stream.pick(&far)
}

/// The angle between two hues, 0 to 180.
fn hue_gap(one: f64, other: f64) -> f64 {
    let gap = (one - other).rem_euclid(360.0);
    gap.min(360.0 - gap)
}

#[cfg(test)]
mod tests {
    use super::{BLINK_MAX, BLINK_MIN, hue_gap};
    use crate::components::persona::spec::PersonaSpec;

    #[test]
    fn hue_gaps_wrap() {
        const CASES: &[(f64, f64, f64)] =
            &[(40.0, 355.0, 45.0), (0.0, 180.0, 180.0), (85.0, 85.0, 0.0)];
        for &(one, other, want) in CASES {
            assert!((hue_gap(one, other) - want).abs() < 1e-9, "{one} {other}");
        }
    }

    #[test]
    fn blink_gaps_stay_in_the_window() {
        for seed in 0..64 {
            let spec = PersonaSpec::from_seed(seed);
            for gap in spec.blink_gaps().take(16) {
                assert!((BLINK_MIN..=BLINK_MAX).contains(&gap), "{seed}: {gap:?}");
            }
        }
    }
}
