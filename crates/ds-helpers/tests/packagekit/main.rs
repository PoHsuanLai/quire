//! The PackageKit backend against a fake PackageKit on a private bus: Installed, Declined,
//! NotFound, plus the failure and absent-daemon cases. Nothing here reaches the real system or
//! session bus, and nothing is installed.

mod bus;
mod fake_pk;

use bus::PrivateBus;
use ds_helpers::{Family, Installer, Outcome, PackageKit, PackageName, Request};
use fake_pk::{Behaviour, Calls, serve};

fn request(names: &[&str]) -> Request {
    Request {
        family: Family::Dnf,
        candidates: names
            .iter()
            .map(|name| PackageName::new(name).expect("a package name"))
            .collect(),
    }
}

/// A fake daemon with `behaviour` and a client connection to the same private bus.
async fn setup(tag: &str, behaviour: Behaviour) -> (PrivateBus, Installer, Calls) {
    let bus = PrivateBus::start(tag);
    let daemon = bus.connect().await;
    let calls = serve(&daemon, behaviour).await;
    // The daemon connection must outlive the test: leak it into the guard's scope by
    // keeping it inside the installer's bus (it is the same private bus, so a second
    // connection is the client).
    std::mem::forget(daemon);
    let client = bus.connect().await;
    (bus, Installer::PackageKit(PackageKit::on(client)), calls)
}

#[tokio::test]
async fn installs_the_first_candidate_that_exists() {
    let behaviour = Behaviour::default().available(&["ffmpeg-free"]);
    let (_bus, installer, calls) = setup("first", behaviour).await;
    let outcome = installer
        .install(&request(&["ffmpeg", "ffmpeg-free"]))
        .await;
    assert_eq!(outcome, Outcome::Installed);
    assert_eq!(calls.installed(), ["ffmpeg-free;1.0;x86_64;fedora"]);
    assert_eq!(calls.resolved(), ["ffmpeg", "ffmpeg-free"]);
    assert_eq!(calls.hints(), ["interactive=true"; 3]);
    assert_eq!(calls.flags(), [1 << 1]);
}

#[tokio::test]
async fn an_already_installed_package_installs_nothing() {
    let behaviour = Behaviour::default().already(&["mpv"]);
    let (_bus, installer, calls) = setup("already", behaviour).await;
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::Installed
    );
    assert!(calls.installed().is_empty());
}

#[tokio::test]
async fn a_cancelled_password_prompt_is_declined() {
    let behaviour = Behaviour::default().available(&["mpv"]).refuse_policy();
    let (_bus, installer, calls) = setup("declined", behaviour).await;
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::Declined
    );
    assert!(calls.installed().is_empty());
}

#[tokio::test]
async fn a_transaction_cancelled_after_auth_is_declined() {
    let behaviour = Behaviour::default()
        .available(&["mpv"])
        .cancel_transaction();
    let (_bus, installer, _calls) = setup("cancelled", behaviour).await;
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::Declined
    );
}

#[tokio::test]
async fn no_candidate_in_the_repositories_is_not_found() {
    let (_bus, installer, calls) =
        setup("absent", Behaviour::default().available(&["other"])).await;
    let outcome = installer.install(&request(&["mpv", "mpv-nox"])).await;
    assert_eq!(outcome, Outcome::NotFound);
    assert!(calls.installed().is_empty());
}

#[tokio::test]
async fn a_package_not_found_error_counts_as_absent() {
    let behaviour = Behaviour::default().resolve_errors(8, "no such package");
    let (_bus, installer, _calls) = setup("notfound-err", behaviour).await;
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::NotFound
    );
}

#[tokio::test]
async fn an_install_error_carries_the_daemons_words() {
    let behaviour = Behaviour::default()
        .available(&["mpv"])
        .fail_install("no network");
    let (_bus, installer, _calls) = setup("failed", behaviour).await;
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::Failed("no network".to_owned())
    );
}

/// PackageKit's own error numbers (PackageKitGlib-1.0: 17 transaction-cancelled, 48
/// not-authorized, 23 failed-finalise): the first two are the person saying no, the third is not.
#[tokio::test]
async fn install_error_codes_map_to_outcomes() {
    let cases = [
        (17, "cancelled", Outcome::Declined),
        (48, "not authorized", Outcome::Declined),
        (
            23,
            "could not finalise",
            Outcome::Failed("could not finalise".to_owned()),
        ),
    ];
    for (code, text, want) in cases {
        let behaviour = Behaviour::default()
            .available(&["mpv"])
            .install_errors(code, text);
        let (_bus, installer, _calls) = setup(&format!("code-{code}"), behaviour).await;
        assert_eq!(
            installer.install(&request(&["mpv"])).await,
            want,
            "error {code}"
        );
    }
}

#[tokio::test]
async fn no_packagekit_on_the_bus_is_unsupported() {
    let bus = PrivateBus::start("nopk");
    let client = bus.connect().await;
    let installer = Installer::PackageKit(PackageKit::on(client));
    assert_eq!(
        installer.install(&request(&["mpv"])).await,
        Outcome::Unsupported
    );
}
