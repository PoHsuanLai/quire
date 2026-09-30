//! How an item leaves: the `data-exit` word and the animation it plays.

use crate::anim::Anim;
use ds_core::word::Word;

/// How an item leaves: `data-exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Exit {
    /// A list row leaving (archive, delete, dismissed): it fades and slides up, `row-out`.
    Row,
    /// A notification banner leaving: `banner-out`, a slide to the right.
    BannerOut,
    /// The OSD card fading and lifting away: `osd-out`.
    OsdOut,
    /// A screenshot thumbnail sliding out to the right: `shot-out`.
    ShotOut,
    /// A preview pane sliding out to the right: `pane-out`.
    PaneOut,
    /// A popover, tooltip or hover card fading away: `menu-out`, `--t-quick`.
    Fade,
    /// A sheet sliding back up past the top edge: `sheet-out`.
    SheetOut,
    /// A side panel or a toast sliding out to the right: `panel-out`.
    PanelOut,
}

impl Exit {
    /// The animation this exit plays.
    pub(crate) fn anim(self) -> Anim {
        match self {
            Exit::Row => Anim::RowOut,
            Exit::BannerOut => Anim::BannerOut,
            Exit::OsdOut => Anim::OsdOut,
            Exit::ShotOut => Anim::ShotOut,
            Exit::PaneOut => Anim::PaneOutR,
            Exit::Fade => Anim::MenuOut,
            Exit::SheetOut => Anim::SheetOut,
            Exit::PanelOut => Anim::PanelOut,
        }
    }
}
