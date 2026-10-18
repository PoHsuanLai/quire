//! The install command for a package, as text a person can copy into a terminal: what to show
//! where nothing can install it for them (no PackageKit, as on a stock Arch).

use crate::capability::PackageName;
use crate::catalog::Entry;
use crate::family::Family;
use std::fmt;

/// One distro family's install command for one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallLine {
    /// The family whose package manager runs it.
    pub family: Family,
    /// The package it installs.
    pub package: PackageName,
}

impl InstallLine {
    /// The command, ready to paste: `sudo pacman -S mpv`.
    pub fn text(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for InstallLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.family.install_command(), self.package)
    }
}

/// The lines for `entry`: the first candidate package on the distro's family, or, where the
/// family is not known, the first candidate of every family the file names one for. The first
/// candidate only, since the alternatives are exclusive: the first that exists wins.
pub(crate) fn install_lines(entry: &Entry, family: Option<Family>) -> Vec<InstallLine> {
    let families = family.map_or_else(|| Family::ALL.to_vec(), |family| vec![family]);
    families
        .into_iter()
        .filter_map(|family| {
            let package = entry.candidates(family).first()?.clone();
            Some(InstallLine { family, package })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::install_lines;
    use crate::capability::Capability;
    use crate::catalog::Catalog;
    use crate::family::Family;

    const FILE: &str = r#"
[video-playback]
tool = "mpv"
purpose = "play videos"
probe = ["mpv"]
[video-playback.packages]
dnf = ["mpv", "mpv-libs"]
apt = ["mpv"]
pacman = ["mpv"]

[heic-decode]
tool = "heif-dec"
purpose = "open HEIC photos"
probe = ["heif-dec"]
[heic-decode.packages]
dnf = ["libheif-tools"]
apt = ["libheif-examples"]
pacman = ["libheif"]
"#;

    fn texts(capability: &str, family: Option<Family>) -> Vec<String> {
        let catalog = Catalog::parse(FILE).expect("parses");
        let entry = catalog
            .get(&Capability::new(capability).expect("capability"))
            .expect("declared");
        install_lines(entry, family)
            .iter()
            .map(|line| line.text())
            .collect()
    }

    #[test]
    fn each_family_gets_its_own_command_with_the_first_candidate() {
        let table: &[(&str, &str, Option<Family>, &[&str])] = &[
            (
                "fedora",
                "video-playback",
                Some(Family::Dnf),
                &["sudo dnf install mpv"],
            ),
            (
                "ubuntu",
                "heic-decode",
                Some(Family::Apt),
                &["sudo apt install libheif-examples"],
            ),
            (
                "arch",
                "heic-decode",
                Some(Family::Pacman),
                &["sudo pacman -S libheif"],
            ),
            (
                "suse without an entry",
                "heic-decode",
                Some(Family::Zypper),
                &[],
            ),
            (
                "unknown distro lists every family",
                "heic-decode",
                None,
                &[
                    "sudo dnf install libheif-tools",
                    "sudo apt install libheif-examples",
                    "sudo pacman -S libheif",
                ],
            ),
        ];
        for (name, capability, family, want) in table {
            assert_eq!(texts(capability, *family), *want, "{name}");
        }
    }
}
