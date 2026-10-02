use super::model::{OrbPeriod, OrbSize};
use ds_core::word::Word;
use ds_style::tokens::timing::DurationToken;

#[test]
fn every_orb_size_has_its_own_width_and_they_grow() {
    const WIDTHS: &[(OrbSize, f32)] = &[
        (OrbSize::Inline, 14.0),
        (OrbSize::Bar, 16.0),
        (OrbSize::Field, 20.0),
        (OrbSize::Module, 40.0),
        (OrbSize::Hero, 192.0),
    ];
    assert_eq!(WIDTHS.len(), OrbSize::ALL.len());
    for &(size, width) in WIDTHS {
        assert_eq!(size.px().0, width, "{size:?}");
    }
    assert!(WIDTHS.windows(2).all(|pair| pair[0].1 < pair[1].1));
}

#[test]
fn each_period_is_timed_by_its_own_token() {
    const TOKENS: &[(OrbPeriod, DurationToken)] = &[
        (OrbPeriod::Listen, DurationToken::OrbListen),
        (OrbPeriod::Work, DurationToken::OrbWork),
        (OrbPeriod::Act, DurationToken::OrbAct),
    ];
    assert_eq!(TOKENS.len(), OrbPeriod::ALL.len());
    for &(period, token) in TOKENS {
        assert_eq!(period.token(), token, "{period:?}");
    }
}
