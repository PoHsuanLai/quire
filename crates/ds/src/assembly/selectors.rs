//! The public selector table: what a user stylesheet may rely on (ARCHITECTURE.md section 10,
//! design/30 section 1.7).
//!
//! A person restyles their desktop by editing one CSS file after the design system's own. That
//! file may name exactly what this table lists: `[data-surface=<name>]` on every surface root,
//! `.ds-<component>` on a component's root, the parts a component lists as public
//! (`.ds-<component>-<part>`), the attribute axes below, and every token variable
//! (`--<prefix><slug>`, from `Kits::vocabulary`). Anything else is internal and may change
//! without notice; a rename of anything listed here is a breaking change for the person's file,
//! so it updates this table and `docs/selectors.md` (a test keeps the page equal to
//! [`markdown`]).
//!
//! A component's public parts are added here in the change that gives it the part
//! (a gallery example that restyles it and a `lint::user_stylesheet` case that accepts it come
//! with the row).

/// One component's public class names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentSelectors {
    /// The component, as design/30 names it.
    pub component: &'static str,
    /// The root class without its dot: `ds-<component>`.
    pub root: &'static str,
    /// The public parts, without the root's prefix: `remove` is `.ds-chip-remove`.
    pub parts: &'static [&'static str],
}

impl ComponentSelectors {
    /// The root as a selector: `.ds-chip`.
    pub fn root_selector(&self) -> String {
        format!(".{}", self.root)
    }

    /// Each public part as a selector: `.ds-chip-remove`.
    pub fn part_selectors(&self) -> Vec<String> {
        self.parts
            .iter()
            .map(|part| format!(".{}-{part}", self.root))
            .collect()
    }
}

/// An attribute a user stylesheet may select on, on any component that writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    /// The attribute, or the family of attributes (`aria-*`).
    pub attribute: &'static str,
    /// What it says.
    pub meaning: &'static str,
}

/// The attribute every surface root carries, naming the surface: `[data-surface=bar]`. Written
/// by the consumer's surface root, not by a component.
pub const SURFACE_ATTRIBUTE: &str = "data-surface";

/// The components whose root and parts are public.
pub const COMPONENTS: &[ComponentSelectors] = &[
    ComponentSelectors {
        component: "Alert",
        root: "ds-alert",
        parts: &["icon", "title", "body", "footer"],
    },
    ComponentSelectors {
        component: "Avatar",
        root: "ds-avatar",
        parts: &[],
    },
    ComponentSelectors {
        component: "Badge",
        root: "ds-badge",
        parts: &["label"],
    },
    ComponentSelectors {
        component: "Button",
        root: "ds-button",
        parts: &["icon", "label"],
    },
    ComponentSelectors {
        component: "Checkbox",
        root: "ds-checkbox",
        parts: &["indicator", "label"],
    },
    ComponentSelectors {
        component: "Chip",
        root: "ds-chip",
        parts: &["remove"],
    },
    ComponentSelectors {
        component: "EmptyState",
        root: "ds-empty-state",
        parts: &["icon", "title", "body", "action"],
    },
    ComponentSelectors {
        component: "FactList",
        root: "ds-fact-list",
        parts: &["item", "label", "value"],
    },
    ComponentSelectors {
        component: "HoverCard",
        root: "ds-hovercard",
        parts: &["body"],
    },
    ComponentSelectors {
        component: "InlineBanner",
        root: "ds-inline-banner",
        parts: &["icon", "body", "actions"],
    },
    ComponentSelectors {
        component: "KeyEquivalent",
        root: "ds-key-equivalent",
        parts: &["key"],
    },
    ComponentSelectors {
        component: "Label",
        root: "ds-label",
        parts: &[],
    },
    ComponentSelectors {
        component: "LevelIndicator",
        root: "ds-level-indicator",
        parts: &["track", "fill", "icon"],
    },
    ComponentSelectors {
        component: "Disclosure",
        root: "ds-disclosure",
        parts: &["indicator", "body"],
    },
    ComponentSelectors {
        component: "List",
        root: "ds-list",
        parts: &["item"],
    },
    ComponentSelectors {
        component: "Loadable",
        root: "ds-loadable",
        parts: &["layer", "spin"],
    },
    ComponentSelectors {
        component: "Menu",
        root: "ds-menu",
        parts: &["item", "separator", "header"],
    },
    ComponentSelectors {
        component: "Popover",
        root: "ds-popover",
        parts: &["body", "arrow"],
    },
    ComponentSelectors {
        component: "ProgressIndicator",
        root: "ds-progress",
        parts: &["track", "fill", "indicator", "glyph"],
    },
    ComponentSelectors {
        component: "RadioGroup",
        root: "ds-radio-group",
        parts: &["item", "indicator", "label", "image"],
    },
    ComponentSelectors {
        component: "PopUpButton",
        root: "ds-popup",
        parts: &["sizer"],
    },
    ComponentSelectors {
        component: "Row",
        root: "ds-row",
        parts: &["leading", "title", "detail", "trailing"],
    },
    ComponentSelectors {
        component: "SectionHeader",
        root: "ds-section-header",
        parts: &["title", "value", "action"],
    },
    ComponentSelectors {
        component: "SegmentedControl",
        root: "ds-segmented",
        parts: &["segment", "indicator", "label", "icon"],
    },
    ComponentSelectors {
        component: "Sheet",
        root: "ds-sheet",
        parts: &["body"],
    },
    ComponentSelectors {
        component: "SidePanel",
        root: "ds-side-panel",
        parts: &["header", "body"],
    },
    ComponentSelectors {
        component: "Skeleton",
        root: "ds-skeleton",
        parts: &[],
    },
    ComponentSelectors {
        component: "SkeletonRow",
        root: "ds-skeleton-row",
        parts: &["lines"],
    },
    ComponentSelectors {
        component: "Slider",
        root: "ds-slider",
        parts: &["track", "fill", "thumb", "icon", "tick"],
    },
    ComponentSelectors {
        component: "TextField",
        root: "ds-text-field",
        parts: &["frame", "icon", "suffix", "help", "tokens"],
    },
    ComponentSelectors {
        component: "Toast",
        root: "ds-toast",
        parts: &["body", "action"],
    },
    ComponentSelectors {
        component: "Toggle",
        root: "ds-toggle",
        parts: &["track", "indicator"],
    },
    ComponentSelectors {
        component: "Tooltip",
        root: "ds-tooltip",
        parts: &[],
    },
];

/// The attributes a user stylesheet may select on.
pub const AXES: &[Axis] = &[
    Axis {
        attribute: "data-variant",
        meaning: "a component's role or bezel",
    },
    Axis {
        attribute: "data-size",
        meaning: "the `ControlSize` slug: mini, small, regular, large",
    },
    Axis {
        attribute: "data-state",
        meaning: "`Check`: off, on, mixed",
    },
    Axis {
        attribute: "data-selected",
        meaning: "`Selection`",
    },
    Axis {
        attribute: "data-busy",
        meaning: "`Availability::Busy`",
    },
    Axis {
        attribute: "data-availability",
        meaning: "`Availability`: enabled, disabled, busy",
    },
    Axis {
        attribute: "data-pressed",
        meaning: "`PressPhase`: present while a pointer or a key holds the control down",
    },
    Axis {
        attribute: "data-role",
        meaning: "a button's role: normal or destructive",
    },
    Axis {
        attribute: "data-severity",
        meaning: "`Severity` of an inline banner or a label: info, ok, warn, danger",
    },
    Axis {
        attribute: "data-focus",
        meaning: "`FocusStyle`: ring, highlight",
    },
    Axis {
        attribute: "data-activity",
        meaning: "`Activity`, on `.ds`: written as `inactive` while the window is not the one focused, absent while it is",
    },
    Axis {
        attribute: "aria-*",
        meaning: "the state an element exposes to assistive technology",
    },
];

/// Every class a user stylesheet may select on, without the dot: `ds` (the root, where token
/// overrides go), each component's root and each public part. The linter's copy of the table.
pub fn classes() -> Vec<String> {
    std::iter::once("ds".to_string())
        .chain(COMPONENTS.iter().flat_map(|component| {
            std::iter::once(component.root.to_string()).chain(
                component
                    .part_selectors()
                    .into_iter()
                    .map(|part| part[1..].to_string()),
            )
        }))
        .collect()
}

/// Every attribute a user stylesheet may select on: the surface attribute and each axis.
pub fn attributes() -> Vec<String> {
    std::iter::once(SURFACE_ATTRIBUTE.to_string())
        .chain(AXES.iter().map(|axis| axis.attribute.to_string()))
        .collect()
}

/// The page `docs/selectors.md` holds: the table, as markdown.
pub fn markdown() -> String {
    let mut page = String::from(
        "# Public selectors\n\n\
         What a user stylesheet (`~/.config/<app>/style.css`) may rely on. Anything not listed\n\
         here is internal and may change without notice; renaming anything listed here is a\n\
         breaking change for the person's file.\n\n\
         Prefer token overrides (`.ds { --accent: ...; --t-quick: 120ms }`) to selectors: they\n\
         re-theme every surface consistently, and every token variable\n\
         (`--<prefix><slug>`) is public.\n\n",
    );
    page.push_str(&format!(
        "## Surfaces\n\n`[{SURFACE_ATTRIBUTE}=<name>]` is on every surface root.\n\n"
    ));
    page.push_str("## Components\n\n| Component | Root | Parts |\n| --- | --- | --- |\n");
    for component in COMPONENTS {
        let parts = component.part_selectors();
        let parts = match parts.is_empty() {
            true => "none yet".to_string(),
            false => parts
                .iter()
                .map(|part| format!("`{part}`"))
                .collect::<Vec<_>>()
                .join(", "),
        };
        page.push_str(&format!(
            "| {} | `{}` | {parts} |\n",
            component.component,
            component.root_selector()
        ));
    }
    page.push_str("\n## Attributes\n\n| Attribute | Meaning |\n| --- | --- |\n");
    for axis in AXES {
        page.push_str(&format!("| `{}` | {} |\n", axis.attribute, axis.meaning));
    }
    page
}
