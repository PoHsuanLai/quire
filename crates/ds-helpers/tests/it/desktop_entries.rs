//! The desktop-entry layer on scratch folders and plain values: the folders searched, the walk
//! and its ids, the five-rule window match, and the identity an entry declares.

use ds_helpers::{
    AppIdentity, AppKeys, DESKTOP_ENTRY_CAP, DesktopFile, DesktopId, XdgEnv, entry_for_window,
    first_of_each_id, load_entries, match_app, walk_applications,
};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// One test's own directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ds-helpers-entries-{}-{n}-{tag}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch dir");
        Scratch(path)
    }

    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("folders");
        fs::write(&path, text).expect("write");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn paths(dirs: Vec<PathBuf>) -> Vec<String> {
    dirs.iter().map(|dir| dir.display().to_string()).collect()
}

#[test]
fn the_folders_searched_follow_the_xdg_rules_and_add_the_stores_once() {
    let env = |home: Option<&str>, dirs: Option<&str>, user: Option<&str>| XdgEnv {
        data_home: home.map(OsString::from),
        data_dirs: dirs.map(OsString::from),
        home: user.map(PathBuf::from),
    };
    let stores = [
        "/var/lib/flatpak/exports/share/applications",
        "/var/lib/snapd/desktop/applications",
    ];
    // name, environment, folders before the stores, the user's flatpak folder
    let cases = [
        (
            "defaults under HOME",
            env(None, None, Some("/home/a")),
            vec![
                "/home/a/.local/share/applications",
                "/usr/local/share/applications",
                "/usr/share/applications",
            ],
            Some("/home/a/.local/share/flatpak/exports/share/applications"),
        ),
        (
            "XDG_DATA_HOME wins, a relative one does not",
            env(Some("/data"), Some("/x"), Some("/home/a")),
            vec!["/data/applications", "/x/applications"],
            Some("/data/flatpak/exports/share/applications"),
        ),
        (
            "relative XDG_DATA_HOME falls back to HOME",
            env(Some("rel"), Some("/x"), Some("/home/a")),
            vec!["/home/a/.local/share/applications", "/x/applications"],
            Some("/home/a/.local/share/flatpak/exports/share/applications"),
        ),
        (
            "no home at all",
            env(None, Some("/x::rel"), None),
            vec!["/x/applications"],
            None,
        ),
    ];
    for (name, env, first, user_store) in cases {
        let mut want: Vec<String> = first.iter().map(|dir| (*dir).to_owned()).collect();
        want.extend(user_store.map(str::to_owned));
        want.extend(stores.map(str::to_owned));
        assert_eq!(paths(env.applications_dirs()), want, "{name}");
    }

    let listed = env(
        Some("/h"),
        Some("/var/lib/flatpak/exports/share:/usr/share"),
        None,
    );
    let got = paths(listed.applications_dirs());
    let flatpak = "/var/lib/flatpak/exports/share/applications";
    assert_eq!(got.iter().filter(|dir| *dir == flatpak).count(), 1);
    assert_eq!(
        got.get(1).map(String::as_str),
        Some(flatpak),
        "keeps its place"
    );
}

#[test]
fn a_desktop_id_dashes_the_folders_and_keeps_the_suffix() {
    // name, relative path, id
    let cases = [
        ("plain", "foot.desktop", Some("foot.desktop")),
        ("one folder", "kde/foo.desktop", Some("kde-foo.desktop")),
        ("two folders", "a/b/c.desktop", Some("a-b-c.desktop")),
        ("reverse dns", "org.a.B.desktop", Some("org.a.B.desktop")),
        ("not an entry", "notes.txt", None),
        ("no name", ".desktop", None),
    ];
    for (name, relative, want) in cases {
        let got = DesktopId::from_relative(Path::new(relative));
        assert_eq!(got.as_ref().map(DesktopId::as_str), want, "{name}");
    }
    let id = DesktopId::parse("kde-foo").expect("an id");
    assert_eq!((id.as_str(), id.stem()), ("kde-foo.desktop", "kde-foo"));
    assert_eq!(DesktopId::parse(".desktop"), None);
}

#[test]
fn the_walk_recurses_in_order_ends_at_a_loop_and_the_first_folder_shadows() {
    let first = Scratch::new("first");
    let second = Scratch::new("second");
    first.write("b.desktop", "");
    first.write("a.desktop", "");
    first.write("notes.txt", "");
    first.write("kde/c.desktop", "");
    second.write("a.desktop", "");
    second.write("z.desktop", "");
    std::os::unix::fs::symlink(&first.0, first.0.join("kde/loop")).expect("link");
    let roots = [first.0.clone(), second.0.join("missing"), second.0.clone()];

    let all = walk_applications(&roots);
    let ids: Vec<&str> = all.iter().map(|file| file.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "a.desktop",
            "b.desktop",
            "kde-c.desktop",
            "a.desktop",
            "z.desktop"
        ]
    );
    let once = first_of_each_id(all);
    let got: Vec<(&str, bool)> = once
        .iter()
        .map(|file| (file.id.as_str(), file.path.starts_with(&first.0)))
        .collect();
    let want = [
        ("a.desktop", true),
        ("b.desktop", true),
        ("kde-c.desktop", true),
        ("z.desktop", false),
    ];
    assert_eq!(got, want);
}

fn keys(id: &str, class: Option<&str>, flatpak: Option<&str>) -> AppKeys {
    AppKeys {
        id: id.to_owned(),
        startup_wm_class: class.map(str::to_owned),
        flatpak: flatpak.map(str::to_owned),
    }
}

#[test]
fn each_rule_matches_in_order() {
    let apps = [
        keys("org.mozilla.firefox", None, None),
        keys("org.gnome.Nautilus", None, None),
        keys("foot", None, None),
        keys("code", Some("Code"), None),
        keys(
            "com.valvesoftware.Steam",
            Some("steam"),
            Some("com.valvesoftware.Steam"),
        ),
        keys("spotify-launcher", None, Some("com.spotify.Client")),
        keys("org.kde.fox", None, None),
        keys("net.lutris.Lutris", None, None),
        keys("lutris", None, None),
    ];
    // name, app id, wm class, the id wanted
    let cases: &[(&str, &str, Option<&str>, Option<&str>)] = &[
        ("exact", "foot", None, Some("foot")),
        (
            "exact reverse-dns",
            "org.gnome.Nautilus",
            None,
            Some("org.gnome.Nautilus"),
        ),
        (
            "lowercase",
            "org.gnome.nautilus",
            None,
            Some("org.gnome.Nautilus"),
        ),
        ("lowercase simple", "FOOT", None, Some("foot")),
        ("class by app id", "code", None, Some("code")),
        ("class, case ignored", "CODE", None, Some("code")),
        (
            "class by wm_class",
            "",
            Some("steam"),
            Some("com.valvesoftware.Steam"),
        ),
        ("suffix", "firefox", None, Some("org.mozilla.firefox")),
        (
            "suffix of two labels",
            "mozilla.firefox",
            None,
            Some("org.mozilla.firefox"),
        ),
        (
            "flatpak id",
            "com.spotify.Client",
            None,
            Some("spotify-launcher"),
        ),
        (
            "flatpak id, case ignored",
            "com.spotify.client",
            None,
            Some("spotify-launcher"),
        ),
        ("an exact id beats a suffix", "lutris", None, Some("lutris")),
        ("a substring is not a label", "irefox", None, None),
        ("a label is whole", "fox", None, Some("org.kde.fox")),
        ("unknown", "org.example.Nothing", None, None),
        ("empty", "", None, None),
        ("blank wm class", " ", Some(" "), None),
        (
            "steam by suffix, class and flatpak agree",
            "Steam",
            None,
            Some("com.valvesoftware.Steam"),
        ),
    ];
    for &(name, app_id, wm_class, want) in cases {
        let got = match_app(app_id, wm_class, &apps).map(|at| apps[at].id.as_str());
        assert_eq!(got, want, "{name}: {app_id:?} / {wm_class:?}");
    }
}

#[test]
fn a_window_finds_its_entry_on_disk_by_the_same_rules() {
    let first = Scratch::new("window-first");
    let second = Scratch::new("window-second");
    first.write("org.a.Exact.desktop", "[Desktop Entry]\nIcon=exact\n");
    first.write("kde/tool.desktop", "[Desktop Entry]\nIcon=in-folder\n");
    first.write(
        "zzz.desktop",
        "[Desktop Entry]\nIcon=by-class\nStartupWMClass=Classy\n",
    );
    first.write("dup.desktop", "[Desktop Entry]\nIcon=first\n");
    first.write("broken.desktop", "no group here\n");
    second.write("dup.desktop", "[Desktop Entry]\nIcon=second\n");
    second.write(
        "org.b.Shop.desktop",
        "[Desktop Entry]\nIcon=shop\nX-Flatpak=com.shop.App\n",
    );
    let entries = load_entries(&[first.0.clone(), second.0.clone()]);
    // name, app id, icon
    let cases = [
        ("the file named so", "org.a.Exact", Some("exact")),
        ("a folder's dashed id", "kde-tool", Some("in-folder")),
        ("the class", "Classy", Some("by-class")),
        ("the first folder wins", "dup", Some("first")),
        ("the flatpak id", "com.shop.App", Some("shop")),
        ("a file with no entry group", "broken", None),
        ("none", "nothing", None),
    ];
    for (name, app_id, icon) in cases {
        let got = entry_for_window(&entries, app_id, None).and_then(|entry| entry.icon());
        assert_eq!(got.as_deref(), icon, "{name}");
    }
}

#[test]
fn an_identity_reads_x_keys_and_falls_back_to_the_entry() {
    let identity = |id: &str, text: &str| {
        let file = DesktopFile::parse(text, DESKTOP_ENTRY_CAP).expect("under the cap");
        AppIdentity::from_entry(id_of(id), file.entry().expect("entry group"))
    };
    let declared = identity(
        "org.quire.Mail.desktop",
        "[Desktop Entry]\nStartupWMClass=mailo\nX-Quire-WindowId=mailo-main\n\
         X-Quire-BusNames=org.quire.Mail; io.github.PoHsuanLai.mailo;;\nX-Quire-IntentsApp=org.quire.Mail\n",
    );
    assert_eq!(declared.desktop_id, id_of("org.quire.Mail"));
    assert_eq!(declared.window_id, "mailo-main");
    assert_eq!(
        declared.bus_names,
        ["org.quire.Mail", "io.github.PoHsuanLai.mailo"]
    );
    assert_eq!(declared.intents_app.as_deref(), Some("org.quire.Mail"));

    let by_class = identity("foo.desktop", "[Desktop Entry]\nStartupWMClass=Foo\n");
    assert_eq!(by_class.window_id, "Foo");
    let bare = identity(
        "kde-foo.desktop",
        "[Desktop Entry]\nName=Foo\nX-Quire-IntentsApp= \n",
    );
    assert_eq!(bare.window_id, "kde-foo");
    assert!(bare.bus_names.is_empty());
    assert_eq!(bare.intents_app, None);
}

fn id_of(text: &str) -> DesktopId {
    DesktopId::parse(text).expect("an id")
}
