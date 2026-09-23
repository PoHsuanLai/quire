# quire

quire is the design system every surface and app in this program draws with: one set of colour,
type, motion and shape tokens; one component library; one settings schema — shared by the
desktop shell (`sill`, on `shell-host`) and by every bundled app, starting with mailo
(`docs/mailo-migration.md`). It exists so that a menu, a button or a hover card looks and moves
identically whether it is drawn by the bar, a widget, or a mail client, and so that adding a
component or a token happens once, in one place, instead of being re-implemented per app.

`ds` itself is renderer-free and effect-free (no `tokio`, no `zbus`, no `blitz-*`, no `winit`) so
a consumer still on a webview can adopt it exactly as one still on Blitz does
(`scripts/check-boundary.sh` enforces this mechanically). `ds-settings` does I/O — TOML files, a
directory watch, the desktop settings portal — but still never touches a renderer. `ds-native` is
the one crate that names Blitz at all: launching a window, registering fonts, and a headless test
harness. `ds-gallery` is the visual reference every one of those renders against.

## The crates

| Crate | What it is |
| --- | --- |
| `crates/ds` | The design system itself: tokens, palette, icons, motion, components, the `Ds`/`Surface` roots, and (behind the `lint` feature) the coherence linter. Depends only on `dioxus` (core, no renderer), `serde`, `thiserror`, `futures-timer`, and `cssparser` for `lint`. |
| `crates/ds-settings` | `appearance.toml`: atomic read/write, a live directory watch, the freedesktop settings portal, `use_environment`, and `#[derive(SettingsSchema)]` (re-exported from `ds-settings-derive`) for the Settings app's schema-is-data model. |
| `crates/ds-settings-derive` | The `SettingsSchema` proc macro. Not used directly — always through `ds_settings::SettingsSchema`. |
| `crates/ds-native` | Blitz glue: `launch(app, config)` to run a quire app in a real window, `register_fonts`, headless `snapshot()` to PNG, and `Harness` for driving a real (headless) Blitz document with pointer, keyboard and time in a test. |
| `crates/ds-gallery` | A bin: every component across Theme x Accent x MotionLevel x Material x Blur x Space preset, plus a tokens page, a matrix page and a motion lab, with `--snapshot DIR` for a contact sheet. |

Design docs (`design/`) are canonical; `DESIGN.md` maps each doc section to the module that
implements it; `FINDINGS.md` records what the Blitz spike (S1-S16) proved Blitz can and cannot
do, and what each wave found once it started building.

## Adding quire to your own app

See **[`CONSUMING.md`](CONSUMING.md)**: how to depend on `ds`/`ds-settings`/`ds-native`, the
`Ds` root and `Surface`, reading appearance, the component catalogue, the four coherence rules
(with the exact test to add for each), the settings-schema derive, and what Blitz cannot do.
**[`examples/consumer`](examples/consumer)** is a minimal worked app proving all four coherence
rules from outside this workspace — read it alongside `CONSUMING.md`, or copy its `Cargo.toml`
and `tests/coherence.rs` as a starting point for your own.

mailo's migration from its own hand-rolled CSS and markup to quire is
**[`docs/mailo-migration.md`](docs/mailo-migration.md)**: the concrete class-to-component
mapping, the two-phase file-ownership plan, and the acceptance gates.

## Running the tests

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-boundary.sh
cargo deny check licenses
```

Call `cargo` plainly — never by absolute path, never with an inline `CARGO_BUILD_JOBS` or
`RUSTFLAGS` (`ORCHESTRATION.md` "Builds": a shim serialises every compilation on the machine and
caps each at 4 GB). While a wave is in flight, format your own crate with `cargo fmt -p <crate>`
rather than `cargo fmt --all`, which rewrites every file in the workspace.

`examples/consumer` is its own Cargo workspace (outside this one — see its `Cargo.toml`), so its
tests run separately:

```bash
cd examples/consumer && cargo test
```

## Running the gallery

```bash
cargo run -p ds-gallery                              # opens interactively, on the tokens page
cargo run -p ds-gallery -- --page controls            # opens directly on one page
cargo run -p ds-gallery --release -- --snapshot DIR    # renders every page x state to DIR, then exits
```

Compare your own surface against the matching gallery page at the same Appearance/Material —
PNG snapshots are a review artefact, not a CI gate (rasterisation drifts with the Blitz revision
quire is pinned to). See `CONSUMING.md` section 10 for what to look for.

## Repository layout

- `crates/` — the five crates above.
- `design/` — the canonical UX specification (`design/README.md` has the reading order and the
  citation convention every doc in this repository follows).
- `spike/` — the throwaway W0 Blitz probe that produced `FINDINGS.md`'s S1-S16; its own separate
  Cargo workspace, excluded from this one.
- `examples/consumer/` — the minimal external consumer (also its own Cargo workspace).
- `docs/workspace-deps.toml` — the pinned dependency block, copied verbatim into this workspace's
  `Cargo.toml` and into `shell-host`'s and `sill`'s; changes only in a dedicated toolchain-bump
  wave.
- `docs/mailo-migration.md` — the mailo migration brief.
- `CONVENTIONS.md`, `ORCHESTRATION.md` — how the code is written, and how parallel work on it is
  divided; binding on every contributor, human or agent.
- `DESIGN.md`, `FINDINGS.md` — which design doc section each module implements, and what the
  Blitz spike and each wave found.
