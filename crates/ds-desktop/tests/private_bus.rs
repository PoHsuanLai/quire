//! The probe against a private `dbus-daemon`: a fake service claims a name and releases it.
//! Nothing here reaches the real session or system bus; the guard kills the daemon by PID.

use ds_desktop::{Capability, Desktop};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct PrivateBus {
    child: Child,
    dir: PathBuf,
    address: String,
}

impl PrivateBus {
    /// A bus whose `services` directory holds one activation file per name in `activatable`.
    fn start(tag: &str, activatable: &[&str]) -> PrivateBus {
        let dir = std::env::temp_dir().join(format!("ds-desktop-bus-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).expect("scratch dir");
        std::fs::create_dir_all(dir.join("services")).expect("services dir");
        for name in activatable {
            std::fs::write(
                dir.join("services").join(format!("{name}.service")),
                format!("[D-BUS Service]\nName={name}\nExec=/bin/false\n"),
            )
            .expect("service file");
        }
        let socket = dir.join("bus");
        let config = dir.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<busconfig><type>session</type><listen>unix:path={}</listen>\
                 <servicedir>{}</servicedir><auth>EXTERNAL</auth>\
                 <policy context=\"default\"><allow send_destination=\"*\" eavesdrop=\"true\"/>\
                 <allow eavesdrop=\"true\"/><allow own=\"*\"/></policy></busconfig>",
                socket.display(),
                dir.join("services").display()
            ),
        )
        .expect("bus config");
        let mut child = Command::new("dbus-daemon")
            .env_clear()
            .env("HOME", dir.join("home"))
            .env("XDG_RUNTIME_DIR", &dir)
            .env("XDG_CONFIG_HOME", dir.join("home"))
            .arg("--config-file")
            .arg(&config)
            .arg("--nofork")
            .arg("--print-address=1")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dbus-daemon starts");
        let stdout = child.stdout.take().expect("piped stdout");
        let mut address = String::new();
        BufReader::new(stdout)
            .read_line(&mut address)
            .expect("the daemon prints its address");
        PrivateBus {
            child,
            dir,
            address: address.trim().to_owned(),
        }
    }

    async fn connect(&self) -> zbus::Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .expect("a unix address")
            .build()
            .await
            .expect("connects to the private bus")
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const WAIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn an_owned_name_is_here_and_releasing_it_flips_to_absent() {
    let bus = PrivateBus::start("owned", &[]);
    let client = bus.connect().await;
    let service = bus.connect().await;

    let before = Desktop::probe_on(Some(&client), None).await;
    assert!(Capability::ALL.iter().all(|&c| !before.here(c)));

    service
        .request_name("org.quire.Memory1")
        .await
        .expect("claims the name");
    let after = Desktop::probe_on(Some(&client), None).await;
    for capability in Capability::ALL {
        assert_eq!(after.here(capability), capability == Capability::Memory);
    }

    let mut watch = Desktop::watch_on(Some(&client), None).await;
    assert!(watch.current().here(Capability::Memory));
    service
        .release_name("org.quire.Memory1")
        .await
        .expect("releases the name");
    let gone = tokio::time::timeout(WAIT, watch.changed())
        .await
        .expect("a change arrives")
        .expect("the feed is live");
    assert!(!gone.here(Capability::Memory));

    service
        .request_name("org.quire.Memory1")
        .await
        .expect("claims it again");
    let back = tokio::time::timeout(WAIT, watch.changed())
        .await
        .expect("a change arrives")
        .expect("the feed is live");
    assert!(back.here(Capability::Memory));
}

#[tokio::test]
async fn an_activatable_name_is_here_before_anyone_owns_it() {
    let bus = PrivateBus::start("activatable", &["org.quire.Intents1"]);
    let client = bus.connect().await;
    let desktop = Desktop::probe_on(Some(&client), None).await;
    assert!(desktop.here(Capability::Intents));
    assert!(!desktop.here(Capability::Memory));
}

#[tokio::test]
async fn a_missing_bus_leaves_its_capabilities_absent() {
    let bus = PrivateBus::start("system", &[]);
    let session = bus.connect().await;
    let desktop = Desktop::probe_on(Some(&session), None).await;
    assert!(!desktop.here(Capability::Helpers));
    assert!(
        Desktop::watch_on(None, None)
            .await
            .changed()
            .await
            .is_none()
    );
}

#[tokio::test]
async fn the_system_bus_carries_packagekit() {
    let session_bus = PrivateBus::start("sess", &[]);
    let system_bus = PrivateBus::start("sys", &[]);
    let session = session_bus.connect().await;
    let system = system_bus.connect().await;
    let daemon = system_bus.connect().await;
    // A name on the session bus does not count as PackageKit.
    let _decoy = session_bus.connect().await;
    daemon
        .request_name("org.freedesktop.PackageKit")
        .await
        .expect("claims the name");
    let desktop = Desktop::probe_on(Some(&session), Some(&system)).await;
    assert!(desktop.here(Capability::Helpers));
}
