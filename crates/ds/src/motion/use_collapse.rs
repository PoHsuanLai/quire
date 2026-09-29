//! Collapse (design/30-CATALOGUE.md section 1.3): a disclosure's content opens and closes by
//! its height and opacity over `--t-move --e-in-out`, from the height the caller measured.
//! Under Reduced it is instant.

use crate::core::geometry::units::Px;
use crate::core::vocab::{Fraction, Shown};
use crate::motion::detail::tween::{TweenSpec, use_tween};
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
use dioxus::prelude::*;

/// How open the content is, thousandths: 0 closed, 1000 open.
const OPEN: Fraction = Fraction(1000);

/// Where a collapsing block stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Collapse {
    open: Fraction,
}

impl Collapse {
    /// How open the content is, 0 to 1000.
    pub fn open(self) -> Fraction {
        self.open
    }

    /// Whether it is between open and closed, so a frame is wanted.
    pub fn is_moving(self) -> bool {
        self.open.0 > 0 && self.open.0 < OPEN.0
    }

    /// Whether nothing of it shows.
    pub fn is_closed(self) -> bool {
        self.open.0 == 0
    }

    /// The content's `style`: nothing at rest open (it takes its natural size), else the height
    /// and opacity that share of `measured` gives. Closed it is cut to nothing.
    pub fn style(self, measured: Px) -> Option<String> {
        match self.open.0 {
            1000.. => None,
            0 => Some("height:0px;overflow:hidden;opacity:0".to_owned()),
            open => {
                let share = f32::from(open) / 1000.0;
                Some(format!(
                    "height:{:.2}px;overflow:hidden;opacity:{share:.3}",
                    measured.0 * share
                ))
            }
        }
    }
}

/// Follow `shown`: it stands where `shown` says on mount, and each change moves it there from
/// wherever it is now, so a toggle mid-move reverses without a jump.
pub fn use_collapse(shown: Shown) -> Collapse {
    let target = match shown {
        Shown::Visible => OPEN,
        Shown::Hidden => Fraction(0),
    };
    let open = use_tween(
        target,
        TweenSpec {
            duration: DurationToken::Move,
            easing: EasingToken::InOut,
        },
    );
    Collapse { open }
}

#[cfg(test)]
mod tests {
    use super::{Collapse, OPEN};
    use crate::core::geometry::units::Px;
    use crate::core::vocab::Fraction;

    #[test]
    fn a_collapse_draws_its_share_of_the_measured_height() {
        const CASES: &[(u16, Option<&str>)] = &[
            (1000, None),
            (0, Some("height:0px;overflow:hidden;opacity:0")),
            (500, Some("height:40.00px;overflow:hidden;opacity:0.500")),
            (250, Some("height:20.00px;overflow:hidden;opacity:0.250")),
        ];
        for &(open, want) in CASES {
            let collapse = Collapse {
                open: Fraction(open),
            };
            assert_eq!(collapse.style(Px(80.0)).as_deref(), want, "open {open}");
        }
    }

    #[test]
    fn only_between_the_ends_is_it_moving() {
        assert!(!Collapse { open: OPEN }.is_moving());
        assert!(!Collapse { open: Fraction(0) }.is_moving());
        assert!(Collapse { open: Fraction(1) }.is_moving());
        assert!(
            Collapse {
                open: Fraction(999)
            }
            .is_moving()
        );
    }
}
