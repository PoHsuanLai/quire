//! The host of a document nothing renders: every read is unknown and every write does nothing.

use crate::host::caret::{Caret, FieldSelection, InitialCaret};
use crate::host::document::DocumentHost;
use crate::host::focused::Focused;
use crate::host::found::{Found, SameNode};
use crate::host::hand_back::HandBack;
use crate::host::measure::Measured;
use crate::host::parts::{
    CaretHost, ClickFocusHost, EditHost, FileDropHost, FocusHost, GeometryHost,
};
use crate::host::reveal::Scrolled;
use dioxus::prelude::MountedData;

/// A document with no host: a server render, or a root nobody gave one.
#[derive(Debug, Clone, Default)]
pub struct NoHost {
    hand_back: HandBack,
}

impl DocumentHost for NoHost {
    fn focus(&self) -> &dyn FocusHost {
        self
    }

    fn caret(&self) -> &dyn CaretHost {
        self
    }

    fn geometry(&self) -> &dyn GeometryHost {
        self
    }

    fn click_focus(&self) -> Option<&dyn ClickFocusHost> {
        None
    }

    fn edit(&self) -> Option<&dyn EditHost> {
        None
    }

    fn file_drop(&self) -> Option<&dyn FileDropHost> {
        None
    }
}

impl FocusHost for NoHost {
    fn focus(&self, _: &MountedData) -> Focused {
        Focused::Unknown
    }

    fn blur(&self, _: &MountedData) -> Focused {
        Focused::Unknown
    }

    fn select(&self, _: &MountedData) -> Focused {
        Focused::Unknown
    }

    fn hand_back(&self) -> &HandBack {
        &self.hand_back
    }
}

impl CaretHost for NoHost {
    fn caret(&self, _: &MountedData) -> Caret {
        Caret::Unknown
    }

    fn place_caret(&self, _: &MountedData, _: InitialCaret) -> Focused {
        Focused::Unknown
    }

    fn selection(&self, _: &MountedData) -> FieldSelection {
        FieldSelection::Unknown
    }
}

impl GeometryHost for NoHost {
    fn measure(&self, _: &MountedData) -> Measured {
        Measured::Unknown
    }

    fn reveal(&self, _: &MountedData, _: &MountedData) -> Scrolled {
        Scrolled::Unknown
    }

    fn find(&self, _: &str) -> Found {
        Found::Unreachable
    }

    fn same(&self, _: &MountedData, _: &MountedData) -> SameNode {
        SameNode::Different
    }
}
