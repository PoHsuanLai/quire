//! `snap_to_device` leaves a host idle: on a real headless document it never raises the shell's
//! redraw request or starts an animation, a second call changes nothing, and at a whole scale it
//! changes nothing at all.

use crate::headless::Headless;
use crate::snap::snap_to_device;
use crate::snapshot::Viewport;
use blitz_dom::{BaseDocument, NodeData};
use blitz_traits::shell::ShellProvider;
use dioxus::prelude::*;
use ds::{Appearance, Ds, Glyph, Icon, IconSize, Material};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Odd logical coordinates, a sub-pixel border, a `translateX(-50%)` of an odd width, a
/// sub-pixel translation, a fractional height and a glyph: everything the snap rewrites at 1.5.
/// The `.3px` translation (at 1x and 2x) and the 10.3 px height (at 2x, where the device grid is
/// half a logical pixel) are what a snap that ran at a whole scale would change.
const FIXTURE_CSS: &str = ".a{position:absolute;left:11px;top:11px;width:61px;height:var(--hair);\
    background:var(--line)}\
    .b{position:absolute;left:21px;top:31px;width:61px;height:31px;border:var(--hair) solid var(--line)}\
    .c{position:absolute;left:50%;top:71px;width:61px;height:21px;transform:translateX(-50%);\
    border:var(--hair) solid var(--line)}\
    .d{position:absolute;left:5px;top:101px;width:9px;height:9px;transform:translateX(.3px);\
    background:var(--line)}\
    .e{position:absolute;left:21px;top:5px;width:9px;height:10.3px;background:var(--line)}";

#[allow(non_snake_case)]
fn Fixture() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            style { {FIXTURE_CSS} }
            div { class: "a" }
            div { class: "b", Glyph { icon: Icon::Plus, size: IconSize::Base } }
            div { class: "c" }
            div { class: "d" }
            div { class: "e" }
        }
    }
}

/// A shell provider that counts redraw requests, where the host's would schedule a frame.
#[derive(Default)]
struct Redraws(AtomicUsize);

impl ShellProvider for Redraws {
    fn request_redraw(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Everything the snap may write, per node, exactly: the final layout, the transform and the
/// scrollable overflow (`Debug` prints each float to its shortest exact form).
fn fingerprint(doc: &BaseDocument) -> Vec<String> {
    doc.tree()
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.data,
                NodeData::Element(_) | NodeData::AnonymousBlock(_) | NodeData::Document(_)
            )
        })
        .map(|(id, node)| {
            format!(
                "{id:?} {:?} {:?} {:?}",
                node.final_layout(),
                node.transform(),
                node.scrollable_overflow()
            )
        })
        .collect()
}

/// What two calls did: the fingerprints before, after the first and after the second, and the
/// redraw requests and animation state they left.
struct Calls {
    before: Vec<String>,
    first: Vec<String>,
    second: Vec<String>,
    redraws: usize,
    animating: bool,
    /// The `.d` box's horizontal translation in device pixels, and the `.e` box's height in
    /// logical pixels, after both calls.
    shift: f64,
    height: f32,
}

/// Build the fixture at `scale_percent`, resolve it the way a host does (Blitz's own logical
/// rounding, no snap yet), then call `snap_to_device` twice.
fn snap_twice(scale_percent: u16) -> Calls {
    let viewport = Viewport {
        width: 240,
        height: 120,
        scale_percent,
    };
    let mut headless = Headless::new(Fixture, viewport);
    headless.frame(Duration::from_millis(400));
    let mut doc = headless.doc.inner.borrow_mut();
    doc.resolve(0.4);
    let redraws = Arc::new(Redraws::default());
    doc.set_shell_provider(Arc::clone(&redraws) as Arc<dyn ShellProvider>);
    let before = fingerprint(&doc);
    snap_to_device(&mut doc);
    let first = fingerprint(&doc);
    snap_to_device(&mut doc);
    let second = fingerprint(&doc);
    let node = |selector: &str| {
        let id = doc.query_selector(selector).ok().flatten().expect(selector);
        doc.get_node(id).expect(selector)
    };
    let shift = node(".d")
        .transform()
        .as_deref()
        .map_or(0.0, |t| t.as_coeffs()[4]);
    let height = node(".e").final_layout().size.height;
    Calls {
        shift,
        height,
        before,
        first,
        second,
        redraws: redraws.0.load(Ordering::SeqCst),
        animating: doc.is_animating(),
    }
}

#[test]
fn at_a_fractional_scale_the_snap_is_idempotent_and_asks_for_nothing() {
    let calls = snap_twice(150);
    assert_ne!(
        calls.before, calls.first,
        "the first call must snap something at 1.5"
    );
    assert_eq!(
        calls.first, calls.second,
        "a second call changed the layout"
    );
    assert_eq!(calls.redraws, 0, "the snap asked the shell for a redraw");
    assert!(!calls.animating, "the snap left the document animating");
}

#[test]
fn at_a_whole_scale_the_snap_touches_nothing() {
    for scale in [100, 200] {
        let calls = snap_twice(scale);
        assert_eq!(
            calls.before, calls.first,
            "{scale}%: the first call changed the layout"
        );
        assert_eq!(
            calls.first, calls.second,
            "{scale}%: the second call changed it"
        );
        assert_eq!(calls.redraws, 0, "{scale}%: a redraw was requested");
        assert!(!calls.animating, "{scale}%: the document is animating");
        let (shift, height) = (0.3 * f64::from(scale) / 100.0, 10.0);
        assert!(
            (calls.shift - shift).abs() < 1e-4,
            "{scale}%: shift {}",
            calls.shift
        );
        assert_eq!(
            calls.height, height,
            "{scale}%: Blitz's own rounding of 10.3 px"
        );
    }
}
