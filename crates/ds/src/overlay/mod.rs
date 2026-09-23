//! Floating things render at the end of `.ds`, never inside the component that opened them:
//! `position:fixed` breaks inside transformed ancestors (design/05-MOTION.md section 9 rule 10).

pub mod host;
pub mod hover_hub;
pub mod stack;
pub mod toast_hub;

pub use host::{OverlayHost, OverlayId, Overlays, use_overlays};
pub use hover_hub::{HoverHub, HoverKey, HoverKind, HoverWarmth, use_hover_hub};
pub use stack::{Dismissal, LayerId, LayerStack};
pub use toast_hub::{ToastHub, ToastState, UndoToken, use_toast_hub};
