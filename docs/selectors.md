# Public selectors

What a user stylesheet (`~/.config/<app>/style.css`) may rely on. Anything not listed
here is internal and may change without notice; renaming anything listed here is a
breaking change for the person's file.

Prefer token overrides (`.ds { --accent: ...; --t-tap: 120ms }`) to selectors: they
re-theme every surface consistently, and every token variable
(`--<prefix><slug>`) is public.

## Surfaces

`[data-surface=<name>]` is on every surface root.

## Components

| Component | Root | Parts |
| --- | --- | --- |
| Alert | `.ds-alert` | `.ds-alert-icon`, `.ds-alert-title` |
| Avatar | `.ds-avatar` | none yet |
| Button | `.ds-button` | none yet |
| Chip | `.ds-chip` | `.ds-chip-remove` |
| List | `.ds-list` | none yet |
| Menu | `.ds-menu` | `.ds-menu-item`, `.ds-menu-separator` |
| Popover | `.ds-popover` | none yet |
| Row | `.ds-row` | none yet |
| SectionHeader | `.ds-section-header` | none yet |
| SegmentedControl | `.ds-segmented` | none yet |
| Sheet | `.ds-sheet` | none yet |
| Slider | `.ds-slider` | `.ds-slider-track`, `.ds-slider-fill`, `.ds-slider-thumb` |
| Toast | `.ds-toast` | none yet |
| Toggle | `.ds-toggle` | none yet |

## Attributes

| Attribute | Meaning |
| --- | --- |
| `data-variant` | a component's role or bezel |
| `data-size` | the `ControlSize` slug: mini, small, regular, large |
| `data-state` | `Check`: off, on, mixed |
| `data-selected` | `Selection` |
| `data-busy` | `Availability::Busy` |
| `data-availability` | `Availability`: enabled, disabled, busy |
| `data-focus` | `FocusStyle`: ring, highlight |
| `data-activity` | `Activity`, on `.ds`: active, inactive (the window's focus) |
| `aria-*` | the state an element exposes to assistive technology |
