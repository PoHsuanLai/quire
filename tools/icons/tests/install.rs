//! `icons install` run as a user runs it (sill FINDINGS Q71): the repository's set lands under
//! `--to`, every app in every style and size, and `ds_settings`'s lookup finds a file there.

use ds::icon::IconStyle;
use ds_settings::Px;
use ds_settings::icon_assets::{APP_ICON_PX, AppIconName, Presence, find_app_icon};
use std::path::Path;
use std::process::Command;

#[test]
fn install_copies_the_set_where_the_lookup_finds_it() {
    let to = Path::new(env!("CARGO_TARGET_TMPDIR")).join("installed-icons/quire/icons/apps");
    std::fs::remove_dir_all(&to).ok();
    let status = Command::new(env!("CARGO_BIN_EXE_icons"))
        .args(["install", "--to"])
        .arg(&to)
        .status()
        .expect("icons runs");
    assert!(status.success());
    let present = |path: &Path| match path.is_file() {
        true => Presence::Present,
        false => Presence::Absent,
    };
    for app in ["mail", "files", "terminal", "notes", "photos"] {
        let name = AppIconName::parse(app).expect("a name");
        for style in [IconStyle::Colour, IconStyle::Muted, IconStyle::Monochrome] {
            for px in APP_ICON_PX {
                let got = find_app_icon(&to, &name, Px(px), style, present);
                assert!(
                    got.is_some_and(|path| path.ends_with(format!("{px}.png"))),
                    "{app} {style:?} {px}"
                );
            }
        }
    }
}

/// The tool ships the sizes the lookup expects.
#[test]
fn the_shipped_sizes_are_the_lookups() {
    let lookup: Vec<u32> = APP_ICON_PX.into_iter().map(u32::from).collect();
    assert_eq!(lookup, icons::SHIP_PX);
}
