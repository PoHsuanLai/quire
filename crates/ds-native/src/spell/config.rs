//! Where the dictionaries are: the system's Hunspell directories, and the user's own words
//! under XDG data (`~/.local/share/quire/spelling/<lang>.dic`, one word per line).

use ds::Lang;
use std::path::PathBuf;

/// The system's Hunspell directories, searched in order. Fedora installs `hunspell-*` packages
/// into the first; Debian's `hunspell-*` and `myspell-*` use the others.
pub const SYSTEM_DICTIONARIES: &[&str] = &[
    "/usr/share/hunspell",
    "/usr/share/myspell",
    "/usr/share/myspell/dicts",
];

/// Where the checker reads dictionaries from and writes learned words to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellConfig {
    /// Directories holding `<lang>.aff` and `<lang>.dic` pairs, searched in order.
    pub dictionaries: Vec<PathBuf>,
    /// The directory of the user's word lists, `<lang>.dic`, created on the first word learned.
    pub user: PathBuf,
}

impl SpellConfig {
    /// The system's dictionaries (and `$XDG_DATA_HOME/hunspell`), and the user's words under
    /// `$XDG_DATA_HOME/quire/spelling`.
    pub fn system() -> SpellConfig {
        let data = dirs::data_dir().unwrap_or_else(|| PathBuf::from(".local/share"));
        let mut dictionaries: Vec<PathBuf> =
            SYSTEM_DICTIONARIES.iter().map(PathBuf::from).collect();
        dictionaries.push(data.join("hunspell"));
        SpellConfig {
            dictionaries,
            user: data.join("quire").join("spelling"),
        }
    }

    /// Every language with both files installed, in search order, without repeats.
    pub fn installed(&self) -> Vec<Lang> {
        let mut found: Vec<Lang> = Vec::new();
        for dir in &self.dictionaries {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut here: Vec<Lang> = entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    let stem = path.file_stem()?.to_str()?.to_owned();
                    let paired = path.extension()? == "dic" && path.with_extension("aff").is_file();
                    paired.then(|| Lang::parse(&stem).ok()).flatten()
                })
                .collect();
            here.sort();
            here.retain(|lang| !found.contains(lang));
            found.extend(here);
        }
        found
    }

    /// The `.aff` and `.dic` files of an installed `lang`, from the first directory with both.
    pub fn files(&self, lang: &Lang) -> Option<(PathBuf, PathBuf)> {
        self.dictionaries.iter().find_map(|dir| {
            let aff = dir.join(format!("{}.aff", lang.name()));
            let dic = dir.join(format!("{}.dic", lang.name()));
            (aff.is_file() && dic.is_file()).then_some((aff, dic))
        })
    }
}
