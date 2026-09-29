# Conventions

Binding on every contributor, human or agent. `design/` says *what* every token, component and
motion looks like; this file says *how the code is written*.

If a convention here blocks you, say so in your report. Do not quietly deviate — a silent
deviation in a shared type is exactly the failure this file prevents.

---

## 0. Design style

The rules below are the general shape of the code. Everything after this section is a specific
application of them, so when a later rule seems arbitrary, this is why it exists.

### Prefer pure functions

A function that takes values and returns values can be tested with a table, reasoned about in
isolation, and reused from a context its author never imagined. One that reaches for a socket,
a clock or a global cannot.

Push effects to the edges. The layering already encodes this: `ds` is renderer-free and
effect-free, mechanically kept that way by `scripts/check-boundary.sh` (§10); `ds-settings`
touches the disk and the bus; `ds-native` owns the renderer. When a pure crate seems to need an
effect, that is a sign the effect belongs to the caller: return a description of what should
happen rather than doing it.

Effects belong in the signature. `&mut self`, `Result`, or living in a crate permitted to do
I/O are how a reader sees that something happens; a function that looks pure and is not is
worse than one that is honestly imperative.

### Separate data from logic

Data types describe what is true. Functions describe what follows from it. Keep them apart.

A struct's own `impl` should hold construction, derivation and accessors — the things that are
*about* that value. Anything that coordinates several values, or makes a decision, reads better
as a free function or as an `impl` on the type that owns the decision.

Avoid the type that grows methods because it was convenient rather than because they belong.
If half a struct's methods never touch half its fields, it is two types.

### Make illegal states unrepresentable

Encode invariants in types rather than checking them at runtime and hoping every caller
remembers. This is the whole reason the vocabulary is enums.

- Newtype every identifier and every unit. `WorkspaceId`, not `String`; `Px`, not `f32`.
  A newtype costs nothing at runtime and makes an argument-order mistake a compile error.
- No stringly-typed fields where a closed set exists. `Accent`, not `&str`.
- No `bool` in state (see §7). No `bool` parameters either: a call reading `send(true)` tells
  the reader nothing, while `send(Confirm::Yes)` tells them everything.
- Prefer an empty collection to `Option<Vec<T>>`. There is one way to say "none" and it needs
  no unwrapping.
- Model absence precisely. A type that cannot represent zero of something is better than an
  `Option<u32>` whose every caller must decide what `Some(0)` meant.

### Parse, don't validate

Convert untrusted input into a type that cannot be wrong, once, at the boundary — then the rest
of the program needs no defensive checks. A validation function
that returns `bool` and leaves the caller holding the same loose type has moved the problem,
not solved it.

A fallible constructor returns `Result`. A constructor that cannot fail returns `Self`.

### Traits are seams, not decoration

A trait earns its place when two implementations are genuinely swapped at that boundary, or a
generic consumer needs it. A trait with one implementation is indirection with no payoff, and a trait invented to
group functions that merely feel related makes the call site harder to follow.

Prefer an enum to a trait for closed vocabulary: exhaustive in one place, so that adding a
variant produces a list of compile errors naming every site that must be updated. A `dyn Trait` hides that list.

Implement the standard traits when the semantics genuinely hold, and derive rather than
hand-write unless a behaviour must differ. `Display` is for humans, `Debug` is for developers,
and neither is a serialization format — that is `serde`'s job.

### Errors are values, and they carry a decision

See §5 for the mechanics. The principle: an error type exists so a caller can *act*, not so a
string can be logged.

Reserve panics for broken invariants inside our own code, and document them as caller
contracts. Malformed input is never a panic.

### Borrowing from other projects

This workspace is `MIT OR Apache-2.0`. Much of what is worth reading is not, and the licence
determines what you may do with what you read. `docs/licensing-references.md` has the verified
table; three buckets, not six:

- **permissive** (MIT, Apache-2.0, BSD, 0BSD) — code may be copied, with attribution.
- **weak copyleft** (MPL, LGPL) — link freely, but MPL's unit of copyleft is the *file*: paste one
  function and that file becomes MPL, and our crate-level licence claim becomes false.
- **strong copyleft** (GPL, AGPL, EUPL) — read for facts, never paste.

**The rule for reading is note-and-close.** Read the source, reduce what you learned to a sentence
about observable behaviour, close the file, implement from the sentence. The test: if it fits in a
sentence about observable behaviour, it is a fact and it is yours; if you would need the source
open to reproduce it, it is expression and it is theirs. Signs you actually copied: matching
identifiers, surviving comments, matching branch order, matching magic constants, matching error
strings, the same bug.

**A worker in an isolated worktree cannot see gitignored files.** A git worktree carries tracked
content only; copy any ignored input into the worktree before starting.

**Never put GPL, AGPL or MPL source into a model's context and ask for an equivalent.** That is
note-and-close inverted, at volume, with nobody able to testify to independent derivation. Put the
spec section and our own fixture in the prompt instead.

**Cite the behaviour, not the source.** `// taken from <gpl-project>/file.c:412` in an MIT crate
is a pointer to copyleft code that reads as an admission, and it does not tell the next reader
the thing they need. Write what the behaviour is and point at the test that proves it. If you genuinely
copied permissively-licensed code, attribute it properly instead — that is a different situation
with different obligations.

### Composition over accumulation

Prefer iterator chains and small combinators where they read more clearly than a loop, and a
plain loop where they do not — clarity wins over point-free style, every time. Prefer
returning a new value to mutating an argument. Where a mutable accumulator really is the
clearest expression, keep it local to the function so the mutation cannot be observed from
outside.

Keep functions short enough to hold in your head. A long function is usually several functions
that have not been named yet, and naming them is most of the work of understanding them.

## 2. Derives

Every public type derives, in this order:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
```

(`Serialize, Deserialize` only on a type that is stored.) Then, where they apply:

- `Copy` — only for types that are one word or smaller (IDs, fieldless enums).
- `Hash` — every ID newtype, and anything used as a map key.
- `PartialOrd, Ord` — only where a total order genuinely means something. Not on enums
  whose variant order is arbitrary.
- `Default` — only where a default is genuinely meaningful.

**Secrets.** Never `#[derive(Debug)]` on a type that holds a password or a token; derived
`Debug` leaks into logs, panics, and error chains. Write it by hand, redacting the contents.

## 3. Serde

The serde form of a stored type (a settings file, a state file) is its on-disk schema.

- **Enums with data: adjacently tagged.**
  ```rust
  #[serde(tag = "kind", content = "v", rename_all = "snake_case")]
  ```
  Adjacent tagging, not internal tagging: internal tagging cannot represent newtype variants
  over non-map types (it fails at runtime, not compile time).
- **Fieldless enums:** `#[serde(rename_all = "snake_case")]`.
- **Structs:** field names as written, `snake_case`.
- **Never `deny_unknown_fields`** on a stored type: a settings file keeps keys it does not know
  (design/22-SETTINGS.md section 2).
- Every stored type has a round-trip test.

## 4. `#[non_exhaustive]`

On nothing. Nothing is published, so `non_exhaustive` buys no compatibility and costs
exhaustive matching — which is the whole
reason the vocabulary is enums. Revisit only if a crate is ever published.

## 5. Errors

One error enum per crate, in `error.rs`, built with `thiserror`.

- Libraries return typed errors; no `anyhow` in a library crate.
- No `unwrap()` or `expect()` in non-test code, except where a comment on the same line
  proves the invariant.
- `panic!` is for programmer error only. Malformed input parses to an error value, never a
  panic.

## 6. Time

A function that needs the time takes it as an argument, or (in `ds`) reads it through
`ds::time` (§11). Tests use fixed instants or the virtual clock so failures are reproducible.

`std::time::Duration` for configured intervals.

## 7. Naming

- Enums read as values at the use site: `Availability::Disabled`, not `Availability::IsDisabled`.
- No `bool` fields in state. Predicate **returns** are `bool`.
- No `get_` prefix. `fn label(&self)`, not `fn get_label(&self)`.
- Vendor vocabulary stays out of names: code, class names and tokens never name the reference
  platform.

## 8. Modules and files

One concept per file; a file over ~400 lines wants splitting. `lib.rs` declares modules and
re-exports the flat public surface — it holds no definitions.

Public items carry a doc comment saying what they mean, not what they are.

## 9. Tests

- **Unit tests** in `#[cfg(test)] mod tests` beside the code, for private behaviour.
- **Integration tests** in `crates/<crate>/tests/<topic>.rs`, for the public surface.
- Table-driven wherever there is more than one case: a `const CASES: &[(Input, Expected)]`
  and one loop. A failure must name the case.
- No test touches the network or the real system (the real session bus, real `~/.config`).

## 10. Verification

Before reporting work complete, all of these must pass:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/check-boundary.sh
cargo deny check licenses
```

`scripts/check-boundary.sh` enforces the effect boundary: `ds` must never reach `zbus`,
`notify`, `tokio`, the blitz and anyrender crates, `dioxus-native` or `winit`; `ds-settings`
must never reach the blitz crates or `dioxus-native`.

Note that `cargo tree -p <crate> -i <dep>` **exits 101 when the dependency is absent**, which
is the condition we want. A CI step that checks the exit status alone therefore fails exactly
when it should pass; `scripts/check-boundary.sh` checks for output instead.

While other work shares the checkout, run `cargo fmt -p <your-crate>`, not `cargo fmt --all`:
the `--all` form rewrites every file in the workspace. `--all --check` is read-only.

Report honestly. A failing test reported as passing costs more than the bug did.

## Substrings are not tokens

`contains`, `ends_with` and `starts_with` compare *characters*. A domain, a class name, a CSS
property and a word are **tokens**, and matching one with a substring is a bug that passes its
own test: `ends_with("example.com")` matches `evil-example.com`; `contains("io")` matches every
`connection`.

**So: when the thing being matched has a grammar, match it at its boundaries.** A domain compares
whole labels; a class list is split on whitespace; CSS goes through a tokenizer. The cost is
asymmetric, which is why this is a rule rather than a preference: a too-narrow match fails
visibly on input you have, a too-wide one silently does the wrong thing on input you have not
thought of.

And before reaching for the text at all, ask **what the structure already told you**: a value's
position in a grammar (a property name, a selector, a reply to a known command) is more reliable
than the words it happens to carry.

## Run it the way its user would, before concluding anything about it

A failure that reproduces only through your own harness is a fact about the harness.

**So: before reporting that something does not work, run it with nothing of yours around it.**
No wrapper, no forced backend, no injected variable, no redirected output. If it works there, the
finding is about the wrapper. If it fails there too, you have something. The tell is a conclusion
drawn entirely from runs that share a flag you added.

A component-tree test proves the components agree with each other; it does not prove the
renderer draws them or that a keystroke arrives. Reach for the harness (`ds_native::Harness`)
whenever a conclusion is about what the user sees.

## An assertion that was already true proves nothing

A test that asserts `count > 0` after an action that should add one passes whatever the action
did if the fixture already had one.

**So: assert the difference the action makes, not a state the action happens to be compatible
with.** Read the quantity before, act, and compare: `before + 1`, not `> 0`. Where the "before"
is awkward to capture, name the exact value the action must produce. The tell is an assertion
that could be moved *above* the line that does the work and still pass.

A fixture can be vacuous the same way. A test that spells out the value under test and then
asserts against its own copy agrees with itself whatever the program does. **Ask the program for
the thing you are testing.** Ask it of every new assertion that uses `>`, `<`, `is_empty`,
`is_some`, `contains` or `!`: those are where a condition wide enough to be accidentally
satisfied hides.

## 11. quire addenda (2026-09-24)

These apply to quire, shell-host, sill and every bundled app, on top of §0-§10.

- Coherence rules for every consumer: no stylesheet outside quire contains a hex/rgb/hsl colour,
  a raw `ms`/`s` duration, a raw `cubic-bezier`, a `@keyframes`, a `font-family`, or a `:root`
  selector (enforced by `ds::lint::stylesheet()`, which every downstream crate runs in its own
  tests); no surface or app writes raw `button`/`input`/menu markup, it uses quire components
  (enforced by `ds::lint::markup()` over an SSR render); motion state is driven by quire timers,
  never by ad-hoc sleeps.

- Small structs; functional style where it fits; data separate from logic; type-driven:
  newtypes and enums carry meaning, traits only at real seams (§0 already says this; it is
  restated because the user restated it).
- No `bool` anywhere in a public signature or a settings key: two-state enums with named
  variants (`Magnification::{On, Off}`, `Availability::{Enabled, Disabled}`).
- No long files or long functions: one concept per file, split near 400 lines (§8); a function
  that needs a comment to separate its phases is two functions.
- No `unsafe` outside the single allowed module per repo (shell-host's raw window handle),
  each block with a SAFETY comment; `unsafe_code = "deny"` at the workspace.
- Dependencies are settled first: the pinned block in `docs/workspace-deps.toml` is copied
  verbatim into every workspace; a new dependency is a change to that file, reviewed, then
  copied, never added to one crate alone.
- Design system rules: every class is prefixed `ds-`; variants in `data-variant`/`data-size`,
  state in `aria-*`; consumers never style `.ds-*` or `[data-theme|accent|motion|material]`;
  every colour, duration, easing, keyframe and font comes from the token table; a missing
  component or token is added to quire first (stop and report), never patched locally.
- Every value the design docs mark "proposed" is read from a settings key
  (`design/22-SETTINGS.md`) with that default; never hard-coded.
- Timing tests never assert a state at one fixed instant near a settle, hover-intent, submenu
  or toast boundary: `ds_native::Harness::advance` lets real wall-clock time pass (quire's
  timers are `futures-timer` sleeps a harness cannot fake), it only ever guarantees *at least*
  the time asked for, and a loaded parallel `cargo test --workspace` can stretch it past a
  boundary the test meant to stop short of. Instead, poll with `ds_native::harness::settle_until`
  up to its bound, time the settle on the wall clock (`Instant::now()`), and assert order (A
  landed before B, or only after at least the window's duration since it was asked for) rather
  than a state at an instant; a "not yet" check may still assert a fixed elapsed time, but only
  at or under half the window, so a loaded machine's overshoot cannot cross the boundary first.
  A test built with `HarnessConfig::with_clock(Clock::Virtual)` is
  exempt: there `advance` moves one clock for CSS and every ds timer, so a fixed instant is exact
  and a boundary assertion (`advance(window - 1ms)` not yet, `advance(1ms)` now) is the better
  test. ds code reads time only through `ds::time::now`/`since`/`sleep`, never
  `Instant::now()` or `futures_timer` directly, or the virtual clock cannot reach it.
