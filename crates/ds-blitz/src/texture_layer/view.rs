//! The components: [`TextureLayer`] and the [`use_gpu`] hook. They read the model and hand it to
//! the widget; no decision beyond that.

use super::gpu::Gpu;
use super::model::{Pace, Sampling, TexelRect, TextureFit};
use super::watch::Watching;
use super::widget::{LayerProps, LayerWidget};
use crate::node_ref::NodeRef;
use dioxus::prelude::*;
use dioxus_native::CustomWidgetAttr;
use ds::prelude::Word;
use ds::root::common::Common;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The GPU of the window this component is in: the one device its renderer draws with, found
/// where the renderer made it. The window provides one at its root; under a root that did not
/// (a test document, a popup's), the first caller provides it for the components below it.
///
/// The device arrives with the window's first frame, so [`Gpu::device`] is `None` on the first
/// render, and the component calling this re-renders once it is there (and again if the renderer
/// ever gives the window another). An app that makes textures does it in a render that sees
/// `Some`, or on a thread it hands the `Gpu` to.
pub fn use_gpu() -> Gpu {
    let gpu =
        use_hook(|| try_consume_context::<Gpu>().unwrap_or_else(|| provide_context(Gpu::empty())));
    let subscription = use_hook(|| gpu.subscribe(dioxus::core::schedule_update()));
    let owner = gpu.clone();
    use_drop(move || owner.unsubscribe(subscription));
    gpu
}

/// A texture shown inside the document: it fills its parent's box (`width` and `height` 100%) as
/// `fit` says, showing `source` (the whole texture when `None`), and draws whatever `texture`
/// holds at each paint, so an app that replaces or updates the texture needs no re-render.
/// `texture` of `None`, or an empty handle, draws nothing.
///
/// The picture sits in the document's paint order like an image: later siblings draw over it,
/// CSS `opacity`, `filter`, `mask` and `border-radius` clipping apply, and it is hit-tested as
/// the element it is. Its alpha is the texture's, premultiplied, composited over what is beneath.
/// On the CPU painter (no GPU) it draws nothing. `common` puts the consumer's `id`, `data-*`
/// and classes on it; it is decorative (`aria-hidden`) unless `aria_label` names it.
#[component]
pub fn TextureLayer(
    #[props(default)] texture: Option<super::gpu::TextureHandle>,
    #[props(default)] fit: TextureFit,
    #[props(default)] source: Option<TexelRect>,
    #[props(default)] sampling: Sampling,
    #[props(default)] pace: Pace,
    #[props(default)] common: Common,
) -> Element {
    let gpu = use_gpu();
    let props = use_hook(|| {
        Rc::new(RefCell::new(LayerProps {
            texture: None,
            fit,
            source,
            sampling,
            pace,
        }))
    });
    let widget = use_hook({
        let shared = Rc::clone(&props);
        move || CustomWidgetAttr::new(LayerWidget::new(gpu, shared))
    });
    let watching = use_hook(|| Rc::new(RefCell::new(Watching::default())));
    let next = LayerProps {
        texture,
        fit,
        source,
        sampling,
        pace,
    };
    if *props.borrow() != next {
        let texture = next.texture.clone();
        *props.borrow_mut() = next;
        let mut watching = watching.borrow_mut();
        watching.showing(texture);
        watching.repaint();
    }
    let released = Rc::clone(&watching);
    use_drop(move || released.borrow_mut().release());
    let named = common.aria_label.clone();
    let decorative = named.is_none().then_some("true");
    let data = common.data_attributes();
    let class = common.class("ds-texture-layer");
    let mounted = Rc::clone(&watching);
    rsx! {
        object {
            id: common.id.clone(),
            class,
            role: "img",
            data: widget,
            "data-fit": fit.slug(),
            "data-sampling": sampling.slug(),
            "data-pace": pace.slug(),
            "aria-hidden": decorative,
            "aria-label": named,
            style: "display:block;width:100%;height:100%",
            onmounted: move |event| {
                let shell = NodeRef::of(&event.data())
                    .and_then(|node| node.read(|doc| Arc::clone(&doc.shell_provider)));
                if let Some(shell) = shell {
                    mounted.borrow_mut().in_window(shell);
                }
                common.mounted(event);
            },
            ..data,
        }
    }
}

/// A layer no bigger than a pixel and fully transparent, mounted at a window's root so the
/// renderer paints a widget on its first frame and hands the window's [`Gpu`] over before any
/// app layer exists.
#[component]
pub(crate) fn GpuProbe() -> Element {
    rsx! {
        div {
            style: "position:absolute;left:0;top:0;width:1px;height:1px;opacity:0;pointer-events:none;overflow:hidden",
            TextureLayer {}
        }
    }
}
