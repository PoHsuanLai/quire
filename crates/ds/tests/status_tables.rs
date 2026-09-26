//! design/26 D1: the status glyphs' moment tables as data, and the quantising each compares on
//! (R2). Every (from, to) row of the catalogue's section 5.1 is here.

use ds::detail::{EventStamp, Moment, first_table, moment_table};
use ds::{
    BatteryPower, BatteryState, BluetoothState, Fraction, LowAt, StatusState, VolumeState,
    VolumeWaves, WifiBars, WifiReach, WifiState,
};

const ONE: EventStamp = EventStamp(1);
const TWO: EventStamp = EventStamp(2);

fn joined(bars: WifiBars) -> WifiState {
    WifiState::Joined {
        bars,
        reach: WifiReach::Internet,
    }
}

fn no_internet(bars: WifiBars) -> WifiState {
    WifiState::Joined {
        bars,
        reach: WifiReach::NoInternet,
    }
}

#[test]
fn the_wifi_table() {
    use WifiBars::{One, Three, Two};
    use WifiState::{Failed, Idle, Joining, Off};
    moment_table(&[
        (joined(Two), joined(Two), Moment::Rest),
        // G1: joining searches, from anywhere, and a new join is a new operation.
        (Off, Joining(ONE), Moment::Pending),
        (Idle, Joining(ONE), Moment::Pending),
        (joined(Three), Joining(ONE), Moment::Pending),
        (Failed(ONE), Joining(TWO), Moment::Pending),
        (Joining(ONE), Joining(TWO), Moment::Pending),
        (Joining(ONE), Joining(ONE), Moment::Rest),
        // G2: a join that lands fills once.
        (Joining(ONE), joined(Two), Moment::Success),
        (Joining(ONE), no_internet(One), Moment::Success),
        // G3: strength changes cross-fade; a join nobody watched (auto-join) is a change.
        (joined(Three), joined(One), Moment::Change),
        (Idle, joined(Three), Moment::Change),
        (Off, joined(Three), Moment::Change),
        // G4: the badge.
        (joined(Two), no_internet(Two), Moment::Change),
        (no_internet(Two), joined(Two), Moment::Change),
        // G5: a failed join shakes, once per stamp.
        (Joining(ONE), Failed(ONE), Moment::Failure),
        (Failed(ONE), Failed(TWO), Moment::Failure),
        (Failed(ONE), Failed(ONE), Moment::Rest),
        // The radio.
        (joined(Two), Off, Moment::Unavailable),
        (Joining(ONE), Off, Moment::Unavailable),
        (Off, Idle, Moment::Change),
        (Joining(ONE), Idle, Moment::Change),
    ]);
    first_table(&[
        (Joining(ONE), Moment::Pending),
        (joined(Two), Moment::Rest),
        (Off, Moment::Rest),
        (Idle, Moment::Rest),
        (Failed(ONE), Moment::Rest),
    ]);
}

#[test]
fn wifi_bars_follow_strength_by_thirds() {
    const CASES: &[(u16, WifiBars)] = &[
        (0, WifiBars::One),
        (333, WifiBars::One),
        (334, WifiBars::Two),
        (666, WifiBars::Two),
        (667, WifiBars::Three),
        (1000, WifiBars::Three),
    ];
    for &(strength, want) in CASES {
        assert_eq!(WifiBars::of(Fraction(strength)), want, "{strength}");
    }
    // 67 then 68 % is the same three bars: no moment (R2).
    moment_table(&[(
        joined(WifiBars::of(Fraction(670))),
        joined(WifiBars::of(Fraction(680))),
        Moment::Rest,
    )]);
}

fn battery(level: u16, power: BatteryPower) -> BatteryState {
    BatteryState {
        level: Fraction(level),
        power,
        low_at: LowAt::default(),
    }
}

#[test]
fn the_battery_table() {
    use BatteryPower::{Battery, Charging, Held};
    moment_table(&[
        // R2: a change finer than a drawn step is none.
        (battery(804, Battery), battery(801, Battery), Moment::Rest),
        // G8: a step of the fill.
        (battery(800, Battery), battery(300, Battery), Moment::Change),
        // G9: the bolt and the plug.
        (
            battery(600, Battery),
            battery(600, Charging),
            Moment::Change,
        ),
        (battery(1000, Charging), battery(1000, Held), Moment::Change),
        (battery(1000, Held), battery(1000, Battery), Moment::Change),
        // G10: crossing the threshold is a change of tone even inside one step.
        (battery(201, Battery), battery(195, Battery), Moment::Change),
        // Charging is never low: no tone change there.
        (battery(201, Charging), battery(195, Charging), Moment::Rest),
    ]);
    first_table(&[(battery(500, Battery), Moment::Appear)]);
}

#[test]
fn the_low_threshold_is_the_callers() {
    let at = |level, low| BatteryState {
        level: Fraction(level),
        power: BatteryPower::Battery,
        low_at: LowAt(Fraction(low)),
    };
    moment_table(&[
        (at(170, 165), at(160, 165), Moment::Change),
        (at(170, 200), at(160, 200), Moment::Rest),
    ]);
    assert_eq!(LowAt::default(), LowAt(Fraction(200)));
}

#[test]
fn the_bluetooth_table() {
    use BluetoothState::{Connected, Connecting, Failed, Off, On};
    moment_table(&[
        (On, On, Moment::Rest),
        (Off, On, Moment::Change),
        (On, Off, Moment::Unavailable),
        (Connected, Off, Moment::Unavailable),
        (On, Connecting(ONE), Moment::Pending),
        (Connecting(ONE), Connecting(TWO), Moment::Pending),
        (Connecting(ONE), Connected, Moment::Success),
        (On, Connected, Moment::Change),
        (Connected, On, Moment::Change),
        (Connecting(ONE), Failed(ONE), Moment::Failure),
        (Failed(ONE), Failed(ONE), Moment::Rest),
        (Failed(ONE), Failed(TWO), Moment::Failure),
    ]);
    first_table(&[
        (Connecting(ONE), Moment::Pending),
        (Connected, Moment::Rest),
        (Off, Moment::Rest),
    ]);
}

#[test]
fn the_volume_table() {
    use VolumeState::{Heard, Muted, NoDevice};
    moment_table(&[
        (
            Heard(VolumeWaves::Two),
            Heard(VolumeWaves::Two),
            Moment::Rest,
        ),
        (
            Heard(VolumeWaves::Two),
            Heard(VolumeWaves::Three),
            Moment::Change,
        ),
        (Heard(VolumeWaves::Two), Muted, Moment::Change),
        (Muted, Heard(VolumeWaves::One), Moment::Change),
        (Heard(VolumeWaves::One), NoDevice, Moment::Unavailable),
        (NoDevice, Muted, Moment::Change),
    ]);
    first_table(&[
        (Muted, Moment::Rest),
        (Heard(VolumeWaves::Three), Moment::Rest),
    ]);
    // 50 then 51 % is the same two waves (R2).
    assert_eq!(
        VolumeWaves::of(Fraction(500)),
        VolumeWaves::of(Fraction(510))
    );
    assert_eq!(VolumeWaves::of(Fraction(0)), VolumeWaves::Zero);
    assert_eq!(VolumeWaves::of(Fraction(1000)), VolumeWaves::Three);
}

#[test]
fn every_state_has_words() {
    let states = [
        StatusState::Wifi(WifiState::Joining(ONE)),
        StatusState::Wifi(no_internet(WifiBars::Two)),
        StatusState::Battery(battery(804, BatteryPower::Charging)),
        StatusState::Bluetooth(BluetoothState::Off),
        StatusState::Volume(VolumeState::NoDevice),
    ];
    let words: Vec<String> = states.iter().map(|state| state.words()).collect();
    assert_eq!(
        words,
        [
            "Joining…",
            "No internet",
            "80%, charging",
            "Bluetooth off",
            "No output device"
        ]
    );
}
