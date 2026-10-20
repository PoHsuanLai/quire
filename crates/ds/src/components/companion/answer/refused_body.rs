//! A refusal: why there is no answer, in words the person can act on.

use super::refused::RefusedView;
use crate::components::content::text_runs::TextLine;
use crate::components::overlays::inline_banner::InlineBanner;
use crate::root::common::Common;
use crate::root::pass_through::{DataAttr, DataName};
use dioxus::prelude::*;
use ds_core::vocab::Severity;

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

/// How serious each refusal is: a failure is a danger; a limit or a rule the person set is a
/// warning; the rest tell what would have to change.
fn severity(why: &RefusedView) -> Severity {
    match why {
        RefusedView::Failed { .. } => Severity::Danger,
        RefusedView::NotAllowed { .. } | RefusedView::OverBudget => Severity::Warn,
        RefusedView::NeedsCloud { .. } | RefusedView::NoWay { .. } => Severity::Info,
    }
}

/// The refusal as the card body: an `InlineBanner` that says the headline and, under it, the
/// sentence, with `data-why` on it.
#[component]
pub fn RefusedBody(why: RefusedView) -> Element {
    let (head, detail) = words(&why);
    let data = DataName::parse("why")
        .map(|name| vec![DataAttr::new(name, why_slug(&why))])
        .unwrap_or_default();
    rsx! {
        InlineBanner {
            severity: severity(&why),
            text: head,
            detail: TextLine::from(detail),
            common: Common { data, ..Common::default() },
        }
    }
}
