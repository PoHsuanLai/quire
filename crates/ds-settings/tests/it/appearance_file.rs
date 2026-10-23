//! `quire/appearance.toml` through a `Store` on a scratch directory: the round trip, the first
//! run, a damaged file, a bad value and a key nobody reads.

use crate::support;

use ds_style::appearance::accent::Accent;

use ds_style::appearance::motion::Motion;

use ds_settings::{AppearanceFile, IconDarkVariant, Loaded, Percent, PlateGlyphPolicy};
use ds_style::appearance::theme::Theme;
use ds_style::appearance::transparency::Transparency;
use ds_style::kit::CustomStyles;
use std::path::Path;
use support::Scratch;

const FILE: &str = "appearance.toml";
/// The file a save locks, beside the settings file.
const LOCK: &str = ".appearance.toml.lock";

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
            f.appearance.accent = Accent::Purple;
            f.appearance.motion_level = Motion::Reduced;
            f.appearance.transparency = Transparency::Reduced;
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
        // that did not finish. The lock's sidecar is the only other file.
        assert_eq!(entries(&scratch.app_dir()), [LOCK, FILE], "{file:?}");
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
        ("binary", &[0xff, 0xfe, b'['], &[""]),
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
        clamped: &'static [&'static str],
    }
    let cases = [
        Case {
            name: "unknown theme keeps the motion",
            text: "[appearance]\ntheme = \"sepia\"\nmotion_level = \"reduced\"\n",
            want: with(|f| f.appearance.motion_level = Motion::Reduced),
            invalid: &["appearance.theme"],
            clamped: &[],
        },
        Case {
            name: "unknown motion keeps the theme",
            text: "[appearance]\ntheme = \"dark\"\nmotion_level = \"wild\"\n",
            want: with(|f| f.appearance.theme = Theme::Dark),
            invalid: &["appearance.motion_level"],
            clamped: &[],
        },
        Case {
            name: "a number where a word belongs keeps the accent",
            text: "[appearance]\ntheme = 7\naccent = \"green\"\n",
            want: with(|f| f.appearance.accent = Accent::Green),
            invalid: &["appearance.theme"],
            clamped: &[],
        },
        Case {
            name: "the retired postmark accent falls back to blue and is reported",
            text: "[appearance]\naccent = \"postmark\"\ntheme = \"dark\"\n",
            want: with(|f| f.appearance.theme = Theme::Dark),
            invalid: &["appearance.accent"],
            clamped: &[],
        },
        Case {
            name: "the retired amber accent falls back to blue and is reported",
            text: "[appearance]\naccent = \"amber\"\n",
            want: AppearanceFile::default(),
            invalid: &["appearance.accent"],
            clamped: &[],
        },
        Case {
            name: "the retired violet accent falls back to blue and is reported",
            text: "[appearance]\naccent = \"violet\"\n",
            want: AppearanceFile::default(),
            invalid: &["appearance.accent"],
            clamped: &[],
        },
        Case {
            name: "a file from before the field reads transparency as System",
            text: "[appearance]\ntheme = \"dark\"\nmotion_level = \"reduced\"\n",
            want: with(|f| {
                f.appearance.theme = Theme::Dark;
                f.appearance.motion_level = Motion::Reduced;
            }),
            invalid: &[],
            clamped: &[],
        },
        Case {
            name: "reduced transparency is read",
            text: "[appearance]\ntransparency = \"reduced\"\n",
            want: with(|f| f.appearance.transparency = Transparency::Reduced),
            invalid: &[],
            clamped: &[],
        },
        Case {
            name: "an unknown transparency costs only itself",
            text: "[appearance]\ntransparency = \"clear\"\ntheme = \"dark\"\n",
            want: with(|f| f.appearance.theme = Theme::Dark),
            invalid: &["appearance.transparency"],
            clamped: &[],
        },
        Case {
            name: "use my styles off is read",
            text: "[appearance]\ncustom_styles = \"off\"\n",
            want: with(|f| f.appearance.custom_styles = CustomStyles::Off),
            invalid: &[],
            clamped: &[],
        },
        Case {
            name: "an unknown custom_styles costs only itself",
            text: "[appearance]\ncustom_styles = \"maybe\"\ntheme = \"dark\"\n",
            want: with(|f| f.appearance.theme = Theme::Dark),
            invalid: &["appearance.custom_styles"],
            clamped: &[],
        },
        Case {
            name: "an out-of-range percent is clamped and reported, not refused",
            text: "[appearance]\nmaterial_tint_alpha = 255\n",
            want: with(|f| f.appearance.material_tint_alpha = Percent(100)),
            invalid: &[],
            clamped: &["appearance.material_tint_alpha"],
        },
        Case {
            name: "a negative percent is held to zero and keeps the theme",
            text: "[appearance]\nmaterial_tint_alpha = -4\ntheme = \"light\"\n",
            want: with(|f| {
                f.appearance.theme = Theme::Light;
                f.appearance.material_tint_alpha = Percent(0);
            }),
            invalid: &[],
            clamped: &["appearance.material_tint_alpha"],
        },
        Case {
            name: "a missing table is that table's defaults",
            text: "[icons]\nplate_inset_percent = 60\n",
            want: with(|f| f.icons.plate_inset_percent = Percent(60)),
            invalid: &[],
            clamped: &[],
        },
        Case {
            name: "a table that is not a table is its defaults",
            text: "appearance = 3\n[icons]\ndark_mode_variant = \"adaptive\"\n",
            want: with(|f| f.icons.dark_mode_variant = IconDarkVariant::Adaptive),
            invalid: &["appearance"],
            clamped: &[],
        },
    ];
    for case in cases {
        let scratch = Scratch::new();
        write(&scratch, case.text.as_bytes());
        let loaded = scratch.store().load::<AppearanceFile>();
        assert_eq!(loaded.value, case.want, "{}: {}", case.name, case.text);
        assert_eq!(invalid(&loaded), case.invalid, "{}", case.name);
        let clamped: Vec<&str> = loaded.clamped.iter().map(|key| key.path.as_str()).collect();
        assert_eq!(clamped, case.clamped, "{}", case.name);
        assert!(
            loaded.unknown.is_empty(),
            "{}: {:?}",
            case.name,
            loaded.unknown
        );
    }
}

#[test]
fn a_key_nobody_reads_and_a_comment_survive_the_next_save() {
    const TEXT: &str = "# mine, not the program's\nversion = 1\nfuture = \"kept\"\n\
        [appearance]\ntheme = \"dark\"   # night owl\npuppy = true\nlook = \"arc\"\n\
        [icons]\nsparkle = 3\n[later]\nanswer = 42\n";
    let scratch = Scratch::new();
    let store = scratch.store();
    write(&scratch, TEXT.as_bytes());
    let read = store.load::<AppearanceFile>();
    assert_eq!(read.value.appearance.theme, Theme::Dark);
    let reported = [
        "appearance.look",
        "appearance.puppy",
        "future",
        "icons.sparkle",
        "later",
    ];
    assert_eq!(unknown(&read), reported);
    assert!(read.invalid.is_empty(), "{:?}", read.invalid);

    // A known key changes; everything else in the file stays as the person left it.
    let changed = with(|f| {
        f.appearance = read.value.appearance.clone();
        f.appearance.theme = Theme::Light;
    });
    let saved = store.save(&changed).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(saved.kept_at, None);
    let written =
        std::fs::read_to_string(scratch.app_dir().join(FILE)).unwrap_or_else(|e| panic!("{e}"));
    for kept in [
        "# mine, not the program's",
        "future = \"kept\"",
        "# night owl",
        "puppy = true",
        "look = \"arc\"",
        "sparkle = 3",
        "[later]",
        "answer = 42",
    ] {
        assert!(written.contains(kept), "{kept} was lost:\n{written}");
    }
    assert!(written.contains("theme = \"light\""), "{written}");
    let again = store.load::<AppearanceFile>();
    assert_eq!(again.value, changed);
    assert_eq!(
        unknown(&again),
        reported,
        "the unknown keys are still reported"
    );
}

#[test]
fn a_save_of_a_fresh_file_is_what_the_encoder_writes() {
    let scratch = Scratch::new();
    let file = with(|f| {
        f.appearance.theme = Theme::Dark;
        f.icons.dark_mode_variant = IconDarkVariant::Adaptive;
    });
    scratch
        .store()
        .save(&file)
        .unwrap_or_else(|e| panic!("{e}"));
    let written =
        std::fs::read_to_string(scratch.app_dir().join(FILE)).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        written,
        toml::to_string(&file).unwrap_or_else(|e| panic!("{e}"))
    );
}
