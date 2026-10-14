//! A refusal: why there is no answer, in words the person can act on.

use super::refused::RefusedView;
use dioxus::prelude::*;

/// The headline and the sentence under it.
fn words(why: &RefusedView) -> (String, String) {
    match why {
        RefusedView::NeedsCloud { class } => (
            "This needs a cloud model".to_owned(),
            format!("Your {class} would leave this computer. You can allow it just this once."),
        ),
        RefusedView::NotAllowed { what } => (
            "Not allowed".to_owned(),
            format!("{what} is not allowed here."),
        ),
        RefusedView::NoWay { app } => (
            format!("Can't do that in {app}"),
            format!(
                "{app} offers no way to do it directly, and controlling it by sight is turned off."
            ),
        ),
        RefusedView::OverBudget => (
            "Limit reached".to_owned(),
            "The limit you set for this has been used up.".to_owned(),
        ),
        RefusedView::Failed { why } => ("That didn't work".to_owned(), why.clone()),
    }
}

/// `data-why`.
fn why_slug(why: &RefusedView) -> &'static str {
    match why {
        RefusedView::NeedsCloud { .. } => "needs-cloud",
        RefusedView::NotAllowed { .. } => "not-allowed",
        RefusedView::NoWay { .. } => "no-way",
        RefusedView::OverBudget => "over-budget",
        RefusedView::Failed { .. } => "failed",
    }
}

/// The refusal as the card body.
#[component]
pub fn RefusedBody(why: RefusedView) -> Element {
    let (head, detail) = words(&why);
    rsx! {
        div { class: "ds-card-refused", "data-why": why_slug(&why), role: "alert",
            div { class: "ds-card-refused-head", "{head}" }
            p { class: "ds-card-line", "{detail}" }
        }
    }
}
