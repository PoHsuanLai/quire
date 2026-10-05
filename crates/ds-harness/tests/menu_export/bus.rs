//! A private `dbus-daemon` for the menu export test: a scratch config, a scratch socket, a
//! scratch HOME and XDG. It never reaches the session bus; the guard kills the daemon by PID.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct PrivateBus {
    child: Child,
    dir: PathBuf,
    pub address: String,
}

impl PrivateBus {
    pub fn start(tag: &str) -> PrivateBus {
        let dir =
            std::env::temp_dir().join(format!("ds-harness-menu-bus-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).expect("scratch dir");
        let socket = dir.join("bus");
        let config = dir.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<busconfig><type>session</type><listen>unix:path={}</listen>\
                 <auth>EXTERNAL</auth>\
                 <policy context=\"default\"><allow send_destination=\"*\" eavesdrop=\"true\"/>\
                 <allow eavesdrop=\"true\"/><allow own=\"*\"/></policy></busconfig>",
                socket.display()
            ),
        )
        .expect("bus config");
        // The environment is cleared: nothing of the real session (DBUS_SESSION_BUS_ADDRESS,
        // XDG_RUNTIME_DIR, HOME) reaches the daemon.
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
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
