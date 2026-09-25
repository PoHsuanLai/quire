//! GroupHeader: the head of one app's notifications in the notification center (sill Q125;
//! design/20 section 1.6, design/13 section 13.3.6 "Grouping by app"). The app's icon at 16,
//! its name and how many it holds in `SectionHeader`'s group type (the data face in capitals at
//! the caption size, `--ink-faint`), then the trailing pair: the toggle that folds the group to
//! its newest ("N more" while folded, "Show less" while open) and Clear. Both are Quiet buttons
//! that keep their press, so a header inside a pressable group row never also presses the row.

use crate::components::button::{Button, ButtonVariant};
use crate::components::icon_view::IconView;
use crate::components::press::{Press, Propagation};
use crate::components::text_runs::{Text, text};
use crate::components::vocab::Expanded;
use crate::icon::external::IconSource;
use crate::icon::render::IconSize;
use dioxus::prelude::*;

/// The toggle's words: "Show less" while the group is open; while folded to its newest, how
/// many are hidden ("2 more"), or nothing when there is nothing to show.
fn toggle_label(expanded: Expanded, count: u32) -> Option<String> {
    match (expanded, count) {
        (Expanded::Open, _) => Some("Show less".to_owned()),
        (Expanded::Closed, 0 | 1) => None,
        (Expanded::Closed, n) => Some(format!("{} more", n - 1)),
    }
}

/// One app's group header. `count` is how many notifications the group holds; `expanded` is
/// whether the group shows them all (`aria-expanded` on the toggle). `on_toggle` folds or opens
/// it; `on_clear` clears the group.
#[component]
pub fn GroupHeader(
    icon: IconSource,
    #[props(into)] name: Text,
    count: u32,
    #[props(default)] expanded: Expanded,
    on_toggle: EventHandler<Press>,
    on_clear: EventHandler<Press>,
) -> Element {
    let clear = format!("Clear {}", name.plain_text());
    rsx! {
        div { class: "ds-group-header", "data-expanded": expanded.slug(),
            span { class: "ds-group-header-icon",
                IconView { source: icon, size: IconSize::Base }
            }
            span { class: "ds-group-header-name", {text(&name)} }
            if count > 0 {
                span { class: "ds-group-header-count", "{count}" }
            }
            span { class: "ds-group-header-actions",
                if let Some(label) = toggle_label(expanded, count) {
                    Button {
                        variant: ButtonVariant::Quiet,
                        label,
                        expanded,
                        propagation: Propagation::Stop,
                        onclick: move |press| on_toggle.call(press),
                    }
                }
                Button {
                    variant: ButtonVariant::Quiet,
                    label: "Clear",
                    aria_label: clear,
                    propagation: Propagation::Stop,
                    onclick: move |press| on_clear.call(press),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::toggle_label;
    use crate::components::vocab::Expanded;

    #[test]
    fn the_toggle_names_what_it_will_do() {
        let cases = [
            (Expanded::Open, 4, Some("Show less")),
            (Expanded::Open, 1, Some("Show less")),
            (Expanded::Closed, 4, Some("3 more")),
            (Expanded::Closed, 2, Some("1 more")),
            (Expanded::Closed, 1, None),
            (Expanded::Closed, 0, None),
        ];
        for (expanded, count, want) in cases {
            assert_eq!(
                toggle_label(expanded, count).as_deref(),
                want,
                "{expanded:?} {count}"
            );
        }
    }
}
