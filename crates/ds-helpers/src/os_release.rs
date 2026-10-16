//! What `/etc/os-release` says about the distro: its `ID`, `ID_LIKE` and `VERSION_ID`.

/// The fields of an `os-release` text a program picks a distro and its release by.
///
/// The text is the file's own (read through `Environment::os_release`), parsed as the shell that
/// sources it would see these keys: one `KEY=value` per line, the value optionally in single or
/// double quotes, `#` lines and lines of other shapes ignored, the last assignment of a key
/// winning. Nothing here reads the file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OsRelease {
    /// `ID`, lower case: `fedora`, `ubuntu`, `debian`, `arch`, `opensuse-tumbleweed`. Empty when
    /// the text names none.
    pub id: String,
    /// `ID_LIKE`, lower case, one entry per word, in the file's order: the distros this one is
    /// derived from (`debian` on Ubuntu). Empty when the text has none.
    pub id_like: Vec<String>,
    /// `VERSION_ID`: `44`, `24.04`, `13`, a Tumbleweed snapshot date. `None` on a rolling or
    /// testing release that carries none (Arch, Debian testing) and when the value is empty.
    pub version_id: Option<String>,
}

impl OsRelease {
    /// The fields of `text`, an `os-release` file's contents.
    pub fn parse(text: &str) -> OsRelease {
        let value = |key: &str| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.starts_with('#'))
                .filter_map(|line| line.split_once('='))
                .filter(|(name, _)| name.trim() == key)
                .map(|(_, value)| unquoted(value))
                .next_back()
                .unwrap_or_default()
        };
        OsRelease {
            id: value("ID").to_ascii_lowercase(),
            id_like: value("ID_LIKE")
                .split_whitespace()
                .map(str::to_ascii_lowercase)
                .collect(),
            version_id: Some(value("VERSION_ID"))
                .filter(|version| !version.is_empty())
                .map(str::to_owned),
        }
    }
}

/// `value` without the whitespace around it or the one pair of quotes that wraps it.
fn unquoted(value: &str) -> &str {
    value.trim().trim_matches(|c| c == '"' || c == '\'')
}

#[cfg(test)]
mod tests {
    use super::OsRelease;

    const FEDORA_44: &str = r#"NAME="Fedora Linux"
VERSION="44 (Workstation Edition)"
RELEASE_TYPE=stable
ID=fedora
VERSION_ID=44
VERSION_CODENAME=""
PLATFORM_ID="platform:f44"
PRETTY_NAME="Fedora Linux 44 (Workstation Edition)"
ANSI_COLOR="0;38;2;60;110;180"
LOGO=fedora-logo-icon
CPE_NAME="cpe:/o:fedoraproject:fedora:44"
DEFAULT_HOSTNAME="fedora"
HOME_URL="https://fedoraproject.org/"
SUPPORT_URL="https://ask.fedoraproject.org/"
BUG_REPORT_URL="https://bugzilla.redhat.com/"
VARIANT="Workstation Edition"
VARIANT_ID=workstation
"#;

    const UBUNTU_2404: &str = r#"PRETTY_NAME="Ubuntu 24.04.2 LTS"
NAME="Ubuntu"
VERSION_ID="24.04"
VERSION="24.04.2 LTS (Noble Numbat)"
VERSION_CODENAME=noble
ID=ubuntu
ID_LIKE=debian
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
UBUNTU_CODENAME=noble
LOGO=ubuntu-logo
"#;

    const UBUNTU_2604: &str = r#"PRETTY_NAME="Ubuntu 26.04 LTS"
NAME="Ubuntu"
VERSION_ID="26.04"
VERSION="26.04 LTS (Resolute Raccoon)"
VERSION_CODENAME=resolute
ID=ubuntu
ID_LIKE=debian
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
UBUNTU_CODENAME=resolute
LOGO=ubuntu-logo
"#;

    const DEBIAN_13: &str = r#"PRETTY_NAME="Debian GNU/Linux 13 (trixie)"
NAME="Debian GNU/Linux"
VERSION_ID="13"
VERSION="13 (trixie)"
VERSION_CODENAME=trixie
DEBIAN_VERSION_FULL=13.0
ID=debian
HOME_URL="https://www.debian.org/"
SUPPORT_URL="https://www.debian.org/support"
BUG_REPORT_URL="https://bugs.debian.org/"
"#;

    /// Debian 14 while it is still testing: no `VERSION_ID`, the codename in `PRETTY_NAME`.
    const DEBIAN_14_FORKY: &str = r#"PRETTY_NAME="Debian GNU/Linux forky/sid"
NAME="Debian GNU/Linux"
VERSION_CODENAME=forky
ID=debian
HOME_URL="https://www.debian.org/"
SUPPORT_URL="https://www.debian.org/support"
BUG_REPORT_URL="https://bugs.debian.org/"
"#;

    const ARCH: &str = r#"NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
ID=arch
BUILD_ID=rolling
ANSI_COLOR="38;2;23;147;209"
HOME_URL="https://archlinux.org/"
DOCUMENTATION_URL="https://wiki.archlinux.org/"
SUPPORT_URL="https://bbs.archlinux.org/"
BUG_REPORT_URL="https://gitlab.archlinux.org/groups/archlinux/-/issues"
PRIVACY_POLICY_URL="https://terms.archlinux.org/docs/privacy-policy/"
LOGO=archlinux-logo
"#;

    const TUMBLEWEED: &str = r#"NAME="openSUSE Tumbleweed"
# VERSION="20261012"
ID="opensuse-tumbleweed"
ID_LIKE="opensuse suse"
VERSION_ID="20261012"
PRETTY_NAME="openSUSE Tumbleweed"
ANSI_COLOR="0;32"
CPE_NAME="cpe:/o:opensuse:tumbleweed:20261012"
BUG_REPORT_URL="https://bugzilla.opensuse.org"
SUPPORT_URL="https://bugs.opensuse.org"
HOME_URL="https://www.opensuse.org/"
DOCUMENTATION_URL="https://en.opensuse.org/Portal:Tumbleweed"
LOGO=distributor-logo-Tumbleweed
"#;

    fn release(id: &str, id_like: &[&str], version_id: Option<&str>) -> OsRelease {
        OsRelease {
            id: id.to_owned(),
            id_like: id_like.iter().map(|word| (*word).to_owned()).collect(),
            version_id: version_id.map(str::to_owned),
        }
    }

    #[test]
    fn real_os_release_texts_give_the_id_and_the_version() {
        let table: &[(&str, &str, OsRelease)] = &[
            ("Fedora 44", FEDORA_44, release("fedora", &[], Some("44"))),
            (
                "Ubuntu 24.04",
                UBUNTU_2404,
                release("ubuntu", &["debian"], Some("24.04")),
            ),
            (
                "Ubuntu 26.04",
                UBUNTU_2604,
                release("ubuntu", &["debian"], Some("26.04")),
            ),
            ("Debian 13", DEBIAN_13, release("debian", &[], Some("13"))),
            (
                "Debian 14 (forky, testing: no VERSION_ID)",
                DEBIAN_14_FORKY,
                release("debian", &[], None),
            ),
            (
                "Arch (rolling: no VERSION_ID)",
                ARCH,
                release("arch", &[], None),
            ),
            (
                "openSUSE Tumbleweed (a commented VERSION is not read)",
                TUMBLEWEED,
                release(
                    "opensuse-tumbleweed",
                    &["opensuse", "suse"],
                    Some("20261012"),
                ),
            ),
        ];
        for (name, text, want) in table {
            assert_eq!(&OsRelease::parse(text), want, "{name}");
        }
    }

    #[test]
    fn the_shapes_a_file_may_take() {
        let table: &[(&str, &str, OsRelease)] = &[
            ("empty", "", OsRelease::default()),
            (
                "single quotes, spaces and CRLF",
                " ID = 'Fedora'\r\n VERSION_ID='44'\r\n",
                release("fedora", &[], Some("44")),
            ),
            (
                "an empty VERSION_ID is none",
                "ID=arch\nVERSION_ID=\"\"\n",
                release("arch", &[], None),
            ),
            (
                "the last assignment wins, as when the shell sources the file",
                "ID=one\nID=two\n",
                release("two", &[], None),
            ),
            (
                "a key that only ends in VERSION_ID is not it",
                "ID=x\nBUILD_VERSION_ID=9\n",
                release("x", &[], None),
            ),
        ];
        for (name, text, want) in table {
            assert_eq!(&OsRelease::parse(text), want, "{name}");
        }
    }
}
