//! `hot_corners.*` as a file: the list-of-bindings form, the older one-word form, and what the
//! lenient reader drops.

use crate::support::Scratch;

use ds_settings::schema::{
    CornerAction, CornerBinding, CornerDo, CornerKey, HotCornerSettings, OpenItem,
};
use ds_settings::{FileName, Format, Loaded, SettingsDoc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
struct Sill {
    hot_corners: HotCornerSettings,
}

impl SettingsDoc for Sill {
    const FILE: FileName = FileName("settings.toml");
    const FORMAT: Format = Format::Toml;
}

fn bind(keys: &[CornerKey], action: CornerDo) -> CornerBinding {
    CornerBinding {
        keys: keys.iter().copied().collect(),
        action,
    }
}

fn loaded(scratch: &Scratch, text: &str) -> Loaded<Sill> {
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join("settings.toml"), text).unwrap_or_else(|e| panic!("{e}"));
    scratch.store().load::<Sill>()
}

fn saved(scratch: &Scratch, doc: &Sill) -> String {
    scratch.store().save(doc).unwrap_or_else(|e| panic!("{e}"));
    std::fs::read_to_string(scratch.app_dir().join("settings.toml"))
        .unwrap_or_else(|e| panic!("{e}"))
}

const SPEC_EXAMPLE: &str = r#"
[hot_corners]
dwell_ms = 150

[[hot_corners.top_left]]
action = "workspaces"

[[hot_corners.top_left]]
keys = ["shift", "option"]
action = "lock"

[[hot_corners.bottom_right]]
keys = ["command"]
action = "app_action"
app = "org.quire.Mail"
intent = "compose"

[[hot_corners.bottom_left]]
action = "open"
path = "~/Documents"

[[hot_corners.top_right]]
action = "command"
command = "notify-send hi"
"#;

#[test]
fn the_list_form_loads_round_trips_and_writes_keys_in_a_stable_order() {
    let scratch = Scratch::new();
    let read = loaded(&scratch, SPEC_EXAMPLE);
    assert!(
        read.unknown.is_empty() && read.invalid.is_empty(),
        "{read:?}"
    );
    let corners = &read.value.hot_corners;
    assert_eq!(
        corners.top_left.0,
        [
            bind(&[], CornerDo::Desktop(CornerAction::Workspaces)),
            bind(
                &[CornerKey::Option, CornerKey::Shift],
                CornerDo::Desktop(CornerAction::Lock)
            ),
        ]
    );
    assert_eq!(
        corners.bottom_right.0,
        [bind(
            &[CornerKey::Command],
            CornerDo::AppAction {
                app: "org.quire.Mail".to_owned(),
                intent: "compose".to_owned()
            }
        )]
    );
    assert_eq!(
        corners.bottom_left.0,
        [bind(
            &[],
            CornerDo::Open(OpenItem::Path("~/Documents".to_owned()))
        )]
    );
    assert_eq!(
        corners.top_right.0,
        [bind(&[], CornerDo::Command("notify-send hi".to_owned()))]
    );
    let text = saved(&scratch, &read.value);
    assert!(text.contains(r#"keys = ["option", "shift"]"#), "{text}");
    assert_eq!(scratch.store().load::<Sill>(), Loaded::clean(read.value));
}

#[test]
fn each_older_form_loads_and_saving_writes_only_the_new_one() {
    let scratch = Scratch::new();
    let read = loaded(
        &scratch,
        r#"
[hot_corners]
top_left = "none"
top_right = "lock"
bottom_left = "command"
bottom_left_command = "foot"
bottom_right = "command"
bottom_right_command = ""
"#,
    );
    assert!(
        read.unknown.is_empty() && read.invalid.is_empty(),
        "{read:?}"
    );
    let corners = &read.value.hot_corners;
    assert!(corners.top_left.0.is_empty());
    assert_eq!(
        corners.top_right.0,
        [bind(&[], CornerDo::Desktop(CornerAction::Lock))]
    );
    assert_eq!(
        corners.bottom_left.0,
        [bind(&[], CornerDo::Command("foot".to_owned()))]
    );
    assert!(corners.bottom_right.0.is_empty());
    let text = saved(&scratch, &read.value);
    assert!(!text.contains("_command"), "{text}");
    assert!(!text.contains(r#"top_right = "lock""#), "{text}");
    assert!(text.contains(r#"command = "foot""#), "{text}");
    assert_eq!(scratch.store().load::<Sill>(), Loaded::clean(read.value));
}

#[test]
fn a_bad_binding_is_dropped_and_reported_and_the_others_are_kept() {
    let scratch = Scratch::new();
    let read = loaded(
        &scratch,
        r#"
[[hot_corners.top_left]]
action = "workspaces"

[[hot_corners.top_left]]
action = "teleport"

[[hot_corners.top_left]]
keys = ["hyper"]
action = "lock"

[[hot_corners.top_left]]
keys = ["option"]
action = "lock"

[[hot_corners.top_right]]
action = "launcher"
"#,
    );
    let corners = &read.value.hot_corners;
    assert_eq!(
        corners.top_left.0,
        [
            bind(&[], CornerDo::Desktop(CornerAction::Workspaces)),
            bind(&[CornerKey::Option], CornerDo::Desktop(CornerAction::Lock)),
        ]
    );
    assert_eq!(corners.top_right.0.len(), 1);
    let reported: Vec<&str> = read.invalid.iter().map(|key| key.path.as_str()).collect();
    assert_eq!(reported, ["hot_corners.top_left"]);
}

#[test]
fn a_second_binding_for_the_same_keys_is_dropped_and_reported() {
    let scratch = Scratch::new();
    let read = loaded(
        &scratch,
        r#"
[[hot_corners.top_left]]
keys = ["option", "command"]
action = "lock"

[[hot_corners.top_left]]
keys = ["command", "option"]
action = "launcher"
"#,
    );
    assert_eq!(
        read.value.hot_corners.top_left.0,
        [bind(
            &[CornerKey::Command, CornerKey::Option],
            CornerDo::Desktop(CornerAction::Lock)
        )]
    );
    assert_eq!(read.invalid.len(), 1, "{:?}", read.invalid);
}

#[test]
fn the_four_corner_rows_are_one_custom_control_and_the_timing_rows_stay_advanced() {
    use ds_settings::schema::{Control, Exposure, SettingsSchema};
    let schema = HotCornerSettings::schema();
    let rows: Vec<(&str, Control, Exposure)> = schema
        .key
        .iter()
        .map(|key| (key.path.0.as_str(), key.control(), key.exposure))
        .collect();
    for (path, control, exposure) in rows {
        let corner = ["top_left", "top_right", "bottom_left", "bottom_right"]
            .iter()
            .any(|name| path == format!("hot_corners.{name}"));
        if corner {
            assert_eq!(
                (control, exposure),
                (Control::Corners, Exposure::Basic),
                "{path}"
            );
        } else {
            assert_eq!(exposure, Exposure::Advanced, "{path}");
        }
    }
}
