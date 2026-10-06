//! The helpers file an app ships and the entries it parses into. Parsing is pure: text in, a
//! [`Catalog`] or the first thing wrong with it out.

use crate::capability::{Capability, Executable, NameError, PackageName};
use crate::family::Family;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What one capability needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The tool's name as the person knows it ("mpv").
    pub tool: String,
    /// What it is for, finishing "{app} needs {tool} to ...": "play videos".
    pub purpose: String,
    /// Any one of these on `PATH` proves the tool is there.
    pub probe: Vec<Executable>,
    packages: Packages,
}

/// The package alternatives per family, in preference order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Packages {
    dnf: Vec<PackageName>,
    apt: Vec<PackageName>,
    pacman: Vec<PackageName>,
    zypper: Vec<PackageName>,
}

impl Entry {
    /// The packages to try on `family`, best first; empty when the file names none for it.
    pub fn candidates(&self, family: Family) -> &[PackageName] {
        match family {
            Family::Dnf => &self.packages.dnf,
            Family::Apt => &self.packages.apt,
            Family::Pacman => &self.packages.pacman,
            Family::Zypper => &self.packages.zypper,
        }
    }
}

/// Every capability one helpers file declares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    entries: BTreeMap<Capability, Entry>,
}

/// Why a helpers file could not be used.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// The text is not TOML of the expected shape.
    #[error("the helpers file does not parse: {0}")]
    Syntax(#[from] toml::de::Error),
    /// A name in it is not valid.
    #[error("capability {capability}: {source}")]
    Name {
        /// The capability whose entry holds the name.
        capability: String,
        /// What is wrong with the name.
        source: NameError,
    },
    /// An entry lacks something it needs.
    #[error("capability {capability}: {what}")]
    Missing {
        /// The capability.
        capability: String,
        /// What is absent.
        what: &'static str,
    },
    /// No file for the app in any data directory.
    #[error("no helpers file {file} in any data directory")]
    NotFound {
        /// The file name looked for.
        file: String,
    },
    /// The file exists but could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// The file.
        path: PathBuf,
        /// The I/O error.
        source: std::io::Error,
    },
}

#[derive(Deserialize)]
struct RawEntry {
    tool: String,
    purpose: String,
    probe: Vec<String>,
    #[serde(default)]
    packages: RawPackages,
}

/// Unknown family keys are ignored, so a newer file still loads on an older quire.
#[derive(Deserialize, Default)]
struct RawPackages {
    #[serde(default)]
    dnf: Vec<String>,
    #[serde(default)]
    apt: Vec<String>,
    #[serde(default)]
    pacman: Vec<String>,
    #[serde(default)]
    zypper: Vec<String>,
}

fn names<T>(
    capability: &str,
    raw: &[String],
    make: impl Fn(&str) -> Result<T, NameError>,
) -> Result<Vec<T>, CatalogError> {
    raw.iter()
        .map(|name| {
            make(name).map_err(|source| CatalogError::Name {
                capability: capability.to_owned(),
                source,
            })
        })
        .collect()
}

fn entry(capability: &str, raw: RawEntry) -> Result<Entry, CatalogError> {
    let missing = |what| CatalogError::Missing {
        capability: capability.to_owned(),
        what,
    };
    if raw.tool.trim().is_empty() {
        return Err(missing("`tool` is empty"));
    }
    if raw.purpose.trim().is_empty() {
        return Err(missing("`purpose` is empty"));
    }
    if raw.probe.is_empty() {
        return Err(missing("`probe` names no executable"));
    }
    let packages = Packages {
        dnf: names(capability, &raw.packages.dnf, PackageName::new)?,
        apt: names(capability, &raw.packages.apt, PackageName::new)?,
        pacman: names(capability, &raw.packages.pacman, PackageName::new)?,
        zypper: names(capability, &raw.packages.zypper, PackageName::new)?,
    };
    Ok(Entry {
        tool: raw.tool,
        purpose: raw.purpose,
        probe: names(capability, &raw.probe, Executable::new)?,
        packages,
    })
}

impl Catalog {
    /// Parse a helpers file.
    pub fn parse(text: &str) -> Result<Catalog, CatalogError> {
        let raw: BTreeMap<String, RawEntry> = toml::from_str(text)?;
        let entries = raw
            .into_iter()
            .map(|(key, raw)| {
                let capability = Capability::new(&key).map_err(|source| CatalogError::Name {
                    capability: key.clone(),
                    source,
                })?;
                Ok((capability, entry(&key, raw)?))
            })
            .collect::<Result<_, CatalogError>>()?;
        Ok(Catalog { entries })
    }

    /// The app's helpers file: the first `<dir>/quire/helpers/<app>.toml` among `data_dirs`
    /// (an `XDG_DATA_DIRS` list, earlier entries first).
    pub fn load(app: &str, data_dirs: &[PathBuf]) -> Result<Catalog, CatalogError> {
        let file = format!("{app}.toml");
        let found = data_dirs
            .iter()
            .map(|dir| dir.join("quire").join("helpers").join(&file))
            .find(|path| Path::is_file(path));
        let path = found.ok_or(CatalogError::NotFound { file })?;
        let text = std::fs::read_to_string(&path).map_err(|source| CatalogError::Read {
            path: path.clone(),
            source,
        })?;
        Catalog::parse(&text)
    }

    /// What `capability` needs, if the file declares it.
    pub fn get(&self, capability: &Capability) -> Option<&Entry> {
        self.entries.get(capability)
    }

    /// Every capability the file declares, in name order.
    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.entries.keys()
    }
}

/// The directories of an `XDG_DATA_DIRS` value: the default list when it is unset or empty, and
/// empty or relative entries dropped.
pub fn data_dirs(value: Option<&std::ffi::OsStr>) -> Vec<PathBuf> {
    let list = value
        .filter(|value| !value.is_empty())
        .unwrap_or(std::ffi::OsStr::new("/usr/local/share:/usr/share"));
    std::env::split_paths(list)
        .filter(|dir| dir.is_absolute())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Catalog, CatalogError, data_dirs};
    use crate::capability::Capability;
    use crate::family::Family;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    const GOOD: &str = r#"
        [video-playback]
        tool = "mpv"
        purpose = "play videos"
        probe = ["mpv"]
        [video-playback.packages]
        dnf = ["mpv"]
        apt = ["mpv"]
        future-distro = ["whatever"]

        [media-probe]
        tool = "FFmpeg"
        purpose = "read video details"
        probe = ["ffprobe", "ffmpeg"]
        [media-probe.packages]
        dnf = ["ffmpeg", "ffmpeg-free"]
    "#;

    fn cap(name: &str) -> Capability {
        Capability::new(name).expect("a capability")
    }

    #[test]
    fn a_good_file_parses_and_resolves_per_family() {
        let catalog = Catalog::parse(GOOD).expect("parses");
        let video = catalog.get(&cap("video-playback")).expect("declared");
        assert_eq!(video.tool, "mpv");
        assert_eq!(video.candidates(Family::Apt)[0].as_str(), "mpv");
        assert!(video.candidates(Family::Pacman).is_empty());
        let probe = catalog.get(&cap("media-probe")).expect("declared");
        let dnf: Vec<_> = probe
            .candidates(Family::Dnf)
            .iter()
            .map(|p| p.as_str())
            .collect();
        assert_eq!(dnf, ["ffmpeg", "ffmpeg-free"]);
        assert_eq!(probe.probe.len(), 2);
        assert!(catalog.get(&cap("nope")).is_none());
    }

    #[test]
    fn a_bad_file_names_what_is_wrong() {
        let table: &[(&str, &str, &str)] = &[
            ("not toml", "[[[", "does not parse"),
            (
                "bad key",
                "[Bad]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"x\"]\n",
                "lowercase",
            ),
            (
                "no probe",
                "[a]\ntool=\"x\"\npurpose=\"y\"\nprobe=[]\n",
                "probe",
            ),
            (
                "empty purpose",
                "[a]\ntool=\"x\"\npurpose=\" \"\nprobe=[\"x\"]\n",
                "purpose",
            ),
            (
                "path probe",
                "[a]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"/bin/x\"]\n",
                "bare file name",
            ),
            (
                "group package",
                "[a]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"x\"]\n[a.packages]\ndnf=[\"@multimedia\"]\n",
                "plain package name",
            ),
            (
                "missing tool",
                "[a]\npurpose=\"y\"\nprobe=[\"x\"]\n",
                "does not parse",
            ),
        ];
        for (name, text, expect) in table {
            let error = Catalog::parse(text).expect_err(name).to_string();
            assert!(error.contains(expect), "{name}: {error}");
        }
    }

    #[test]
    fn load_takes_the_first_data_dir_that_has_the_file() {
        let root = std::env::temp_dir().join(format!("ds-helpers-load-{}", std::process::id()));
        let (first, second) = (root.join("a"), root.join("b"));
        for (dir, tool) in [(&first, "first"), (&second, "second")] {
            let folder = dir.join("quire/helpers");
            std::fs::create_dir_all(&folder).expect("scratch dir");
            let body = format!("[x]\ntool=\"{tool}\"\npurpose=\"p\"\nprobe=[\"x\"]\n");
            std::fs::write(folder.join("anyview.toml"), body).expect("scratch file");
        }
        let both = [first.clone(), second.clone()];
        let loaded = Catalog::load("anyview", &both).expect("loads");
        assert_eq!(loaded.get(&cap("x")).expect("x").tool, "first");
        let only_second = [root.join("none"), second];
        let loaded = Catalog::load("anyview", &only_second).expect("loads");
        assert_eq!(loaded.get(&cap("x")).expect("x").tool, "second");
        let missing = Catalog::load("other", &both);
        assert!(matches!(missing, Err(CatalogError::NotFound { .. })));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn data_dirs_follow_the_xdg_rules() {
        let paths = |dirs: Vec<PathBuf>| -> Vec<String> {
            dirs.iter().map(|d| d.display().to_string()).collect()
        };
        assert_eq!(paths(data_dirs(None)), ["/usr/local/share", "/usr/share"]);
        assert_eq!(
            paths(data_dirs(Some(OsStr::new("")))),
            ["/usr/local/share", "/usr/share"]
        );
        assert_eq!(
            paths(data_dirs(Some(OsStr::new("/a::rel:/b")))),
            ["/a", "/b"]
        );
    }
}
