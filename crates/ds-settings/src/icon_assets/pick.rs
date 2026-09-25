//! Which file draws an app at a size and a style (design/08-ICONS.md 2.11).

use super::lookup::{Presence, a_file, apps_dir};
use crate::units::Px;
use ds::icon::IconStyle;
use std::path::{Path, PathBuf};

/// The sizes every app ships in every style, each drawn directly (design/08 2.11): the
/// freedesktop sizes, their `@2` up to 256@2, and the 1.5x sizes 36, 48, 72, 96. `tools/icons`
/// checks its `SHIP_PX` against this list.
pub const APP_ICON_PX: [u16; 13] = [16, 22, 24, 32, 36, 44, 48, 64, 72, 96, 128, 256, 512];

/// An app's directory name under the apps directory (`mail`, `files`): lower-case ASCII
/// letters, digits, `-` and `_`, so it can never climb out of the directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppIconName(String);

impl AppIconName {
    /// `None` for an empty name or one with any other character (`/`, `.`, a capital).
    pub fn parse(name: &str) -> Option<AppIconName> {
        let allowed =
            |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
        (!name.is_empty() && name.chars().all(allowed)).then(|| AppIconName(name.to_owned()))
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The sizes to try for a `size` px icon, in order: the shipped sizes at or above it,
/// smallest first (a larger file scales down cleanly), then the ones below it, largest first
/// (a request above 512 still gets the 512).
pub fn sizes_to_try(size: Px) -> Vec<u16> {
    let larger = APP_ICON_PX.into_iter().filter(|px| *px >= size.0);
    let smaller = APP_ICON_PX.into_iter().rev().filter(|px| *px < size.0);
    larger.chain(smaller).collect()
}

/// The style's subdirectory: none for Colour, `muted`, `monochrome` (a neutral grey set that
/// the caller then re-colours with `ds::icon::retint`).
fn style_dir(style: IconStyle) -> Option<&'static str> {
    match style {
        IconStyle::Colour => None,
        IconStyle::Muted => Some("muted"),
        IconStyle::Monochrome => Some("monochrome"),
    }
}

/// The first of `app`'s files for `size` and `style` under `dir` that `probe` finds.
pub fn find_app_icon(
    dir: &Path,
    app: &AppIconName,
    size: Px,
    style: IconStyle,
    probe: impl Fn(&Path) -> Presence,
) -> Option<PathBuf> {
    let base = dir.join(app.as_str());
    let base = style_dir(style).map_or(base.clone(), |sub| base.join(sub));
    sizes_to_try(size)
        .into_iter()
        .map(|px| base.join(format!("{px}.png")))
        .find(|path| probe(path) == Presence::Present)
}

/// The file that draws `app` at `size` physical pixels (the icon's size times the output
/// scale) in `style`: `<apps>/<app>/<px>.png`, `<app>/muted/<px>.png` or
/// `<app>/monochrome/<px>.png`, the exact size or the nearest shipped one above it, under the
/// directory [`apps_dir`] finds. `None` when there is no set, no such app, or a name that
/// [`AppIconName::parse`] refuses.
pub fn app_icon_path(app: &str, size: Px, style: IconStyle) -> Option<PathBuf> {
    let app = AppIconName::parse(app)?;
    find_app_icon(&apps_dir()?, &app, size, style, a_file)
}

#[cfg(test)]
mod tests {
    use super::{APP_ICON_PX, AppIconName, find_app_icon, sizes_to_try};
    use crate::icon_assets::lookup::{Presence, a_file};
    use crate::units::Px;
    use ds::icon::IconStyle;
    use std::path::{Path, PathBuf};

    #[test]
    fn the_exact_size_then_the_nearest_larger_then_the_largest_smaller() {
        let cases: &[(u16, &[u16])] = &[
            (48, &[48, 64, 72, 96, 128, 256, 512, 44, 36, 32, 24, 22, 16]),
            (50, &[64, 72, 96, 128, 256, 512, 48, 44, 36, 32, 24, 22, 16]),
            (1, &APP_ICON_PX),
            (
                512,
                &[512, 256, 128, 96, 72, 64, 48, 44, 36, 32, 24, 22, 16],
            ),
            (
                1024,
                &[512, 256, 128, 96, 72, 64, 48, 44, 36, 32, 24, 22, 16],
            ),
        ];
        for (size, want) in cases {
            assert_eq!(sizes_to_try(Px(*size)), *want, "{size}");
        }
    }

    #[test]
    fn each_style_has_its_directory_and_a_missing_size_takes_the_next_one_up() {
        let dir = Path::new("/icons");
        let present = [
            "/icons/mail/48.png",
            "/icons/mail/muted/64.png",
            "/icons/mail/monochrome/48.png",
            "/icons/mail/monochrome/32.png",
        ];
        let probe = |path: &Path| match present.iter().any(|p| Path::new(p) == path) {
            true => Presence::Present,
            false => Presence::Absent,
        };
        let mail = AppIconName::parse("mail").expect("a name");
        let cases: &[(u16, IconStyle, Option<&str>)] = &[
            (48, IconStyle::Colour, Some("/icons/mail/48.png")),
            (48, IconStyle::Muted, Some("/icons/mail/muted/64.png")),
            (
                32,
                IconStyle::Monochrome,
                Some("/icons/mail/monochrome/32.png"),
            ),
            (
                36,
                IconStyle::Monochrome,
                Some("/icons/mail/monochrome/48.png"),
            ),
            (
                96,
                IconStyle::Monochrome,
                Some("/icons/mail/monochrome/48.png"),
            ),
        ];
        for (size, style, want) in cases {
            assert_eq!(
                find_app_icon(dir, &mail, Px(*size), *style, probe),
                want.map(PathBuf::from),
                "{size} {style:?}"
            );
        }
        let files = AppIconName::parse("files").expect("a name");
        assert_eq!(
            find_app_icon(dir, &files, Px(48), IconStyle::Colour, probe),
            None
        );
    }

    #[test]
    fn a_name_cannot_leave_the_directory() {
        for bad in ["", "..", "../mail", "a/b", "Mail", "mail.png"] {
            assert_eq!(AppIconName::parse(bad), None, "{bad:?}");
        }
        assert!(AppIconName::parse("mail-2_x").is_some());
    }

    /// The repository's own set answers every app, size and style it ships.
    #[test]
    fn the_shipped_set_has_every_file() {
        let dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/icons/apps"
        ));
        for app in ["mail", "files", "terminal", "notes", "photos"] {
            let name = AppIconName::parse(app).expect("a name");
            for style in [IconStyle::Colour, IconStyle::Muted, IconStyle::Monochrome] {
                for px in APP_ICON_PX {
                    let got = find_app_icon(dir, &name, Px(px), style, a_file);
                    assert!(
                        got.as_ref()
                            .is_some_and(|path| path.ends_with(format!("{px}.png"))),
                        "{app} {style:?} {px}: {got:?}"
                    );
                }
            }
        }
    }
}
