//! The names a helpers file is keyed by.

use std::fmt;

/// What an app needs to do, in the app's own words: `video-playback`, `heic-decode`. Lowercase
/// ASCII letters, digits and hyphens, so it is a legal bare TOML key and never needs quoting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability(String);

/// A name that is not valid for the type it was offered as.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{kind} {name:?} is not valid: {why}")]
pub struct NameError {
    kind: &'static str,
    name: String,
    why: &'static str,
}

impl NameError {
    pub(crate) fn new(kind: &'static str, name: &str, why: &'static str) -> NameError {
        NameError {
            kind,
            name: name.to_owned(),
            why,
        }
    }
}

impl Capability {
    /// `name` as a capability.
    pub fn new(name: &str) -> Result<Capability, NameError> {
        let legal = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
        match (name.is_empty(), name.chars().all(legal)) {
            (true, _) => Err(NameError::new("capability", name, "it is empty")),
            (_, false) => Err(NameError::new(
                "capability",
                name,
                "use lowercase letters, digits and hyphens",
            )),
            _ => Ok(Capability(name.to_owned())),
        }
    }

    /// The name as written in the file.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A package name as the distro spells it (`libheif-tools`). PackageKit reads `@name` as a group
/// and `name;version;arch;repo` as an exact id, so neither, nor whitespace, is a plain name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName(String);

impl PackageName {
    /// `name` as a package name.
    pub fn new(name: &str) -> Result<PackageName, NameError> {
        let bad = |c: char| c.is_whitespace() || c == ';' || c == '/' || c.is_control();
        if name.is_empty() || name.starts_with('@') || name.chars().any(bad) {
            return Err(NameError::new(
                "package name",
                name,
                "it must be a plain package name",
            ));
        }
        Ok(PackageName(name.to_owned()))
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The file name of a program found by searching `PATH`: no directory part.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Executable(String);

impl Executable {
    /// `name` as an executable file name.
    pub fn new(name: &str) -> Result<Executable, NameError> {
        if name.is_empty() || name.contains('/') || name.chars().any(char::is_whitespace) {
            return Err(NameError::new(
                "executable",
                name,
                "it must be a bare file name",
            ));
        }
        Ok(Executable(name.to_owned()))
    }

    /// The file name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability, Executable, PackageName};

    #[test]
    fn capabilities_are_bare_keys() {
        let table: &[(&str, bool)] = &[
            ("video-playback", true),
            ("heic-decode", true),
            ("x264", true),
            ("", false),
            ("Video", false),
            ("a b", false),
            ("a.b", false),
        ];
        for (name, ok) in table {
            assert_eq!(Capability::new(name).is_ok(), *ok, "{name}");
        }
    }

    #[test]
    fn package_names_are_plain() {
        let table: &[(&str, bool)] = &[
            ("mpv", true),
            ("libheif-tools", true),
            ("LibRaw-utils", true),
            ("g++", true),
            ("@web-development", false),
            ("mpv;1.0;x86_64;fedora", false),
            ("two words", false),
            ("", false),
        ];
        for (name, ok) in table {
            assert_eq!(PackageName::new(name).is_ok(), *ok, "{name}");
        }
    }

    #[test]
    fn executables_have_no_directory() {
        assert!(Executable::new("mpv").is_ok());
        assert!(Executable::new("/usr/bin/mpv").is_err());
        assert!(Executable::new("").is_err());
    }
}
