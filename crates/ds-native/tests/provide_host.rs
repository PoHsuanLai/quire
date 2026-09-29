//! `ds_native::provide_host` gives a root every part of the document host in one call, so a
//! shell's surface root can never hold a subset, and it keeps the host a window or the harness
//! already wired.

use dioxus::prelude::*;
use ds::{DocumentHost, use_document_host};
use std::cell::RefCell;
use std::rc::Rc;

/// What the roots under test saw, handed out through a root context.
#[derive(Clone, Default)]
struct Seen(Rc<RefCell<Vec<Rc<dyn DocumentHost>>>>);

fn render(root: fn() -> Element) -> Vec<Rc<dyn DocumentHost>> {
    let seen = Seen::default();
    let mut dom = VirtualDom::new(root);
    dom.provide_root_context(seen.clone());
    dom.rebuild_in_place();
    seen.0.borrow().clone()
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    let host = ds_native::provide_host();
    consume_context::<Seen>().0.borrow_mut().push(host);
    rsx! {}
}

#[allow(non_snake_case)]
fn Nested() -> Element {
    let host = ds_native::provide_host();
    consume_context::<Seen>().0.borrow_mut().push(host);
    rsx! { Inner {} }
}

#[allow(non_snake_case)]
fn Inner() -> Element {
    let host = ds_native::provide_host();
    consume_context::<Seen>().0.borrow_mut().push(host);
    rsx! {}
}

#[test]
fn a_bare_root_gets_the_edit_and_drop_parts_but_no_click_focus() {
    let hosts = render(Bare);
    let [host] = hosts.as_slice() else {
        panic!("one root rendered, {} hosts seen", hosts.len());
    };
    assert!(host.edit().is_some(), "edit part");
    assert!(host.file_drop().is_some(), "file drop part");
    assert!(host.click_focus().is_none(), "click focus is a window's");
}

#[test]
fn a_root_under_a_host_keeps_that_host() {
    let hosts = render(Nested);
    let [outer, inner] = hosts.as_slice() else {
        panic!("two roots rendered, {} hosts seen", hosts.len());
    };
    assert!(Rc::ptr_eq(outer, inner), "the inner root shadowed the host");
}

#[test]
fn a_document_without_a_host_has_none_of_the_optional_parts() {
    #[allow(non_snake_case)]
    fn Unhosted() -> Element {
        let host = use_document_host();
        consume_context::<Seen>().0.borrow_mut().push(host);
        rsx! {}
    }
    let hosts = render(Unhosted);
    let [host] = hosts.as_slice() else {
        panic!("one root rendered, {} hosts seen", hosts.len());
    };
    assert!(host.edit().is_none(), "NoHost has no edit part");
    assert!(host.file_drop().is_none(), "NoHost has no drop part");
}
