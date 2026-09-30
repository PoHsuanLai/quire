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
| Badge | `.ds-badge` | `.ds-badge-label` |
| Button | `.ds-button` | `.ds-button-icon`, `.ds-button-label` |
| Checkbox | `.ds-checkbox` | `.ds-checkbox-indicator`, `.ds-checkbox-label` |
| Chip | `.ds-chip` | `.ds-chip-remove` |
| EmptyState | `.ds-empty-state` | `.ds-empty-state-icon`, `.ds-empty-state-title`, `.ds-empty-state-body`, `.ds-empty-state-action` |
| HoverCard | `.ds-hovercard` | `.ds-hovercard-body` |
| KeyEquivalent | `.ds-key-equivalent` | `.ds-key-equivalent-key` |
| Label | `.ds-label` | none yet |
| LevelIndicator | `.ds-level-indicator` | `.ds-level-indicator-track`, `.ds-level-indicator-fill`, `.ds-level-indicator-icon` |
| List | `.ds-list` | none yet |
| Menu | `.ds-menu` | `.ds-menu-item`, `.ds-menu-separator` |
| Popover | `.ds-popover` | `.ds-popover-body`, `.ds-popover-arrow` |
| ProgressIndicator | `.ds-progress` | `.ds-progress-track`, `.ds-progress-fill`, `.ds-progress-indicator`, `.ds-progress-glyph` |
| RadioGroup | `.ds-radio-group` | `.ds-radio-group-item`, `.ds-radio-group-indicator`, `.ds-radio-group-label`, `.ds-radio-group-image` |
| Row | `.ds-row` | none yet |
| SectionHeader | `.ds-section-header` | none yet |
| SegmentedControl | `.ds-segmented` | `.ds-segmented-segment`, `.ds-segmented-indicator`, `.ds-segmented-label`, `.ds-segmented-icon` |
| Sheet | `.ds-sheet` | `.ds-sheet-body` |
| SidePanel | `.ds-side-panel` | `.ds-side-panel-header`, `.ds-side-panel-body` |
| Skeleton | `.ds-skeleton` | none yet |
| Slider | `.ds-slider` | `.ds-slider-track`, `.ds-slider-fill`, `.ds-slider-thumb`, `.ds-slider-icon`, `.ds-slider-tick` |
| TextField | `.ds-text-field` | `.ds-text-field-frame`, `.ds-text-field-icon`, `.ds-text-field-suffix`, `.ds-text-field-help`, `.ds-text-field-tokens` |
| Toast | `.ds-toast` | `.ds-toast-body`, `.ds-toast-action` |
| Toggle | `.ds-toggle` | `.ds-toggle-track`, `.ds-toggle-indicator` |
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
| `data-pressed` | `PressPhase`: present while a pointer or a key holds the control down |
| `data-role` | a button's role: normal or destructive |
| `data-focus` | `FocusStyle`: ring, highlight |
| `data-activity` | `Activity`, on `.ds`: written as `inactive` while the window is not the one focused, absent while it is |
| `aria-*` | the state an element exposes to assistive technology |
