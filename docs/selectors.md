# Public selectors

What a user stylesheet (`~/.config/<app>/style.css`) may rely on. Anything not listed
here is internal and may change without notice; renaming anything listed here is a
breaking change for the person's file.

Prefer token overrides (`.ds { --accent: ...; --t-quick: 120ms }`) to selectors: they
re-theme every surface consistently, and every token variable
(`--<prefix><slug>`) is public.

## Surfaces

`[data-surface=<name>]` is on every surface root.

## Components

| Component | Root | Parts |
| --- | --- | --- |
| Alert | `.ds-alert` | `.ds-alert-icon`, `.ds-alert-title`, `.ds-alert-body`, `.ds-alert-footer` |
| Avatar | `.ds-avatar` | none yet |
| Button | `.ds-button` | none yet |
| Chip | `.ds-chip` | `.ds-chip-remove` |
| EmptyState | `.ds-empty-state` | `.ds-empty-state-icon`, `.ds-empty-state-title`, `.ds-empty-state-body`, `.ds-empty-state-action` |
| HoverCard | `.ds-hovercard` | `.ds-hovercard-body` |
| List | `.ds-list` | none yet |
| Menu | `.ds-menu` | `.ds-menu-item`, `.ds-menu-separator` |
| Popover | `.ds-popover` | `.ds-popover-body`, `.ds-popover-arrow` |
| Row | `.ds-row` | none yet |
| SectionHeader | `.ds-section-header` | none yet |
| SegmentedControl | `.ds-segmented` | none yet |
| Sheet | `.ds-sheet` | `.ds-sheet-body` |
| SidePanel | `.ds-side-panel` | `.ds-side-panel-header`, `.ds-side-panel-body` |
| Skeleton | `.ds-skeleton` | none yet |
| Slider | `.ds-slider` | `.ds-slider-track`, `.ds-slider-fill`, `.ds-slider-thumb` |
| Toast | `.ds-toast` | `.ds-toast-body`, `.ds-toast-action` |
| Toggle | `.ds-toggle` | none yet |
| Tooltip | `.ds-tooltip` | none yet |

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
| `data-activity` | `Activity`, on `.ds`: written as `inactive` while the window is not the one focused, absent while it is |
| `aria-*` | the state an element exposes to assistive technology |
