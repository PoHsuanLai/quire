//! The worker's dictionaries: the system's Hunspell files, loaded on first use per language,
//! with the user's learned words added, and the words ignored this session. Runs on the worker
//! thread only; every read of a file happens here.

use super::choose::pick;
use super::config::SpellConfig;
use ds::{Lang, Learned};
use spellbook::Dictionary;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// What the worker holds.
pub(crate) struct Books {
    config: SpellConfig,
    /// Each asked language's dictionary; `None` when none is installed or it would not parse.
    loaded: HashMap<Lang, Option<Dictionary>>,
    ignored: HashSet<String>,
    /// The installed languages, read once.
    installed: Option<Vec<Lang>>,
}

impl Books {
    pub(crate) fn new(config: SpellConfig) -> Books {
        Books {
            config,
            loaded: HashMap::new(),
            ignored: HashSet::new(),
            installed: None,
        }
    }

    /// The words of `words` no dictionary of `langs` accepts. With no dictionary at all, none:
    /// a surface that cannot be checked shows no marks rather than marking every word.
    pub(crate) fn check(&mut self, langs: &[Lang], words: Vec<String>) -> Vec<String> {
        let books = self.books(langs);
        if books.is_empty() {
            return Vec::new();
        }
        words
            .into_iter()
            .filter(|word| !self.ignored.contains(word))
            .filter(|word| !books.iter().any(|lang| self.accepts(lang, word)))
            .collect()
    }

    /// Replacements for `word`, from each language's dictionary in turn, without repeats.
    pub(crate) fn suggest(&mut self, langs: &[Lang], word: &str) -> Vec<String> {
        let books = self.books(langs);
        let mut found: Vec<String> = Vec::new();
        for lang in books {
            let mut more = Vec::new();
            if let Some(Some(dictionary)) = self.loaded.get(&lang) {
                dictionary.suggest(word, &mut more);
            }
            for guess in more {
                if !found.contains(&guess) {
                    found.push(guess);
                }
            }
        }
        found
    }

    pub(crate) fn ignore(&mut self, word: String) {
        self.ignored.insert(word);
    }

    /// Add `word` to `lang`'s user dictionary file and to the loaded dictionary.
    pub(crate) fn learn(&mut self, lang: &Lang, word: &str) -> Learned {
        let saved = self.append(lang, word);
        for installed in self.books(std::slice::from_ref(lang)) {
            if let Some(Some(dictionary)) = self.loaded.get_mut(&installed) {
                let _ = dictionary.add(word);
            }
        }
        match saved {
            Ok(()) => Learned::Saved,
            Err(_) => {
                self.ignored.insert(word.to_owned());
                Learned::SessionOnly
            }
        }
    }

    fn accepts(&self, lang: &Lang, word: &str) -> bool {
        matches!(self.loaded.get(lang), Some(Some(dictionary)) if dictionary.check(word))
    }

    /// The installed dictionaries for `langs`, loading each on first use.
    fn books(&mut self, langs: &[Lang]) -> Vec<Lang> {
        let installed = self.installed();
        let chosen: Vec<Lang> = langs
            .iter()
            .filter_map(|lang| pick(lang, &installed))
            .collect();
        for lang in &chosen {
            if !self.loaded.contains_key(lang) {
                let loaded = self.load(lang);
                self.loaded.insert(lang.clone(), loaded);
            }
        }
        chosen
            .into_iter()
            .filter(|lang| matches!(self.loaded.get(lang), Some(Some(_))))
            .collect()
    }

    fn installed(&mut self) -> Vec<Lang> {
        let config = &self.config;
        self.installed
            .get_or_insert_with(|| config.installed())
            .clone()
    }

    fn load(&self, lang: &Lang) -> Option<Dictionary> {
        let (aff, dic) = self.config.files(lang)?;
        let mut dictionary = Dictionary::new(
            &fs::read_to_string(aff).ok()?,
            &fs::read_to_string(dic).ok()?,
        )
        .ok()?;
        for word in self.user_words(lang) {
            let _ = dictionary.add(&word);
        }
        Some(dictionary)
    }

    /// The user's learned words for the installed dictionary `lang`, one per line.
    fn user_words(&self, lang: &Lang) -> Vec<String> {
        fs::read_to_string(self.user_file(lang))
            .map(|text| {
                text.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn append(&mut self, lang: &Lang, word: &str) -> std::io::Result<()> {
        let installed = self.installed();
        let file = self.user_file(&pick(lang, &installed).unwrap_or_else(|| lang.clone()));
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut out = OpenOptions::new().create(true).append(true).open(file)?;
        writeln!(out, "{word}")
    }

    fn user_file(&self, lang: &Lang) -> PathBuf {
        self.config.user.join(format!("{}.dic", lang.name()))
    }
}
