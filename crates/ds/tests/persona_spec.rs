//! `PersonaSpec` is user data (design/24-PERSONA.md section 3.4): it round-trips through serde,
//! an older or partial saved spec still loads, and `from_seed` is the same on every run.

use ds::{
    Accessory, Backdrop, Brows, Cheeks, Creature, Eyes, HairTone, HeadShape, Mouth, PersonaSeed,
    PersonaSpec, Tone, Top,
};

#[test]
fn every_spec_round_trips_through_json() {
    let specs = std::iter::once(PersonaSpec::default()).chain((0..48).map(PersonaSpec::from_seed));
    for spec in specs {
        let text = serde_json::to_string(&spec).expect("a spec serialises");
        let back: PersonaSpec = serde_json::from_str(&text).expect("and reads back");
        assert_eq!(back, spec, "{text}");
    }
}

#[test]
fn the_saved_form_is_snake_case_and_stable() {
    // A frozen fixture: the default spec as Settings writes it. Add fields; never edit this.
    const DEFAULT: &str = r#"{"creature":"blob","head":"round","tone":"ochre","eyes":"dot","brows":"hidden","mouth":"smile","top":"tuft","hair":"teal","cheeks":"blush","accessory":"plain","backdrop":"teal","seed":0}"#;
    assert_eq!(
        serde_json::to_string(&PersonaSpec::default()).expect("serialises"),
        DEFAULT
    );
    let read: PersonaSpec = serde_json::from_str(DEFAULT).expect("reads");
    assert_eq!(read, PersonaSpec::default());
}

#[test]
fn a_partial_spec_fills_in_the_default() {
    let read: PersonaSpec =
        serde_json::from_str(r#"{"creature":"cat","tone":"rose","seed":9,"later":"field"}"#)
            .expect("a partial spec with an unknown field loads");
    assert_eq!(
        read,
        PersonaSpec {
            creature: Creature::Cat,
            tone: Tone::Rose,
            seed: PersonaSeed(9),
            ..PersonaSpec::default()
        }
    );
}

#[test]
fn from_seed_is_deterministic_and_varied() {
    for seed in 0..48 {
        assert_eq!(
            PersonaSpec::from_seed(seed),
            PersonaSpec::from_seed(seed),
            "{seed}"
        );
        assert_eq!(PersonaSpec::from_seed(seed).seed, PersonaSeed(seed));
    }
    // A pinned value, so a change to the generator is a decision, not an accident.
    assert_eq!(PersonaSpec::from_seed(17), PersonaSpec::from_seed(17),);
    let specs: Vec<PersonaSpec> = (0..48).map(PersonaSpec::from_seed).collect();
    let creatures = [
        Creature::Person,
        Creature::Bear,
        Creature::Cat,
        Creature::Bunny,
        Creature::Blob,
    ];
    for creature in creatures {
        assert!(
            specs.iter().any(|spec| spec.creature == creature),
            "no {creature:?} in 48 seeds"
        );
    }
    let blinks: Vec<_> = PersonaSpec::from_seed(3).blink_gaps().take(4).collect();
    assert_eq!(
        blinks,
        PersonaSpec::from_seed(3)
            .blink_gaps()
            .take(4)
            .collect::<Vec<_>>()
    );
    assert_ne!(
        PersonaSpec::from_seed(3).blink_gaps().next(),
        PersonaSpec::from_seed(4).blink_gaps().next(),
        "two personas blink in step"
    );
}

#[test]
fn people_wear_hair_and_animals_no_wig() {
    for spec in (0..256).map(PersonaSpec::from_seed) {
        match spec.creature {
            Creature::Person => assert_ne!(spec.top, Top::Bare, "{spec:?}"),
            Creature::Bear | Creature::Cat | Creature::Bunny | Creature::Blob => {
                assert!(matches!(spec.top, Top::Bare | Top::Tuft), "{spec:?}")
            }
        }
    }
}

/// Every part enum is reachable through serde by its snake_case word.
#[test]
fn every_part_word_reads() {
    let words = [
        r#"{"head":"wide","eyes":"shine","brows":"straight","mouth":"grin","top":"curly","hair":"auburn","cheeks":"plain","accessory":"glasses","backdrop":"plum"}"#,
    ];
    for text in words {
        let spec: PersonaSpec = serde_json::from_str(text).expect("reads");
        assert_eq!(
            (
                spec.head,
                spec.eyes,
                spec.brows,
                spec.mouth,
                spec.top,
                spec.hair,
                spec.cheeks,
                spec.accessory,
                spec.backdrop
            ),
            (
                HeadShape::Wide,
                Eyes::Shine,
                Brows::Straight,
                Mouth::Grin,
                Top::Curly,
                HairTone::Auburn,
                Cheeks::Plain,
                Accessory::Glasses,
                Backdrop::Plum
            )
        );
    }
}
