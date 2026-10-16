//! The distro family, which decides whose package names to try.

use crate::os_release::OsRelease;

/// A package-name family: the distros that share one set of package names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Fedora, RHEL and their relatives.
    Dnf,
    /// Debian, Ubuntu and their relatives.
    Apt,
    /// Arch and its relatives.
    Pacman,
    /// openSUSE and SUSE Linux Enterprise.
    Zypper,
}

impl Family {
    /// Every family, in the order a list of them reads.
    pub const ALL: [Family; 4] = [Family::Dnf, Family::Apt, Family::Pacman, Family::Zypper];

    /// The key the helpers file uses for this family.
    pub const fn key(self) -> &'static str {
        match self {
            Family::Dnf => "dnf",
            Family::Apt => "apt",
            Family::Pacman => "pacman",
            Family::Zypper => "zypper",
        }
    }

    /// The command that installs packages on this family, up to the package names, as typed in a
    /// terminal.
    pub const fn install_command(self) -> &'static str {
        match self {
            Family::Dnf => "sudo dnf install",
            Family::Apt => "sudo apt install",
            Family::Pacman => "sudo pacman -S",
            Family::Zypper => "sudo zypper install",
        }
    }

    /// The family one `os-release` id belongs to, if it is one we know.
    fn of_id(id: &str) -> Option<Family> {
        match id {
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" | "nobara" | "bazzite" => {
                Some(Family::Dnf)
            }
            "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" | "raspbian" => {
                Some(Family::Apt)
            }
            "arch" | "manjaro" | "endeavouros" | "cachyos" | "steamos" => Some(Family::Pacman),
            "suse" | "sles" | "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => {
                Some(Family::Zypper)
            }
            _ => None,
        }
    }

    /// The family of the distro an `os-release` file describes: `ID` first, then each `ID_LIKE`
    /// entry in order (see [`OsRelease`]). `None` when none of them is known.
    pub fn detect(os_release: &str) -> Option<Family> {
        let release = OsRelease::parse(os_release);
        std::iter::once(release.id.as_str())
            .chain(release.id_like.iter().map(String::as_str))
            .find_map(Family::of_id)
    }
}

#[cfg(test)]
mod tests {
    use super::Family;

    #[test]
    fn os_release_resolves_to_a_family() {
        let table: &[(&str, &str, Option<Family>)] = &[
            ("fedora", "ID=fedora\nVERSION_ID=44\n", Some(Family::Dnf)),
            (
                "rhel clone",
                "ID=\"almalinux\"\nID_LIKE=\"rhel centos fedora\"\n",
                Some(Family::Dnf),
            ),
            (
                "ubuntu",
                "NAME=Ubuntu\nID=ubuntu\nID_LIKE=debian\n",
                Some(Family::Apt),
            ),
            ("debian", "ID=debian\n", Some(Family::Apt)),
            (
                "derivative by like",
                "ID=zorin\nID_LIKE=\"ubuntu debian\"\n",
                Some(Family::Apt),
            ),
            ("arch", "ID=arch\n", Some(Family::Pacman)),
            (
                "manjaro",
                "ID=manjaro\nID_LIKE=arch\n",
                Some(Family::Pacman),
            ),
            (
                "opensuse",
                "ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n",
                Some(Family::Zypper),
            ),
            (
                "leap",
                "ID=\"opensuse-leap\"\nID_LIKE=\"suse opensuse\"\n",
                Some(Family::Zypper),
            ),
            ("unknown", "ID=nixos\n", None),
            (
                "unknown with like",
                "ID=weird\nID_LIKE=\"other thing\"\n",
                None,
            ),
            ("empty", "", None),
        ];
        for (name, text, want) in table {
            assert_eq!(Family::detect(text), *want, "{name}");
        }
    }

    #[test]
    fn the_id_wins_over_id_like() {
        assert_eq!(
            Family::detect("ID=arch\nID_LIKE=debian\n"),
            Some(Family::Pacman)
        );
    }
}
