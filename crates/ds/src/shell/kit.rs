//! The shell's kit: its metric tokens and paints. Its sheets ride in the components section.

use crate::shell::tokens::{
    control_center::ControlCenterSize, dock::DockToken, notifications::NotificationToken,
    osd::OsdToken, shell_scale::ShellSize, shell_type::ShellType, widget_paint::WidgetPaint,
    widgets::WidgetGrid,
};
use crate::style::kit::{Kit, KitRank, Vocabulary};
use crate::style::tokens::set::{Place, TokenSet};

/// The shell's contribution to the stylesheet and the linter.
pub(crate) static KIT: Kit = Kit {
    rank: KitRank::Shell,
    tokens: &[
        TokenSet::of::<WidgetPaint>().at(Place::Scheme),
        TokenSet::of::<ShellSize>().at(Place::Ladder),
        TokenSet::of::<ControlCenterSize>().at(Place::Ladder),
        TokenSet::of::<ShellType>().at(Place::Metrics),
        TokenSet::of::<DockToken>().at(Place::Metrics),
        TokenSet::of::<OsdToken>().at(Place::Metrics),
        TokenSet::of::<NotificationToken>().at(Place::Metrics),
        TokenSet::of::<WidgetGrid>().at(Place::Metrics),
    ],
    sections: &[],
    vocabulary: Vocabulary::NONE,
};
