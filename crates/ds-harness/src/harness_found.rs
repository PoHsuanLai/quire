//! Reading an element the way an app does: the host finds it by selector, then measures the
//! handle it got.

use crate::harness::Harness;
use ds::host::found::Found;
use ds::host::measure::Measured;
use ds_blitz::seam::{DocRef, focus_finder, measure_element};
use std::rc::Rc;

impl Harness {
    /// The border box of the first element matching `selector`, through the host's `find` and
    /// then its `measure`: what `use_document_host().geometry()` answers a component. `Unknown`
    /// when nothing matches or the selector is not valid.
    pub fn measure_found(&self, selector: &str) -> Measured {
        let doc = DocRef::Cell(Rc::clone(&self.doc.doc.inner));
        match focus_finder(move || Some(doc.clone()))(selector) {
            Found::Element(element) => measure_element(&element),
            Found::Busy => Measured::Busy,
            Found::Missing | Found::BadSelector | Found::Unreachable => Measured::Unknown,
        }
    }
}
