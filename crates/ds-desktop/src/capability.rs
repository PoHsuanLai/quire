//! The capabilities and the bus names they mean.

/// Which bus a service lives on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    /// The per-login session bus.
    Session,
    /// The system bus.
    System,
}

/// A well-known D-Bus name on one bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    /// The bus that carries it.
    pub bus: Bus,
    /// The well-known name.
    pub name: &'static str,
}

const fn session(name: &'static str) -> Option<Service> {
    Some(Service {
        bus: Bus::Session,
        name,
    })
}

/// One extra the desktop adds to an app's portable core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Capability {
    /// intentd: the registry of what apps can do (`quire do`, the companion's actions).
    Intents,
    /// accountd: the session's one account store and consent sheet.
    Accounts,
    /// memoryd: one memory for every app.
    Memory,
    /// companiond: the one companion across all apps.
    Companion,
    /// The desktop broadcasts its appearance (accent, Space tint) beyond the portal.
    Appearance,
    /// Compositor materials: blur behind a surface.
    Materials,
    /// The shell's Quick Look peek of a file.
    Peek,
    /// The shell's share sheet.
    Share,
    /// PackageKit answers, so a missing helper can be installed on the spot.
    Helpers,
}

impl Capability {
    /// Every capability, in declaration order.
    pub const ALL: [Capability; 9] = [
        Capability::Intents,
        Capability::Accounts,
        Capability::Memory,
        Capability::Companion,
        Capability::Appearance,
        Capability::Materials,
        Capability::Peek,
        Capability::Share,
        Capability::Helpers,
    ];

    /// The D-Bus name that means this capability, or `None` where no service name is settled
    /// (such a capability is always `Absent` until one is).
    pub const fn service(self) -> Option<Service> {
        match self {
            // design/33-AGENT.md section 5 (intentd, docket repo).
            Capability::Intents => session("org.quire.Intents1"),
            // accountd, porter repo (porter/crates/accountd/src/lib.rs:2); also named by
            // ds-settings' live client docs.
            Capability::Accounts => session("org.quire.Accounts1"),
            // memoryd, design/33-AGENT.md section 5 (almanac repo).
            Capability::Memory => session("org.quire.Memory1"),
            // companiond, design/33-AGENT.md section 5 (docket repo).
            Capability::Companion => session("org.quire.Companion1"),
            Capability::Helpers => Some(Service {
                bus: Bus::System,
                name: "org.freedesktop.PackageKit",
            }),
            // TODO: no service name exists yet. Appearance today is the freedesktop settings
            // portal, which other desktops also serve, so it cannot tell ours apart.
            Capability::Appearance => None,
            // TODO: blur is a Wayland protocol (ext-background-effect-v1), not a bus service.
            Capability::Materials => None,
            // TODO: anyview serves `org.quire.Anyview1.Peek`, but the shell's peek endpoint has
            // no name in sill or shell-host yet.
            Capability::Peek => None,
            // TODO: no share sheet service is specified in any repo yet.
            Capability::Share => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bus, Capability};

    #[test]
    fn the_name_table() {
        const CASES: &[(Capability, Option<(Bus, &str)>)] = &[
            (
                Capability::Intents,
                Some((Bus::Session, "org.quire.Intents1")),
            ),
            (
                Capability::Accounts,
                Some((Bus::Session, "org.quire.Accounts1")),
            ),
            (
                Capability::Memory,
                Some((Bus::Session, "org.quire.Memory1")),
            ),
            (
                Capability::Companion,
                Some((Bus::Session, "org.quire.Companion1")),
            ),
            (
                Capability::Helpers,
                Some((Bus::System, "org.freedesktop.PackageKit")),
            ),
            (Capability::Appearance, None),
            (Capability::Materials, None),
            (Capability::Peek, None),
            (Capability::Share, None),
        ];
        assert_eq!(CASES.len(), Capability::ALL.len());
        for &(capability, want) in CASES {
            let got = capability.service().map(|s| (s.bus, s.name));
            assert_eq!(got, want, "{capability:?}");
        }
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = Capability::ALL
            .iter()
            .filter_map(|c| c.service().map(|s| s.name))
            .collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before);
    }
}
