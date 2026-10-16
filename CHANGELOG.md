# Release notes

Quire has published its release notes as the tag message. This file keeps them in the tree, newest
first, with what a consumer has to change.

## v0.3.28

Consumers re-pin once. Cargo.lock files are refreshed by whoever bumps the pin.

### Breaking, by crate

**ds-settings**
- `Store::save` returns `Result<Saved, SettingsError>` instead of `Result<(), SettingsError>`.
  `Saved::kept_at` is where a damaged file in the way was moved (`<name>.bad-<seconds>`); tell the
  person when it is `Some`. `SpacesStorage::save_spaces` and `save_today` return `Saved` too, and
  their type parameters now also need `DeserializeOwned` (a file that is not the type is moved
  aside, not overwritten).
- `Origin` (from `boot_spaces`) is `#[non_exhaustive]`, no longer `Copy`, and gains
  `Damaged(DamagedFile)`: a `spaces.json` that is there and unusable. The Spaces run from
  `first_run()` in memory, nothing is written, and the first save moves the file aside. Match it
  with a wildcard arm and clone where you copied it.
- `SpacesStorage::load_spaces` returns `SpacesLoad<P, R>` (`Stored`, `Missing`, `Damaged`;
  `#[non_exhaustive]`) instead of `Option<Spaces<P, R>>`. A damaged file is no longer a first run.
- `DamagedFile` and `Saved` are new, exported from the crate root.
- `data_dirs()` (schema discovery) lists the system directories first and `XDG_DATA_HOME` last, so a
  user's schema never shadows the system's. A schema is admitted only for its own program and
  location, and schema files are capped.
- Settings and Spaces files are read with a size cap and never block on a FIFO; they are written
  durably and atomically, readable by the person only.

**ds-settings, `live` feature**
- `LiveError` (already `#[non_exhaustive]`) gains `Timeout(String)`, raised by the client when a
  module does not answer.
- `LiveClient::with_timeouts(Timeouts)`; `Timeouts::new(read, action)` and `ds_settings::live::Deadline`
  are new. Every client call has a deadline.
- `Changed` is sent by unicast to the peers the module let read (those that called `Describe`,
  `Get`, `Snapshot` or `LiveClient::changes`), not broadcast. A client takes a signal only from the
  module it asked.

**ds-blitz**
- `AppConfig::with_file_roots(FileRoots)` is new, and `ds_blitz::FileRoots` is re-exported from
  blitz-kit. The app document reads a `file:` URL only from inside the directories the app names; a
  path outside them, through a symlink out of them, or with `..` is refused, and so is anything but
  a regular file of at most 32 MiB. The default names none: an app that sets no roots reads no
  `file:` URL at all (`data:` is unaffected, and a frame is never served `file:`). Apps that show
  local images or icons (wallpaper cache, icon directories) must set their roots. The same setter is
  on `ds_harness::HarnessConfig`.
- `ds_blitz::seam::DsNet::top` takes the `FileRoots` as a fourth argument; `Setup` gains `file_roots`.
- `AppConfig::with_link_failure(LinkFailureHandler)`; `LinkFailureHandler` and `LinkOpenError` are
  new. A clicked link is opened on a worker; a link that cannot be opened is reported to the handler
  (default: stderr). A `mailto:` link loses its `attach` and `attachment` fields.
- Clipboard reads run on a worker with a wait and a size cap. The workspace `arboard` no longer
  enables `wayland-data-control`.
- The print portal's calls have deadlines; the print fallback stages the PDF privately.
- Feature `pdf` now depends on ds-helpers. A thumbnail reads its PDF through `read_capped` (128 MiB,
  regular files only); a bigger file shows as a PDF with no preview.

**ds-helpers**
- New files API: `read_capped`, `read_text_capped`, `ReadCap` (`bytes`, `kib`, `mib`), `ReadError`
  (`#[non_exhaustive]`; `NotFound` is the only variant that means absent), `write_atomic` with
  `NewFileMode`, `WriteError`, `move_aside` with `UnixSeconds`.
- `PackageKit::with_timeouts(Timeouts)`; `Timeouts::new(step, install)`; `Deadline` re-exported.
  PackageKit signals are believed only from the PackageKit that made the transaction, every wait has
  a deadline, and only a package that was asked for is installed.
- `PackageName` accepts `[A-Za-z0-9+._-]` only, starting with a letter or digit.

**ds-peer** (new crate)
- `Deadline` (`READ`, `START`, `ACTION`, `PROMPT`, `of`, `duration`), `within(...)` and
  `NotAnswering`: one deadline for every call to a peer.

**Others**
- `ds-desktop`: `Desktop::probe`, `Outputs` and their watches stop waiting on a silent bus after
  `Deadline::START`; output rows with no size or no scale are dropped and the list is capped.
- `ds-shell`: `Hidden` wipes its memory when dropped (adds `zeroize`).
- `ds::root::DataName::parse` takes a `&'static str`.
- A stored `Lang` goes through `Lang::parse`; ids and times read from files are range-checked.
- `deny.toml` checks advisories, bans and sources.

### Pins

- blitz-kit `0ba642e76f3c7edca77b228f05e0399af46e2ec3`: `blitz_kit::net::FileRoots`,
  `LocalNet::new(FileRoots)`, `LocalSource::read_within(&FileRoots)`, a 32 MiB cap and
  regular-file-only reads (`MAX_RESOURCE_BYTES`); `adapter::request_watched_device` with a deadline,
  `DeviceHealth`, `block_on_timeout`, and wgpu errors logged instead of panicking.
- latchkey `99cc9deb198860293253c534c4d26b382ec8b87e`: new `Error` variants `Unsafe` and `WrongOwner`
  (match them), `standing()`.
- keycap-client: quire pins no keycap crate. keycap `fe71ad2c7de113ee6e98f7ea1afd9514536a097c`
  (client socket close-on-exec, bounded non-blocking connect, bounded `read_text`) is for the
  consuming apps and the desktop to pin themselves.
