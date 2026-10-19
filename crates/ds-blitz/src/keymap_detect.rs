//! The keymap source a launched app gets when it names none, and the one an app that builds its own
//! keymap asks for: the platform from the environment (chordkit's `detect`), and for it the
//! desktop's own shortcut settings. Plasma's are in `kdeglobals`; GNOME's are in the dconf
//! database, read through `dconf dump /`; every other platform has its conventions alone. Both
//! stores are watched, so a change the person makes in the desktop's settings reaches the app as it
//! runs. Our desktop's own source (keycap) is plugged in by the desktop's launcher, not found here.
//! The environment is read through a lookup the caller passes, so a test supplies its own.

use chordkit::{
    ChangeCallback, Desktop, HostOs, Keymap, KeymapSource, Platform, SourceError, Watch,
};
use chordkit_gnome::{GnomeSource, parse_dconf_dump};
use chordkit_kde::KdeSource;
use ds::keys::KeySource;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

/// Where, under the config directory, a GNOME session's saved `dconf dump /` is looked for when
/// there is no `dconf` to run.
pub(crate) const GNOME_DUMP: &str = "chordkit/dconf-dump";

/// How often a watched settings file's modified time is looked at.
const LOOK_EVERY: Duration = Duration::from_secs(2);

/// The most of `dconf dump /` that is read; a larger dump is refused, keeping the last one.
const DUMP_LIMIT: usize = 4 << 20;

/// The platform the lookup describes and the desktop's shortcut settings for it: what a launch
/// that names no source uses.
pub(crate) fn detected_key_source(var: &impl Fn(&str) -> Option<String>) -> KeySource {
    system_key_source(platform_of(var), var)
}

/// The desktop's own shortcut settings for `platform`, under the config directory the lookup
/// names (`XDG_CONFIG_HOME`, else `~/.config`): what a launch that names no source uses, for an
/// app that builds a keymap of its own on the system's outside any window. The returned source
/// is watched for changes; on GNOME the dconf database is read when the source is made and again
/// on the watch's thread, never during a load.
pub fn system_key_source(platform: Platform, var: &impl Fn(&str) -> Option<String>) -> KeySource {
    source_for(platform, config_dir(var), Arc::new(run_dconf_dump))
}

/// The platform the lookup describes (`CHORDKIT_PLATFORM` wins).
pub(crate) fn platform_of(var: &impl Fn(&str) -> Option<String>) -> Platform {
    chordkit::detect(HostOs::compiled(), var)
}

/// The user's config directory: `XDG_CONFIG_HOME`, else `~/.config`.
pub(crate) fn config_dir(var: &impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    var("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| PathBuf::from(home).join(".config")))
}

/// What reading the dconf database gives: its dump, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Dumped {
    Text(String),
    /// There is no `dconf` program.
    NoDconf,
    /// `dconf` failed, or its dump was too large.
    Failed,
}

/// Reads the dconf database: injected, so a test never reads the real desktop's.
type DumpDconf = Arc<dyn Fn() -> Dumped + Send + Sync>;

/// `dconf dump /`.
fn run_dconf_dump() -> Dumped {
    match Command::new("dconf")
        .args(["dump", "/"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() && out.stdout.len() <= DUMP_LIMIT => {
            Dumped::Text(String::from_utf8_lossy(&out.stdout).into_owned())
        }
        Ok(_) => Dumped::Failed,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Dumped::NoDconf,
        Err(_) => Dumped::Failed,
    }
}

/// The source `platform` reads its shortcut settings from, under `config`.
fn source_for(platform: Platform, config: Option<PathBuf>, dump: DumpDconf) -> KeySource {
    match (platform, config) {
        (
            Platform::Linux {
                desktop: Desktop::Kde,
            },
            Some(dir),
        ) => {
            let kdeglobals = dir.join("kdeglobals");
            KeySource::new(
                platform,
                Box::new(Polled {
                    inner: KdeSource::new(&kdeglobals),
                    watched: kdeglobals,
                }),
            )
        }
        (
            Platform::Linux {
                desktop: Desktop::Gnome,
            },
            Some(dir),
        ) => KeySource::new(platform, Box::new(Dconf::new(&dir, dump))),
        (platform, _) => KeySource::conventions(platform),
    }
}

/// GNOME's settings: the dconf database's dump, held in memory. It is read when the source is
/// made and again on the watch's thread when the database changes, so a load only parses. With no
/// `dconf` the dump saved under the config directory is read; a failed `dconf` keeps the last
/// dump it gave.
struct Dconf {
    dump: DumpDconf,
    held: Arc<Mutex<Dumped>>,
    saved: GnomeSource,
    database: PathBuf,
}

impl Dconf {
    fn new(config: &Path, dump: DumpDconf) -> Self {
        let held = Arc::new(Mutex::new(dump()));
        Dconf {
            dump,
            held,
            saved: GnomeSource::new(config.join(GNOME_DUMP)),
            database: config.join("dconf").join("user"),
        }
    }
}

/// Takes `fresh` in place of `held`, except that a failure keeps the dump held before it.
fn take(held: &Mutex<Dumped>, fresh: Dumped) {
    let mut held = held.lock().unwrap_or_else(PoisonError::into_inner);
    if fresh != Dumped::Failed || *held == Dumped::NoDconf {
        *held = fresh;
    }
}

impl KeymapSource for Dconf {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        let held = self
            .held
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        match held {
            Dumped::Text(text) => {
                Ok(Keymap::conventional(platform).with_system(parse_dconf_dump(&text).bindings))
            }
            Dumped::NoDconf => self.saved.load(platform),
            Dumped::Failed => Ok(Keymap::conventional(platform)),
        }
    }

    fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        let (dump, held) = (Arc::clone(&self.dump), Arc::clone(&self.held));
        watch_file(
            &self.database,
            LOOK_EVERY,
            Box::new(move || {
                take(&held, dump());
                on_change();
            }),
        )
    }
}

/// A source whose settings live in one file: loads through `inner`, and watches by looking at the
/// file's modified time.
struct Polled<S> {
    inner: S,
    watched: PathBuf,
}

impl<S: KeymapSource> KeymapSource for Polled<S> {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        self.inner.load(platform)
    }

    fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        watch_file(&self.watched, LOOK_EVERY, on_change)
    }
}

/// Calls `on_change` whenever the file at `path` is changed, made or removed, looking at its
/// modified time every `every`, until the returned `Watch` is dropped. The first look is taken
/// now, not on the thread, so a change made before the thread first runs still counts.
fn watch_file(
    path: &Path,
    every: Duration,
    on_change: ChangeCallback,
) -> Result<Watch, SourceError> {
    let (stop, alive) = channel::<()>();
    let path = path.to_path_buf();
    let seen = modified(&path);
    std::thread::Builder::new()
        .name("ds-keys-watch".to_owned())
        .spawn(move || look(&path, seen, every, &alive, &*on_change))
        .map_err(|err| {
            SourceError::Unsupported(format!("cannot watch the shortcut settings: {err}"))
        })?;
    // Dropping the sender ends the thread at its next look.
    Ok(Watch::holding(stop))
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}

/// Calls `on_change` whenever `path`'s modified time (or its being there) differs from the last
/// look (`seen` at first), until `alive` is closed.
fn look(
    path: &Path,
    mut seen: Option<SystemTime>,
    every: Duration,
    alive: &Receiver<()>,
    on_change: &dyn Fn(),
) {
    loop {
        match alive.recv_timeout(every) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        let now = modified(path);
        if now != seen {
            seen = now;
            on_change();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{Action, Chord, StandardAction};

    type Vars = &'static [(&'static str, &'static str)];

    fn lookup(vars: Vars) -> impl Fn(&str) -> Option<String> {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn the_config_directory_is_xdg_then_home() {
        let cases: &[(Vars, Option<&str>)] = &[
            (
                &[("XDG_CONFIG_HOME", "/x/cfg"), ("HOME", "/h")],
                Some("/x/cfg"),
            ),
            (
                &[("XDG_CONFIG_HOME", ""), ("HOME", "/h")],
                Some("/h/.config"),
            ),
            (&[("HOME", "/h")], Some("/h/.config")),
            (&[], None),
        ];
        for (vars, want) in cases {
            let found = config_dir(&lookup(vars));
            assert_eq!(found, want.map(PathBuf::from), "{vars:?}");
        }
    }

    #[test]
    fn the_platform_comes_from_the_environment() {
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let cases: &[(Vars, Platform)] = &[
            (
                &[("CHORDKIT_PLATFORM", "linux-kde"), ("HOME", "/nowhere")],
                kde,
            ),
            (&[("CHORDKIT_PLATFORM", "linux-ours")], ours),
            (&[("CHORDKIT_PLATFORM", "windows")], Platform::Windows),
        ];
        for (vars, want) in cases {
            assert_eq!(
                detected_key_source(&lookup(vars)).platform(),
                *want,
                "{vars:?}"
            );
        }
    }

    #[test]
    fn a_kde_file_changes_the_keymap_and_a_missing_one_leaves_the_conventions() {
        let dir = std::env::temp_dir().join(format!("ds-blitz-keymap-{}", std::process::id()));
        let written = std::fs::create_dir_all(&dir).and_then(|()| {
            std::fs::write(dir.join("kdeglobals"), "[Shortcuts]\nCopy=Ctrl+Insert\n")
        });
        assert!(written.is_ok(), "{written:?}");
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let copy = Action::Standard(StandardAction::Copy);
        let chords_in = |config: PathBuf| {
            source_for(kde, Some(config), Arc::new(|| Dumped::NoDconf))
                .load()
                .map(|keymap| keymap.chords_of(&copy))
        };
        let changed = chords_in(dir.clone());
        let missing = chords_in(dir.join("nowhere"));
        let _ = std::fs::remove_dir_all(&dir);
        let chord = |text: &str| text.parse::<Chord>().ok();
        assert_eq!(changed.ok(), chord("Ctrl+Insert").map(|c| vec![c]));
        assert_eq!(missing.ok(), chord("Ctrl+C").map(|c| vec![c]));
    }

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ds-blitz-keys-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let made = std::fs::create_dir_all(&dir);
        assert!(made.is_ok(), "{made:?}");
        dir
    }

    const GNOME: Platform = Platform::Linux {
        desktop: Desktop::Gnome,
    };

    /// The chords `source` gives the window manager's minimize action.
    fn minimize_chords(source: &KeySource) -> Vec<String> {
        let minimize = Action::Standard(StandardAction::Minimize);
        source
            .load()
            .map(|keymap| {
                keymap
                    .chords_of(&minimize)
                    .into_iter()
                    .map(Chord::canonical)
                    .collect()
            })
            .unwrap_or_default()
    }

    const DUMP: &str = "[org/gnome/desktop/wm/keybindings]\nminimize=['<Super>h']\n";

    #[test]
    fn gnome_reads_dconf_and_keeps_the_last_dump_when_it_fails() {
        let dir = scratch("gnome-fresh");
        let answer = Arc::new(Mutex::new(Dumped::Text(DUMP.to_owned())));
        let given = Arc::clone(&answer);
        let dconf = Dconf::new(
            &dir,
            Arc::new(move || given.lock().map(|a| a.clone()).unwrap_or(Dumped::Failed)),
        );
        let source = KeySource::new(GNOME, Box::new(dconf));
        let chord = |text: &str| text.parse::<Chord>().map(Chord::canonical).ok();
        let first = minimize_chords(&source);
        assert_eq!(first, chord("Super+H").into_iter().collect::<Vec<_>>());
        // A load never runs dconf: a failure now changes nothing until the database moves.
        if let Ok(mut a) = answer.lock() {
            *a = Dumped::Failed;
        }
        assert_eq!(minimize_chords(&source), first);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_dump_keeps_the_held_one_and_no_dconf_reads_the_saved_dump() {
        let held = Mutex::new(Dumped::Text(DUMP.to_owned()));
        take(&held, Dumped::Failed);
        assert_eq!(
            held.lock().ok().map(|h| h.clone()),
            Some(Dumped::Text(DUMP.to_owned()))
        );
        take(&held, Dumped::NoDconf);
        assert_eq!(held.lock().ok().map(|h| h.clone()), Some(Dumped::NoDconf));

        let dir = scratch("gnome-saved");
        let saved = dir.join(GNOME_DUMP);
        let written = std::fs::create_dir_all(saved.parent().unwrap_or(&dir))
            .and_then(|()| std::fs::write(&saved, DUMP));
        assert!(written.is_ok(), "{written:?}");
        let source = source_for(GNOME, Some(dir.clone()), Arc::new(|| Dumped::NoDconf));
        let chord = |text: &str| text.parse::<Chord>().map(Chord::canonical).ok();
        assert_eq!(
            minimize_chords(&source),
            chord("Super+H").into_iter().collect::<Vec<_>>()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_dconf_change_is_dumped_on_the_watch_thread_before_the_call() {
        let dir = scratch("gnome-watch");
        let database = dir.join("dconf").join("user");
        let made = std::fs::create_dir_all(database.parent().unwrap_or(&dir))
            .and_then(|()| std::fs::write(&database, ""));
        assert!(made.is_ok(), "{made:?}");
        let answer = Arc::new(Mutex::new(Dumped::Text(String::new())));
        let given = Arc::clone(&answer);
        let dconf = Dconf::new(
            &dir,
            Arc::new(move || given.lock().map(|a| a.clone()).unwrap_or(Dumped::Failed)),
        );
        let held = Arc::clone(&dconf.held);
        let (told, heard) = std::sync::mpsc::channel::<Dumped>();
        let watch = watch_file(&database, Duration::from_millis(20), {
            let (dump, held) = (Arc::clone(&dconf.dump), Arc::clone(&held));
            Box::new(move || {
                take(&held, dump());
                let _ = told.send(held.lock().map(|h| h.clone()).unwrap_or(Dumped::Failed));
            })
        });
        assert!(watch.is_ok());
        if let Ok(mut a) = answer.lock() {
            *a = Dumped::Text(DUMP.to_owned());
        }
        let moved = std::fs::File::options()
            .write(true)
            .open(&database)
            .and_then(|file| file.set_modified(SystemTime::now() + Duration::from_secs(60)));
        assert!(moved.is_ok(), "{moved:?}");
        assert_eq!(
            heard.recv_timeout(Duration::from_secs(10)).ok(),
            Some(Dumped::Text(DUMP.to_owned()))
        );
        drop(watch);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_changed_file_calls_back_and_a_dropped_watch_stops() {
        let dir = scratch("watch");
        let path = dir.join("kdeglobals");
        assert!(std::fs::write(&path, "[Shortcuts]\n").is_ok());
        let (told, heard) = std::sync::mpsc::channel::<()>();
        let watch = watch_file(
            &path,
            Duration::from_millis(20),
            Box::new(move || {
                let _ = told.send(());
            }),
        );
        assert!(watch.is_ok(), "no watch");
        // A later modified time is a change, whatever the clock's resolution.
        let moved = std::fs::File::options()
            .write(true)
            .open(&path)
            .and_then(|file| file.set_modified(SystemTime::now() + Duration::from_secs(60)));
        assert!(moved.is_ok(), "{moved:?}");
        assert!(
            heard.recv_timeout(Duration::from_secs(10)).is_ok(),
            "no call after the change"
        );
        drop(watch);
        // The thread ends and drops the callback with it, which closes the channel.
        assert!(
            heard.recv_timeout(Duration::from_secs(10)).is_err(),
            "the watch went on after it was dropped"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
