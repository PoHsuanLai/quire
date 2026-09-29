# Architecture

quire is the design system of the desktop shell program: tokens, palette, icons, components and
motion, and the Blitz glue that draws them. `design/` says what everything looks like,
`DESIGN.md` maps each design section to its module, `CONSUMING.md` is the guide for a downstream
crate, and `CONVENTIONS.md` holds the rules every repo shares.

## Crate map

| Crate | What it is |
| --- | --- |
| `ds` | renderer-free and effect-free: vocabulary, tokens, stylesheet, icons, motion, components, the shell parts and widgets, the linter |
| `ds-settings` | settings I/O: lenient load, atomic save, the directory watch, the settings portal |
| `ds-settings-derive` | `#[derive(SettingsSchema)]` |
| `ds-native` | the Blitz glue: launch, fonts, snapshots, PDF output, the headless `Harness` |
| `anyrender_pdfrum` | anyrender scenes as vector PDF pages |
| `ds-gallery` | every component across theme, accent, motion and material |
| `tools/icons` | the app-icon post-process |

Inside `ds` the layers run `core/`, `style/`, `motion/`, `lint/`, the host seams, `overlay/`,
`root/`, `components/`, `shell/`, `assembly/`; each names only the ones below it (`DESIGN.md`
has the full list).

## Repo rules

- **Effect boundary** (`scripts/check-boundary.sh`): `ds` never reaches `zbus`, `notify`,
  `tokio`, the blitz and anyrender crates, `dioxus-native` or `winit`; `ds-settings` never
  reaches the blitz crates or `dioxus-native`. A pure crate that seems to need an effect returns
  a description of it to the caller.
- **Gate:**

  ```bash
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace --all-features
  ./scripts/check-boundary.sh
  cargo deny check licenses
  ```

- **No `unsafe`** anywhere in the workspace.
- **Design-system rules:** every class is prefixed `ds-`; variants go in
  `data-variant`/`data-size`, state in `aria-*`; every colour, duration, easing, keyframe and
  font comes from the token table. A missing component or token is added here, never patched in
  a consumer.
- **Coherence rules for every consumer:** no stylesheet outside quire contains a hex/rgb/hsl
  colour, a raw `ms`/`s` duration, a raw `cubic-bezier`, a `@keyframes`, a `font-family` or a
  `:root` selector, and none styles `.ds-*` or `[data-theme|accent|motion|material]`
  (`ds::lint::stylesheet()`, which every downstream crate runs in its own tests); no surface or
  app writes raw `button`/`input`/menu markup, it uses quire components (`ds::lint::markup()`
  over an SSR render). `CONSUMING.md` section 5 has the tests.
- **Lint profile:** `Profile::Strict` is the default and runs every rule but
  `OffGrammarTiming` (the HIG guardrails report as warnings); `Profile::Details` adds the
  details grammar's timing. quire's own `self_lint` runs Strict. Consumers read quire by path,
  so a new failing rule breaks every consumer's tests at once: tell their owners before it
  lands.
- **Time in `ds`** is read only through `ds::time::now`/`since`/`sleep`, never `Instant::now()`
  or `futures_timer` directly (`tests/clock_rule.rs`), so the virtual clock reaches it.
- **Timing tests** on `Clock::Wall` never assert a state at one fixed instant near a settle,
  hover-intent, submenu or toast boundary: `Harness::advance` only guarantees *at least* the
  time asked for, and a loaded `cargo test --workspace` stretches it past the boundary. Poll
  with `ds_native::harness::settle_until`, time the settle on the wall clock, and assert order
  (A landed before B, or at least the window after it was asked). A "not yet" check may assert
  a fixed elapsed time only at or under half the window. A test built with
  `HarnessConfig::with_clock(Clock::Virtual)` is exempt: there `advance` moves one clock for CSS
  and every ds timer, and a boundary assertion (`advance(window - 1ms)` not yet, `advance(1ms)`
  now) is the better test.
- **A settings key nobody reads is reported on load and not preserved.** A new settings key is
  a design/22 row first.
- **`Px` and the geometry built on it are `f32`**, like the renderer's own layout, and so
  `PartialEq` without `Eq`: the one float type in the data.
- **The pinned dependency block's source of truth is `docs/workspace-deps.toml`**; the root
  `Cargo.toml` and every consumer copy it verbatim.
- **`docs/licensing-references.md`** is the verified licence table the borrowing rules cite.
