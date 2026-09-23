//! quire's renderer-free design system: tokens, palette, icons, components and motion.

use dioxus as _;
use futures_timer as _;
use serde as _;
use serde_json as _;
use thiserror as _;

#[cfg(feature = "lint")]
use cssparser as _;
