//! How an item leaves: the `data-exit` word and the animation it plays.

use crate::core::vocab::Emphasis;
use crate::core::word::Word;
use crate::motion::anim::Anim;

/// How an item leaves: `data-exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Exit {
    /// Archive, restore, wake: `fold`.
    Fold,
    /// Snooze: `curl`.
    Curl,
    /// Trash, delete: `crumple`.
    Crumple,
    /// A Today entry closing: `tab-out`.
    TabOut,
    /// A notification banner leaving: `banner-out`, a slide to the right.
    BannerOut,
    /// The OSD card fading and lifting away: `osd-out`.
    OsdOut,
    /// A screenshot thumbnail sliding out to the right: `shot-out`.
    ShotOut,
    /// A preview pane sliding out to the right: `pane-out`.
    PaneOut,
}

impl Exit {
    /// The animation this exit plays: an unread (`Emphasis::Strong`) row plays the heavy variant
    /// of every row exit, 15 % slower (design/05-MOTION.md principle 6, section 8). The other
    /// exits have no heavy variant.
    pub(crate) fn anim(self, emphasis: Emphasis) -> Anim {
        match (self, emphasis) {
            (Exit::Fold, Emphasis::Strong) => Anim::FoldHeavy,
            (Exit::Fold, Emphasis::Plain) => Anim::Fold,
            (Exit::Curl, Emphasis::Strong) => Anim::CurlHeavy,
            (Exit::Curl, Emphasis::Plain) => Anim::Curl,
            (Exit::Crumple, Emphasis::Strong) => Anim::CrumpleHeavy,
            (Exit::Crumple, Emphasis::Plain) => Anim::Crumple,
            (Exit::TabOut, _) => Anim::TabOut,
            (Exit::BannerOut, _) => Anim::BannerOut,
            (Exit::OsdOut, _) => Anim::OsdOut,
            (Exit::ShotOut, _) => Anim::ShotOut,
            (Exit::PaneOut, _) => Anim::PaneOutR,
        }
    }
}
