//! Edit Widgets' Add buttons settle to a check when the widget they added lands
//! (design/26-DETAILS.md's Success, `SettleStyle::Check`): the label gives way to a check and the
//! gallery's word for it ("Added"), the check drawn on over `--t-move --e-out` and held
//! `SettleHold`, then the button is itself again. The check grows in with `morph-in`, on the
//! spring only when the person's own press added it (design/05 principle 2, design/26 R5).
//! Under Reduced the whole check shows for its hold, with no draw and no growth.

use crate::components::button::{Button, ButtonVariant};
use crate::components::button_face::Leading;
use crate::components::press::Press;
use crate::components::text_runs::Text;
use crate::detail::{
    CheckMark, Detailed, FirstShow, Moment, SettleStyle, Settling, Touch, use_detail, use_settle,
};
use crate::icon::render::IconSize;
use crate::motion::Anim;
use crate::widget::gallery_book::Landing;
use dioxus::prelude::*;

/// The adds of one widget to one surface that have landed, as the button sees them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Landed {
    /// None since the button was drawn.
    Never,
    /// The latest, counted from 1.
    Last(u32),
}

impl Landed {
    fn of(landing: Landing) -> Self {
        match landing.serial {
            0 => Landed::Never,
            serial => Landed::Last(serial),
        }
    }
}

impl Detailed for Landed {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Landed::Never | Landed::Last(_), Landed::Last(_)) => Moment::Success,
            (Landed::Never | Landed::Last(_), Landed::Never) => Moment::Rest,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Landed::Never | Landed::Last(_) => Moment::Rest,
        }
    }
}

/// The check's growth: the spring for the person's own press, `--e-out` otherwise.
fn growth(touch: Touch) -> Anim {
    match touch {
        Touch::Contact(_) => Anim::MorphInSpring,
        Touch::Remote => Anim::MorphIn,
    }
}

/// An Add button: `label`, or, while the latest `landing` settles, a check and `added`.
#[component]
pub(crate) fn AddButton(
    variant: ButtonVariant,
    label: Text,
    added: Text,
    landing: Landing,
    onclick: EventHandler<Press>,
) -> Element {
    let detail = use_detail(Landed::of(landing), FirstShow::Still, landing.touch);
    let settling = use_settle(detail.cue(), SettleStyle::Check);
    let (label, leading) = match settling {
        Settling::Drawing(_) => {
            let grow = growth(detail.cue().touch());
            let mark = rsx! {
                span { class: "ds-widget-gallery-added {grow.class()}", "data-pulse": "a",
                    CheckMark { settling, size: IconSize::Compact }
                }
            };
            (added, Some(Leading::Mark(mark)))
        }
        Settling::Rest | Settling::Filling(_) | Settling::Sealing(_) => (label, None),
    };
    rsx! {
        Button { variant, label, leading, onclick }
    }
}

#[cfg(test)]
mod tests {
    use super::{Landed, growth};
    use crate::detail::{Contact, Moment, Touch, first_table, moment_table};
    use crate::motion::Anim;

    #[test]
    fn a_new_landing_is_a_success_and_nothing_else_is() {
        moment_table(&[
            (Landed::Never, Landed::Last(1), Moment::Success),
            (Landed::Last(1), Landed::Last(2), Moment::Success),
            (Landed::Last(2), Landed::Never, Moment::Rest),
        ]);
        first_table(&[
            (Landed::Never, Moment::Rest),
            (Landed::Last(3), Moment::Rest),
        ]);
    }

    #[test]
    fn only_the_persons_press_springs() {
        assert_eq!(
            growth(Touch::Contact(Contact::for_tests())),
            Anim::MorphInSpring
        );
        assert_eq!(growth(Touch::Remote), Anim::MorphIn);
    }
}
