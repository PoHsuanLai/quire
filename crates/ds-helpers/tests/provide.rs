//! `Helpers::provide` and the availability feed over a fake installer and scratch directories.
//! The example data file is anyview's four capabilities (names are fixtures, not claims).

use ds_helpers::{
    Capability, Catalog, Environment, Executable, FakeInstaller, Family, Helpers, Installer,
    Missing, Outcome, PackageName, Presence, StandIn,
};
use std::path::{Path, PathBuf};

/// The four capabilities anyview declares, as an app would ship them.
const ANYVIEW: &str = r#"
[video-playback]
tool = "mpv"
purpose = "play videos"
probe = ["mpv"]
[video-playback.packages]
dnf = ["mpv"]
apt = ["mpv"]
pacman = ["mpv"]
zypper = ["mpv"]

[media-probe]
tool = "FFmpeg"
purpose = "read video details"
probe = ["ffprobe"]
[media-probe.packages]
dnf = ["ffmpeg", "ffmpeg-free"]
apt = ["ffmpeg"]
pacman = ["ffmpeg"]
zypper = ["ffmpeg"]

[heic-decode]
tool = "libheif tools"
purpose = "open HEIC photos"
probe = ["heif-dec", "heif-convert"]
[heic-decode.packages]
dnf = ["libheif-tools"]
apt = ["libheif-examples"]
pacman = ["libheif"]

[raw-decode]
tool = "LibRaw tools"
purpose = "open camera RAW photos"
probe = ["dcraw_emu"]
# Example fixture: package names per family are not verified here.
[raw-decode.packages]
dnf = ["LibRaw-samples"]
apt = ["libraw-bin"]
pacman = ["libraw"]
"#;

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(tag: &str, os_release: &str) -> Scratch {
        let root = std::env::temp_dir().join(format!("ds-helpers-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("bin")).expect("scratch");
        std::fs::write(root.join("os-release"), os_release).expect("os-release");
        Scratch { root }
    }
    fn bin(&self) -> PathBuf {
        self.root.join("bin")
    }
    fn environment(&self) -> Environment {
        Environment {
            path: self.bin().into(),
            os_release: self.root.join("os-release"),
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn cap(name: &str) -> Capability {
    Capability::new(name).expect("capability")
}

fn missing(capability: &str, package: Option<&str>, program: Option<&str>) -> Missing {
    Missing {
        capability: cap(capability),
        package: package.map(|name| PackageName::new(name).expect("package")),
        program: program.map(|name| Executable::new(name).expect("program")),
    }
}

fn helpers(scratch: &Scratch, installer: FakeInstaller) -> Helpers {
    let catalog = Catalog::parse(ANYVIEW).expect("the example file parses");
    Helpers::new(catalog, scratch.environment(), Installer::Fake(installer))
}

#[test]
fn the_example_file_declares_four_capabilities_with_candidates() {
    let catalog = Catalog::parse(ANYVIEW).expect("parses");
    assert_eq!(catalog.capabilities().count(), 4);
    let media = catalog.get(&cap("media-probe")).expect("declared");
    assert_eq!(media.candidates(Family::Dnf).len(), 2);
}

#[tokio::test]
async fn provide_installs_then_reprobes_and_announces() {
    let scratch = Scratch::new("install", "ID=fedora\n");
    let fake = FakeInstaller::new(Outcome::Installed).leaving(scratch.bin(), &["mpv"]);
    let helpers = helpers(&scratch, fake);
    let video = cap("video-playback");
    assert_eq!(helpers.available(&video), Presence::Missing);
    let mut feed = helpers.subscribe();
    assert_eq!(helpers.provide(&video).await, Outcome::Installed);
    let change = feed.next().await;
    assert_eq!(
        (change.capability, change.presence),
        (video.clone(), Presence::Present)
    );
    assert_eq!(helpers.available(&video), Presence::Present);
    // Already there: no second request.
    assert_eq!(helpers.provide(&video).await, Outcome::Installed);
}

#[tokio::test]
async fn the_request_carries_this_distros_alternatives() {
    let scratch = Scratch::new("request", "ID=fedora\nID_LIKE=\"rhel\"\n");
    let fake = FakeInstaller::new(Outcome::NotFound(missing("x", None, None)));
    let helpers = helpers(&scratch, fake.clone());
    assert_eq!(
        helpers.provide(&cap("media-probe")).await,
        Outcome::NotFound(missing("media-probe", Some("ffmpeg"), Some("ffprobe")))
    );
    let asked = fake.asked();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].family, Family::Dnf);
    let names: Vec<_> = asked[0].candidates.iter().map(|p| p.as_str()).collect();
    assert_eq!(names, ["ffmpeg", "ffmpeg-free"]);
    assert_eq!(helpers.family(), Some(Family::Dnf));
}

#[tokio::test]
async fn outcomes_pass_through() {
    for outcome in [
        Outcome::Declined,
        Outcome::NotFound(missing("video-playback", Some("mpv"), Some("mpv"))),
        Outcome::Failed("disk full".to_owned()),
    ] {
        let scratch = Scratch::new("pass", "ID=ubuntu\n");
        let helpers = helpers(&scratch, FakeInstaller::new(outcome.clone()));
        assert_eq!(helpers.provide(&cap("video-playback")).await, outcome);
    }
}

#[tokio::test]
async fn unknown_distro_or_family_gap_or_capability_is_unsupported() {
    let table = [
        (
            "unknown distro",
            "ID=nixos\n",
            "video-playback",
            None,
            Some("mpv"),
        ),
        (
            "no zypper entry",
            "ID=opensuse-leap\n",
            "heic-decode",
            None,
            Some("heif-dec"),
        ),
        ("undeclared", "ID=fedora\n", "teleport", None, None),
    ];
    for (name, os_release, capability, package, program) in table {
        let scratch = Scratch::new("unsupported", os_release);
        let helpers = helpers(&scratch, FakeInstaller::new(Outcome::Installed));
        assert_eq!(
            helpers.provide(&cap(capability)).await,
            Outcome::Unsupported(missing(capability, package, program)),
            "{name}"
        );
    }
}

#[tokio::test]
async fn an_install_that_leaves_no_tool_is_a_failure() {
    let scratch = Scratch::new("ghost", "ID=fedora\n");
    let helpers = helpers(&scratch, FakeInstaller::new(Outcome::Installed));
    let outcome = helpers.provide(&cap("video-playback")).await;
    assert!(matches!(outcome, Outcome::Failed(why) if why.contains("mpv")));
}

#[tokio::test]
async fn refresh_hears_of_a_tool_installed_some_other_way() {
    let scratch = Scratch::new("refresh", "ID=fedora\n");
    let helpers = helpers(&scratch, FakeInstaller::new(Outcome::Declined));
    let heic = cap("heic-decode");
    let mut feed = helpers.subscribe();
    assert_eq!(helpers.refresh(&heic), Presence::Missing);
    let file = scratch.bin().join("heif-convert");
    std::fs::write(&file, "#!/bin/sh\n").expect("tool");
    set_executable(&file);
    assert_eq!(helpers.refresh(&heic), Presence::Present);
    assert_eq!(feed.next().await.presence, Presence::Present);
    std::fs::remove_file(&file).expect("removed");
    assert_eq!(helpers.refresh(&heic), Presence::Missing);
    assert_eq!(feed.next().await.presence, Presence::Missing);
}

fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("mode");
}

#[tokio::test]
async fn a_stand_in_tool_keeps_its_script() {
    let scratch = Scratch::new("standin", "ID=fedora\n");
    let script = "#!/bin/sh\necho 1.2.3\n".to_owned();
    let tool = StandIn {
        name: "mpv".to_owned(),
        body: script.clone(),
    };
    let fake = FakeInstaller::new(Outcome::Installed).leaving_with(scratch.bin(), vec![tool]);
    let helpers = helpers(&scratch, fake);
    assert_eq!(
        helpers.provide(&cap("video-playback")).await,
        Outcome::Installed
    );
    assert_eq!(
        std::fs::read_to_string(scratch.bin().join("mpv")).expect("tool"),
        script
    );
}
