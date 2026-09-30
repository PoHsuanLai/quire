//! `quire/appearance.toml` through a `Store` on a scratch directory: the round trip, the first
//! run, a damaged file, a bad value and a key nobody reads.

mod support;

use ds_style::appearance::accent::Accent;

use ds_style::appearance::motion::Motion;

use ds_settings::{AppearanceFile, IconDarkVariant, Loaded, Percent, PlateGlyphPolicy};
use ds_style::appearance::theme::Theme;
use std::path::Path;
use support::Scratch;

const FILE: &str = "appearance.toml";

fn entries(dir: &Path) -> Vec<String> {
    let mut names = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("{e}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn with(edit: impl FnOnce(&mut AppearanceFile)) -> AppearanceFile {
    let mut file = AppearanceFile::default();
    edit(&mut file);
    file
}

fn write(scratch: &Scratch, text: &[u8]) {
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join(FILE), text).unwrap_or_else(|e| panic!("{e}"));
}

fn unknown(loaded: &Loaded<AppearanceFile>) -> Vec<&str> {
    loaded.unknown.iter().map(|key| key.0.as_str()).collect()
}

fn invalid(loaded: &Loaded<AppearanceFile>) -> Vec<&str> {
    loaded.invalid.iter().map(|key| key.path.as_str()).collect()
}

#[test]
fn a_file_round_trips() {
    let scratch = Scratch::new();
    let store = scratch.store();
    let cases = [
        AppearanceFile::default(),
        with(|f| {
            f.appearance.theme = Theme::Dark;
            f.appearance.accent = Accent::Violet;
            f.appearance.motion_level = Motion::Reduced;
        }),
        with(|f| {
            f.appearance.material_tint_alpha = Percent(64);
            f.icons.plate_glyph_colour_policy = PlateGlyphPolicy::ForceInk;
            f.icons.dark_mode_variant = IconDarkVariant::Adaptive;
        }),
    ];
    for file in cases {
        store
            .save(&file)
            .unwrap_or_else(|e| panic!("{file:?}: {e}"));
        assert_eq!(
            store.load::<AppearanceFile>(),
            Loaded::clean(file.clone()),
            "{file:?}"
        );
        // The rename is the whole of the write: a temp file left beside the real one is a crash
        // that did not finish, and this directory had no other files.
        assert_eq!(entries(&scratch.app_dir()), [FILE], "{file:?}");
    }
}

#[test]
fn a_missing_file_is_the_first_run() {
    let scratch = Scratch::new();
    assert_eq!(
        scratch.store().load::<AppearanceFile>(),
        Loaded::clean(AppearanceFile::default())
    );
}

#[test]
fn a_damaged_file_is_the_defaults_and_says_so() {
    const CASES: &[(&str, &[u8], &[&str])] = &[
        ("empty", b"", &[]),
        ("prose", b"not toml [[[ = =", &[""]),
        ("binary", &[0xff, 0xfe, b'['], &[]),
    ];
    for &(name, bytes, want_invalid) in CASES {
        let scratch = Scratch::new();
        write(&scratch, bytes);
        let loaded = scratch.store().load::<AppearanceFile>();
        assert_eq!(loaded.value, AppearanceFile::default(), "{name}");
        assert_eq!(invalid(&loaded), want_invalid, "{name}");
    }
}

#[test]
fn a_bad_value_costs_only_its_own_field() {
    // Every row also sets a field to something other than its default, so a loader that threw
    // the whole file away on the bad value would fail it.
    struct Case {
        name: &'static str,
        text: &'static str,
        want: AppearanceFile,
        invalid: &'static [&'static str],
    }
    let cases = [
        Case {
            name: "unknown theme keeps the motion",
            text: "[appearance]\ntheme = \"sepia\"\nmotion_level = \"reduced\"\n",
            want: with(|f| f.appearance.motion_level = Motion::Reduced),
            invalid: &["appearance.theme"],
        },
        Case {
            name: "unknown motion keeps the theme",
            text: "[appearance]\ntheme = \"dark\"\nmotion_level = \"wild\"\n",
            want: with(|f| f.appearance.theme = Theme::Dark),
            invalid: &["appearance.motion_level"],
        },
        Case {
            name: "a number where a word belongs keeps the accent",
            text: "[appearance]\nlook = 7\naccent = \"green\"\n",
            want: with(|f| f.appearance.accent = Accent::Green),
            invalid: &["appearance.look"],
        },
        Case {
            name: "an out-of-range percent is clamped, not refused",
            text: "[appearance]\nmaterial_tint_alpha = 255\n",
            want: with(|f| f.appearance.material_tint_alpha = Percent(100)),
            invalid: &[],
        },
        Case {
            name: "a negative percent is the default and keeps the theme",
            text: "[appearance]\nmaterial_tint_alpha = -4\ntheme = \"light\"\n",
            want: with(|f| f.appearance.theme = Theme::Light),
            invalid: &["appearance.material_tint_alpha"],
        },
        Case {
            name: "a missing table is that table's defaults",
            text: "[icons]\nplate_inset_percent = 60\n",
            want: with(|f| f.icons.plate_inset_percent = Percent(60)),
            invalid: &[],
        },
        Case {
            name: "a table that is not a table is its defaults",
            text: "appearance = 3\n[icons]\ndark_mode_variant = \"adaptive\"\n",
            want: with(|f| f.icons.dark_mode_variant = IconDarkVariant::Adaptive),
            invalid: &["appearance"],
        },
    ];
    for case in cases {
        let scratch = Scratch::new();
        write(&scratch, case.text.as_bytes());
        let loaded = scratch.store().load::<AppearanceFile>();
        assert_eq!(loaded.value, case.want, "{}: {}", case.name, case.text);
        assert_eq!(invalid(&loaded), case.invalid, "{}", case.name);
        assert!(
            loaded.unknown.is_empty(),
            "{}: {:?}",
            case.name,
            loaded.unknown
        );
    }
}

#[test]
fn a_key_nobody_reads_is_reported_and_gone_after_the_next_save() {
    let scratch = Scratch::new();
    let store = scratch.store();
    write(
        &scratch,
        b"version = 1\nfuture = \"dropped\"\n[appearance]\ntheme = \"dark\"\npuppy = true\n\
          [icons]\nsparkle = 3\n[later]\nanswer = 42\n",
    );
    let read = store.load::<AppearanceFile>();
    assert_eq!(read.value.appearance.theme, Theme::Dark);
    assert_eq!(
        unknown(&read),
        ["appearance.puppy", "future", "icons.sparkle", "later"]
    );
    assert!(read.invalid.is_empty(), "{:?}", read.invalid);

    store.save(&read.value).unwrap_or_else(|e| panic!("{e}"));
    let written =
        std::fs::read_to_string(scratch.app_dir().join(FILE)).unwrap_or_else(|e| panic!("{e}"));
    for gone in ["puppy", "future", "sparkle", "later", "answer"] {
        assert!(
            !written.contains(gone),
            "{gone} survived the save:\n{written}"
        );
    }
    let again = store.load::<AppearanceFile>();
    assert_eq!(
        again,
        Loaded::clean(read.value),
        "a saved file has nothing left to report"
    );
}
