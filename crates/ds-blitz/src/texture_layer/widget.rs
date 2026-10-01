//! The Blitz widget behind a layer: it learns the renderer's device when the renderer first
//! paints it, registers the layer's texture with the renderer, and records the paint that draws
//! it. Blitz paints a widget with `blitz-paint/custom-widget` (ds-blitz turns it on).

use super::fit::{place, visible_source};
use super::gpu::{Frame, Gpu, Source, TextureHandle};
use super::model::{Pace, Sampling, TexelRect, TextureFit};
use anyrender::{PaintRef, PaintScene, RenderContext, ResourceId, Scene};
use blitz_dom::IntrinsicSizes;
use blitz_dom::Widget;
use blitz_dom::node::ComputedStyles;
use peniko::kurbo::{Affine, Rect};
use peniko::{Fill, ImageBrush, ImageSampler};
use std::cell::RefCell;
use std::rc::Rc;
use wgpu_context::DeviceHandle;

/// What a layer's props say, shared between the component that renders them and the widget that
/// paints them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayerProps {
    pub(crate) texture: Option<TextureHandle>,
    pub(crate) fit: TextureFit,
    pub(crate) source: Option<TexelRect>,
    pub(crate) sampling: Sampling,
    pub(crate) pace: Pace,
}

/// A texture as the renderer holds it.
#[derive(Debug)]
struct Registered {
    id: ResourceId,
    version: u64,
    handle: TextureHandle,
}

/// The widget a layer's `<object>` becomes.
#[derive(Debug)]
pub(crate) struct LayerWidget {
    gpu: Gpu,
    props: Rc<RefCell<LayerProps>>,
    registered: Option<Registered>,
}

impl LayerWidget {
    pub(crate) fn new(gpu: Gpu, props: Rc<RefCell<LayerProps>>) -> Self {
        LayerWidget {
            gpu,
            props,
            registered: None,
        }
    }

    /// Hand the renderer's copy of the texture back.
    fn forget(&mut self, ctx: &mut dyn RenderContext) {
        if let Some(registered) = self.registered.take() {
            ctx.unregister_resource(registered.id);
        }
    }

    /// The renderer's id for `frame`, registering it if the renderer has not got this texture.
    /// `None` where the renderer cannot draw textures (the CPU painter).
    fn resource(
        &mut self,
        ctx: &mut dyn RenderContext,
        handle: &TextureHandle,
        frame: &Frame,
    ) -> Option<ResourceId> {
        if let Some(held) = &self.registered
            && held.version == frame.version
            && held.handle == *handle
        {
            return Some(held.id);
        }
        self.forget(ctx);
        let resource: Box<dyn std::any::Any> = match &frame.source {
            Source::Texture(texture) => Box::new(texture.clone()),
            Source::View(view) => Box::new(view.clone()),
        };
        let id = ctx.try_register_custom_resource(resource).ok()?;
        self.registered = Some(Registered {
            id,
            version: frame.version,
            handle: handle.clone(),
        });
        Some(id)
    }
}

impl Widget for LayerWidget {
    fn can_create_surfaces(&mut self, render_ctx: &mut dyn RenderContext) {
        // The renderer's registrations are gone with the surface this follows.
        self.registered = None;
        if let Some(handle) = render_ctx
            .renderer_specific_context()
            .and_then(|context| context.downcast::<DeviceHandle>().ok())
        {
            self.gpu.attach(&handle);
        }
    }

    fn destroy_surfaces(&mut self) {
        self.registered = None;
    }

    fn requires_redraw(&self) -> bool {
        self.props.borrow().pace == Pace::EveryFrame
    }

    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        let props = self.props.borrow();
        let ratio = props
            .texture
            .as_ref()
            .and_then(TextureHandle::frame)
            .and_then(|frame| visible_source(props.source, frame.width, frame.height))
            .map(|source| source.width.0 as f32 / source.height.0 as f32);
        IntrinsicSizes {
            width: None,
            height: None,
            ratio,
        }
    }

    fn paint(
        &mut self,
        render_ctx: &mut dyn RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> Scene {
        let props = self.props.borrow().clone();
        let mut scene = Scene::new();
        let Some(handle) = props.texture else {
            self.forget(render_ctx);
            return scene;
        };
        let Some(frame) = handle.frame() else {
            self.forget(render_ctx);
            return scene;
        };
        let Some(source) = visible_source(props.source, frame.width, frame.height) else {
            return scene;
        };
        let Some(id) = self.resource(render_ctx, &handle, &frame) else {
            return scene;
        };
        let texture = Rect::new(
            0.0,
            0.0,
            f64::from(frame.width.0),
            f64::from(frame.height.0),
        );
        let brush = ImageBrush {
            image: id,
            sampler: ImageSampler {
                quality: props.sampling.quality(),
                ..ImageSampler::default()
            },
        };
        for draw in place(props.fit, source, (f64::from(width), f64::from(height))) {
            scene.push_clip_layer(Affine::IDENTITY, &draw.clip);
            scene.fill(
                Fill::NonZero,
                draw.transform,
                PaintRef::Resource(brush),
                None,
                &texture,
            );
            scene.pop_layer();
        }
        scene
    }
}
