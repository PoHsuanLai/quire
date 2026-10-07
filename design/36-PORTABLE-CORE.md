# 36 - Portable core, desktop extras

Settled with the owner 2026-10-07. Everything except the desktop environment itself is
cross-platform (macOS, Windows, other Linux desktops). On our desktop the same software does
more: the extras are additive, never a different program.

## 1. The rule

1. **Every app and service has a portable core** that is complete on its own. mailo reads,
   writes and files mail; anyview views files; almanac remembers; docket runs an agent over
   the app's own intents. A core never shows a disabled control for an extra it lacks: the
   extra is simply not there.
2. **Desktop extras light up by capability**, probed at runtime ("is `org.quire.Intents1`
   answering?"), never by asking which OS or desktop this is. A missing or stopped service
   means the core path, not an error.
3. **Extras stay out of foreign builds.** Desktop integration code sits behind the Cargo
   feature `quire-desktop` (default on for `target_os = "linux"`, absent elsewhere). The core
   must build and pass its tests with `--no-default-features`; that build is a gate check.
4. **One client seam per service, two kinds of transport.** A service's client crate owns a
   `Transport` seam: D-Bus on our desktop (activation, portals, one instance per session,
   intentd routing), and elsewhere either in-process (the app hosts the service) or latchkey
   (a per-user agent over a Unix socket / named pipe, connect-or-start). porter-client and
   almanac-client already have this shape; it becomes the pattern for every service.
   latchkey itself stays a byte-stream lifecycle library: it does not grow a D-Bus side.

## 2. Who is what

| Piece | Portable core (everywhere) | Desktop extra (our DE only) |
| --- | --- | --- |
| quire | components, tokens, motion, ds-blitz windows, scrolling, ds-helpers (PackageKit is Linux; elsewhere `Unsupported`) | `ds-desktop` capability probe; appearance broadcast; compositor blur/materials; Space tint |
| mailo | mail, contacts, rules, keys, settings window; its own sign-in | system accounts and consent (accountd); intents for the companion; memory shared across apps |
| anyview | viewing, thumbnails, plugins with the helper installer | Quick Look peek from the shell; intents; share sheet |
| porter | accounts engine and providers, hosted in-process or as a latchkey agent | accountd as the session's one account store; system consent sheet; syncd |
| almanac | memory files, event log, search, sealing, in-process `Memory` | memoryd: one memory for every app, file watching, consolidation on a timer |
| docket | an in-app agent: router, policy point, reviewer, agent loop, skills over the app's own intents, confirm sheet drawn in-app | companiond and intentd: one companion across all apps; `quire do`; cua (computer use); voiced; readerd quarantine |
| desktop only | | sill, shell-host, casement, detent, keycap, palmrest |

## 3. In code

- **Probe:** `ds_desktop::Desktop` (quire crate `ds-desktop`): `Desktop::probe()` answers per
  capability (`Capability::{Intents, Accounts, Memory, Companion, Appearance, Materials, Peek,
  Share, Helpers}` → `Presence::{Here, Absent}`), cached and refreshed on the bus's
  NameOwnerChanged. `use_desktop()` gives components the answer as a signal. Off Linux, or
  without the feature, every capability is `Absent` and the crate pulls no D-Bus dependency.
- **Feature:** each app's `quire-desktop` feature enables its desktop modules
  (`desktop/` directory by convention) and the service clients' D-Bus transports. Core
  modules never import from `desktop/`; the boundary script checks it.
- **Transports:** a client picks D-Bus when `Desktop::probe()` says the service is here,
  otherwise its portable transport (in-process or latchkey). The choice happens once at start
  and again when the capability changes.
- **UI:** an extra's entry point (a menu item, a toolbar button, a settings pane) renders only
  under `Presence::Here`. No "requires Quire desktop" placeholders.

## 4. Gates

Every cross-platform repo adds to its lane gate: `cargo check --workspace --no-default-features`
(and its tests where they run without the desktop), plus the boundary rule "core does not
reach `desktop/`, zbus, or another service's D-Bus crate". A macOS/Windows CI build is the
eventual check; until then the no-default-features build is the proxy.
