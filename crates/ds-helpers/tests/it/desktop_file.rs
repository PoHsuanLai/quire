//! A real-looking desktop entry read through the public surface: its keys by type, its
//! translations for a person's locales, its actions, and the size cap.

use ds_helpers::{
    DESKTOP_ENTRY_CAP, DesktopFile, DesktopFileError, EntryLocale, ReadCap, read_text_capped,
};
use std::fs;

const FILES: &str = "\u{feff}# A comment\n\
[Desktop Entry]\n\
Type=Application\n\
Name=Files\n\
Name[de]=Dateien\n\
Name[de_AT]=Dateien (AT)\n\
Name[sr@latin]=Fajlovi\n\
Comment=Browse\\sthe\\sfile\\ssystem\n\
Exec=files --new-window %U\n\
Terminal=false\n\
NoDisplay=true\n\
Hidden=yes\n\
MimeType=inode/directory;application/x-gnome-saved-search;\n\
Keywords=folder;manager;\n\
Keywords[de]=Ordner;Verwaltung;\n\
Actions=new-window;\n\
\n\
[Desktop Action new-window]\n\
Name=New Window\n\
Exec=files --new-window\n";

fn files() -> DesktopFile {
    DesktopFile::parse(FILES, DESKTOP_ENTRY_CAP).expect("under the cap")
}

fn locales(texts: &[&str]) -> Vec<EntryLocale> {
    texts
        .iter()
        .filter_map(|text| EntryLocale::parse(text))
        .collect()
}

#[test]
fn an_entry_is_read_by_key_type() {
    let file = files();
    let entry = file.entry().expect("the entry group");
    assert_eq!(entry.string("Type").as_deref(), Some("Application"));
    assert_eq!(entry.string("Name").as_deref(), Some("Files"));
    assert_eq!(
        entry.string("Comment").as_deref(),
        Some("Browse the file system"),
        "escapes are taken"
    );
    assert_eq!(
        entry.string("Exec").as_deref(),
        Some("files --new-window %U"),
        "field codes are left to the caller"
    );
    assert_eq!(
        entry.list("MimeType"),
        ["inode/directory", "application/x-gnome-saved-search"]
    );
    assert_eq!(entry.boolean("Terminal"), Some(false));
    assert!(entry.is_true("NoDisplay"));
    assert_eq!(entry.boolean("Hidden"), None, "yes is not a boolean");
    assert!(!entry.is_true("Hidden"));
    assert!(!entry.is_true("Missing"));
    assert_eq!(file.skipped_lines(), 0);
}

#[test]
fn a_translation_follows_the_persons_locales() {
    // `(case, locales, Name, Keywords)`.
    const CASES: &[(&str, &[&str], &str, &[&str])] = &[
        ("none asked", &[], "Files", &["folder", "manager"]),
        (
            "C has no translation",
            &["C"],
            "Files",
            &["folder", "manager"],
        ),
        (
            "exact language",
            &["de"],
            "Dateien",
            &["Ordner", "Verwaltung"],
        ),
        (
            "language and country",
            &["de_AT.UTF-8"],
            "Dateien (AT)",
            &["Ordner", "Verwaltung"],
        ),
        (
            "country falls back to language",
            &["de_CH"],
            "Dateien",
            &["Ordner", "Verwaltung"],
        ),
        (
            "modifier",
            &["sr_RS@latin"],
            "Fajlovi",
            &["folder", "manager"],
        ),
        (
            "a language with none falls to the default",
            &["fr_FR"],
            "Files",
            &["folder", "manager"],
        ),
        (
            "the first locale with a translation wins",
            &["fr", "de"],
            "Dateien",
            &["Ordner", "Verwaltung"],
        ),
    ];
    let file = files();
    let entry = file.entry().expect("the entry group");
    for (case, asked, name, keywords) in CASES {
        let asked = locales(asked);
        assert_eq!(
            entry.localized("Name", &asked).as_deref(),
            Some(*name),
            "{case}"
        );
        assert_eq!(
            entry.localized_list("Keywords", &asked),
            *keywords,
            "{case}"
        );
    }
}

#[test]
fn an_action_group_is_found_by_the_name_the_entry_lists() {
    let file = files();
    let entry = file.entry().expect("the entry group");
    let names: Vec<_> = entry
        .list("Actions")
        .into_iter()
        .filter_map(|id| {
            file.group(&format!("Desktop Action {id}"))
                .map(|g| g.name().to_owned())
        })
        .collect();
    assert_eq!(names, ["Desktop Action new-window"]);
    let groups: Vec<_> = file.groups().map(|group| group.name()).collect();
    assert_eq!(groups, ["Desktop Entry", "Desktop Action new-window"]);
}

#[test]
fn what_cannot_be_read_costs_only_its_line() {
    // `(case, text, Name, skipped lines)`.
    const CASES: &[(&str, &str, Option<&str>, usize)] = &[
        (
            "a pair before any group",
            "Name=x\n[Desktop Entry]\nName=y\n",
            Some("y"),
            1,
        ),
        (
            "a line with no equals sign",
            "[Desktop Entry]\nName=y\nnonsense\n",
            Some("y"),
            1,
        ),
        (
            "a key given twice keeps the last",
            "[Desktop Entry]\nName=a\nName=b\n",
            Some("b"),
            0,
        ),
        (
            "CRLF line ends",
            "[Desktop Entry]\r\nName=a\r\n",
            Some("a"),
            0,
        ),
        ("empty text", "", None, 0),
    ];
    for (case, text, name, skipped) in CASES {
        let file = DesktopFile::parse(text, DESKTOP_ENTRY_CAP).expect("under the cap");
        let got = file.entry().and_then(|entry| entry.string("Name"));
        assert_eq!(got.as_deref(), *name, "{case}");
        assert_eq!(file.skipped_lines(), *skipped, "{case}");
    }
}

#[test]
fn a_duplicate_group_merges_its_keys() {
    let text = "[Desktop Entry]\nName=a\n[Other]\nX=1\n[Desktop Entry]\nExec=e\n";
    let file = DesktopFile::parse(text, DESKTOP_ENTRY_CAP).expect("under the cap");
    let entry = file.entry().expect("the entry group");
    assert_eq!(entry.key_names().collect::<Vec<_>>(), ["Exec", "Name"]);
    assert_eq!(file.groups().count(), 2);
}

#[test]
fn a_file_over_the_cap_is_refused_whole() {
    let cap = ReadCap::bytes(16);
    let text = "[Desktop Entry]\nName=x\n";
    assert_eq!(
        DesktopFile::parse(text, cap),
        Err(DesktopFileError::TooBig {
            size: text.len() as u64,
            cap
        })
    );
    assert_eq!(
        DesktopFile::from_bytes(text.as_bytes(), cap),
        Err(DesktopFileError::TooBig {
            size: text.len() as u64,
            cap
        })
    );
    let exact = ReadCap::bytes(text.len() as u64);
    assert!(
        DesktopFile::parse(text, exact).is_ok(),
        "a text at the cap is read"
    );
}

#[test]
fn bytes_that_are_not_utf8_are_replaced_not_refused() {
    let bytes = b"[Desktop Entry]\nName=Caf\xe9\nExec=cafe\n";
    let file = DesktopFile::from_bytes(bytes, DESKTOP_ENTRY_CAP).expect("under the cap");
    let entry = file.entry().expect("the entry group");
    assert_eq!(entry.string("Name").as_deref(), Some("Caf\u{fffd}"));
    assert_eq!(entry.string("Exec").as_deref(), Some("cafe"));
}

#[test]
fn a_file_on_disk_reads_through_the_capped_reader() {
    let dir = std::env::temp_dir().join(format!("ds-helpers-desktop-file-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    let path = dir.join("files.desktop");
    fs::write(&path, FILES).expect("write");
    let text = read_text_capped(&path, DESKTOP_ENTRY_CAP).expect("read");
    let file = DesktopFile::parse(&text, DESKTOP_ENTRY_CAP).expect("under the cap");
    assert_eq!(
        file.entry().and_then(|e| e.string("Name")).as_deref(),
        Some("Files")
    );
    let _ = fs::remove_dir_all(&dir);
}
