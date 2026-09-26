//! A tiny Hunspell dictionary written into a fresh temporary directory, so the spelling tests
//! never depend on what the machine has installed.

use std::path::PathBuf;

/// The test dictionary's affix file: UTF-8, and the letters suggestions may try.
const AFF: &str = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'\n";

/// Its words.
const DIC: &str = "8\nthe\ncat\nsat\non\nmat\nhello\nworld\nquick\n";

/// A new directory holding `en_US.aff` and `en_US.dic`, unique to `name` and this process.
pub fn dictionary(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("quire-spell-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temporary directory");
    std::fs::write(dir.join("en_US.aff"), AFF).expect("the affix file");
    std::fs::write(dir.join("en_US.dic"), DIC).expect("the word list");
    dir
}
