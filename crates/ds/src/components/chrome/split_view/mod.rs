//! SplitView: `NSSplitView` (design/30 section 2.7): panes side by side, a hairline divider with
//! a 6 px drag zone between them, each pane held to its widths, a collapsible pane that folds
//! away on a spring, and a double-click on a divider that resets its pane.

pub mod model;
pub(crate) mod pane;
pub mod view;
