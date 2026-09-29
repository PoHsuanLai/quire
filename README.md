# quire

quire is the design system every surface and app in this program draws with: one set of colour,
type, motion and shape tokens; one component library; one settings schema — shared by the
desktop shell (`sill`, on `shell-host`) and by every bundled app, mailo among them. It exists so that a menu, a button or a hover card looks and moves
identically whether it is drawn by the bar, a widget, or a mail client, and so that adding a
component or a token happens once, in one place, instead of being re-implemented per app.

`ds` itself is renderer-free and effect-free (no `tokio`, no `zbus`, no `blitz-*`, no `winit`;
`scripts/check-boundary.sh` enforces this mechanically). `ds-settings` does I/O — TOML files, a
directory watch, the desktop settings portal — but still never touches a renderer. `ds-native` is
the one crate that names Blitz at all: windows, fonts, frames, edit surfaces, focus, PDF output
and a headless test harness. `ds-gallery` is the visual reference every one of those renders against.

## The crates

| Crate | What it is |
| --- | --- |
| `crates/ds` | The design system itself: tokens, palette, icons, motion, components, the `Ds`/`Surface` roots, and (behind the `lint` feature) the coherence linter. Depends only on `dioxus` (core, no renderer), `serde`, `thiserror`, `futures-timer`, and `cssparser` for `lint`. |
| `crates/ds-settings` | `appearance.toml`: atomic read/write, a live directory watch, the freedesktop settings portal, `use_environment`, and `#[derive(SettingsSchema)]` (re-exported from `ds-settings-derive`) for the Settings app's schema-is-data model. |
| `crates/ds-settings-derive` | The `SettingsSchema` proc macro. Not used directly — always through `ds_settings::SettingsSchema`. |
| `crates/ds-native` | Blitz glue: `launch(app, config)` to run a quire app in a real window and `open_window` for another; `register_fonts`; sandboxed `<iframe srcdoc>` frames with link interception; the edit surface (typing, IME, selection); focus and caret hosts; clipboard and file drops; headless `snapshot()` to PNG; PDF output and printing (`pdf`, `print`); and `Harness` for driving a real (headless) Blitz document with pointer, keyboard and a real or virtual clock in a test. |
| `crates/anyrender_pdfrum` | anyrender scenes written as vector PDF pages through pdfrum (paths stay paths, text stays text); `ds-native`'s PDF backend. |
| `crates/ds-gallery` | A bin: every component across Theme x Accent x MotionLevel x Material x Blur x Space preset, plus a tokens page, a matrix page and a motion lab, with `--snapshot DIR` for a contact sheet. |
| `tools/icons` | The app-icon exporter: `icons ship` writes `assets/icons/apps/` from `tools/icons/ship.toml`, `icons install` copies the set where `ds_settings` looks for it. |

Design docs (`design/`) are canonical; `DESIGN.md` maps each doc section to the module that
implements it; `FINDINGS.md` records what Blitz can and cannot do and the open items.

## Adding quire to your own app

See **[`CONSUMING.md`](CONSUMING.md)**: how to depend on `ds`/`ds-settings`/`ds-native`, the
`Ds` root and `Surface`, reading appearance, the component catalogue, the four coherence rules
(with the exact test to add for each), the settings-schema derive, and what Blitz cannot do.
**[`examples/consumer`](examples/consumer)** is a minimal worked app proving all four coherence
rules from outside this workspace — read it alongside `CONSUMING.md`, or copy its `Cargo.toml`
and `tests/coherence.rs` as a starting point for your own.

## Running the tests

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/check-boundary.sh
cargo deny check licenses
```

Call `cargo` plainly: never by absolute path, never with an inline `CARGO_BUILD_JOBS` or
`RUSTFLAGS` (a shim serialises every compilation on the machine).

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
cargo run -p ds-gallery --release -- --snapshot DIR --progress  # also refreshes tools/progress/shots/gallery
```

Compare your own surface against the matching gallery page at the same Appearance/Material —
PNG snapshots are a review artefact, not a CI gate (rasterisation drifts with the Blitz revision
quire is pinned to). See `CONSUMING.md` section 10 for what to look for.

## Repository layout

- `crates/`, `tools/icons` — the crates above.
- `design/` — the canonical UX specification (`design/README.md` has the reading order and the
  citation convention every doc in this repository follows).
- `examples/consumer/` — the minimal external consumer (also its own Cargo workspace).
- `docs/workspace-deps.toml` — the pinned dependency block, copied verbatim into this workspace's
  `Cargo.toml` and into `shell-host`'s and `sill`'s; changes only in a dedicated toolchain-bump.
- `CONVENTIONS.md` — how the code is written; binding on every contributor, human or agent.
- `DESIGN.md`, `FINDINGS.md` — which design doc section each module implements, and what is
  known about Blitz and still open.
