//! The spellchecker's worker (`ds_native::spell`, feature `spellcheck`) against a tiny test
//! dictionary: checking, suggestions, "Ignore Spelling", "Learn Spelling" written to the user's
//! file and read back by a new checker, and a language with no dictionary. The system
//! dictionary is checked only where it is installed.

#[path = "support/spell_dict.rs"]
mod spell_dict;

use ds::{Lang, Learned, SpellService};
use ds_native::spell::{NativeSpell, SpellConfig};
use std::future::Future;
use std::path::Path;

fn en() -> Lang {
    Lang::parse("en_US").expect("a language")
}

fn block<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a runtime")
        .block_on(future)
}

fn checker(dir: &Path) -> NativeSpell {
    let config = SpellConfig {
        dictionaries: vec![dir.to_path_buf()],
        user: dir.join("user"),
    };
    NativeSpell::with_config(config, vec![en()])
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn the_worker_answers_which_words_are_wrong() {
    let spell = checker(&spell_dict::dictionary("check"));
    let wrong = block(spell.check(vec![en()], words(&["teh", "cat", "helo", "The", "sat"])));
    assert_eq!(wrong, words(&["teh", "helo"]));
}

#[test]
fn the_worker_suggests_replacements() {
    let spell = checker(&spell_dict::dictionary("suggest"));
    let guesses = block(spell.suggest(vec![en()], "teh".to_owned()));
    assert_eq!(
        guesses.first().map(String::as_str),
        Some("the"),
        "{guesses:?}"
    );
    let guesses = block(spell.suggest(vec![en()], "helo".to_owned()));
    assert!(guesses.contains(&"hello".to_owned()), "{guesses:?}");
}

#[test]
fn an_ignored_word_is_accepted_for_the_session_only() {
    let dir = spell_dict::dictionary("ignore");
    let spell = checker(&dir);
    assert_eq!(
        block(spell.check(vec![en()], words(&["teh"]))),
        words(&["teh"])
    );
    spell.ignore("teh".to_owned());
    assert_eq!(
        block(spell.check(vec![en()], words(&["teh"]))),
        Vec::<String>::new()
    );
    let next = checker(&dir);
    assert_eq!(
        block(next.check(vec![en()], words(&["teh"]))),
        words(&["teh"]),
        "a new session has forgotten it"
    );
    assert!(
        !dir.join("user").join("en_US.dic").exists(),
        "nothing written"
    );
}

#[test]
fn a_learned_word_is_written_to_the_users_dictionary_and_kept() {
    let dir = spell_dict::dictionary("learn");
    let spell = checker(&dir);
    assert_eq!(
        block(spell.check(vec![en()], words(&["quire"]))),
        words(&["quire"])
    );
    assert_eq!(block(spell.learn(en(), "quire".to_owned())), Learned::Saved);
    assert_eq!(
        block(spell.check(vec![en()], words(&["quire"]))),
        Vec::<String>::new()
    );
    let file =
        std::fs::read_to_string(dir.join("user").join("en_US.dic")).expect("the user's file");
    assert_eq!(file, "quire\n");
    let next = checker(&dir);
    assert_eq!(
        block(next.check(vec![en()], words(&["quire"]))),
        Vec::<String>::new(),
        "a new session reads it back"
    );
}

#[test]
fn a_language_with_no_dictionary_marks_nothing() {
    let spell = checker(&spell_dict::dictionary("missing"));
    let german = Lang::parse("de_DE").expect("a language");
    assert_eq!(
        block(spell.check(vec![german], words(&["teh", "Haus"]))),
        Vec::<String>::new()
    );
}

#[test]
fn a_region_falls_back_to_the_installed_dictionary() {
    let spell = checker(&spell_dict::dictionary("region"));
    let british = Lang::parse("en_GB").expect("a language");
    assert_eq!(
        block(spell.check(vec![british], words(&["teh", "cat"]))),
        words(&["teh"])
    );
}

/// The system's en_US, where it is installed (Fedora's `hunspell-en-US`); skipped otherwise.
#[test]
fn the_system_dictionary_checks_english_where_installed() {
    let system = SpellConfig::system();
    if system.files(&en()).is_none() {
        eprintln!(
            "skipped: no en_US Hunspell dictionary in {:?}",
            system.dictionaries
        );
        return;
    }
    let user = spell_dict::dictionary("system");
    let spell = NativeSpell::with_config(
        SpellConfig {
            dictionaries: system.dictionaries,
            user,
        },
        vec![en()],
    );
    let wrong = block(spell.check(
        vec![en()],
        words(&["recieve", "receive", "definately", "colour"]),
    ));
    assert_eq!(wrong, words(&["recieve", "definately", "colour"]));
}
