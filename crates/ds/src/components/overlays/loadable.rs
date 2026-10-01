//! Loadable: the pane or list that shows a placeholder while its content loads, the content when
//! it is ready, and a Failure `EmptyState` with a Retry when it could not be loaded (design/30
//! section 2.9, sections 1.3 and 2.9's Skeleton and EmptyState rows). It is the one place that
//! decides which of the three is drawn, so every loading pane moves the same way.
//!
//! `phase` says which: [`Phase::Loading`] carries the [`Operation`] that runs (the default
//! placeholder, a `Spinner` at `ControlSize::Regular`, turns only while it is `Running`: R4),
//! [`Phase::Ready`] draws `children`, [`Phase::Failed`] draws the `EmptyState`. When the *kind* of
//! phase changes, the layer that arrives fades in over `--t-quick` (the cross-fade of section 1.3,
//! on the incoming content); a render that leaves the kind alone (a new `Operation`, a new title)
//! plays nothing, and the first render plays nothing. Under Reduced it is the same fade, which
//! moves nothing.
//!
//! Markup: `div.ds-loadable[data-phase]` of one `div.ds-loadable-layer[data-kind][data-enter]`.
//! While loading the root is `aria-busy="true"`; it is a `region` when `common.aria_label` names
//! it. The failure layer is the `EmptyState`'s own `role="status"`.

use super::empty_state::{EmptyForm, EmptyState};
use crate::components::content::text_runs::TextLine;
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_motion::detail::operation::Operation;
use ds_style::tokens::control_size::ControlSize;

/// Where a loadable's content stands.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    /// Being fetched; the operation drives the default placeholder's spinner (R4).
    Loading(Operation),
    /// Here: the `children` are drawn.
    Ready,
    /// It could not be loaded: an `EmptyState` of form `Failure` with this title and description,
    /// and a Retry when the loadable has `onretry`.
    Failed {
        /// The failure's title.
        title: String,
        /// The line under it, whole or as runs.
        description: Option<TextLine>,
    },
}

/// The kind of a [`Phase`], what a cross-fade is keyed on: `data-phase` and `data-kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
enum Kind {
    Loading,
    Ready,
    Failed,
}

impl Phase {
    fn kind(&self) -> Kind {
        match self {
            Phase::Loading(_) => Kind::Loading,
            Phase::Ready => Kind::Ready,
            Phase::Failed { .. } => Kind::Failed,
        }
    }
}

/// A pane that follows `phase`. `placeholder` replaces the default one (a centred regular
/// spinner; a list's `SkeletonRow`s, say); `onretry` adds the Retry button of the failure layer;
/// `children` are what is shown when `Ready`. `common` puts the consumer's `id`, `data-*` and
/// classes on the root and `aria_label` names it.
#[component]
pub fn Loadable(
    phase: Phase,
    #[props(default)] placeholder: Option<Element>,
    #[props(default)] onretry: Option<EventHandler<()>>,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let kind = phase.kind();
    // The kind drawn last, and whether it has ever changed: only a change fades a layer in.
    let mut seen = use_hook(|| CopyValue::new(kind));
    let mut changed = use_hook(|| CopyValue::new(false));
    if *seen.peek() != kind {
        seen.set(kind);
        changed.set(true);
    }
    let enter = (*changed.peek()).then_some("fade");
    let class = common.class("ds-loadable");
    let data = common.data_attributes();
    let busy = (kind == Kind::Loading).then_some("true");
    let region = common.aria_label.as_ref().map(|_| "region");
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: region,
            "aria-label": common.aria_label.clone(),
            "aria-busy": busy,
            "data-phase": kind.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            match phase {
                Phase::Loading(operation) => rsx! {
                    div { class: "ds-loadable-layer", "data-kind": "loading", "data-enter": enter,
                        if let Some(placeholder) = placeholder {
                            {placeholder}
                        } else {
                            div { class: "ds-loadable-spin",
                                ProgressIndicator {
                                    style: ProgressStyle::Spinner,
                                    progress: Progress::Unknown(operation),
                                    size: ControlSize::Regular,
                                    common: Common { aria_label: Some("Loading".to_owned()), ..Common::default() },
                                }
                            }
                        }
                    }
                },
                Phase::Ready => rsx! {
                    div { class: "ds-loadable-layer", "data-kind": "ready", "data-enter": enter, {children} }
                },
                Phase::Failed { title, description } => rsx! {
                    div { class: "ds-loadable-layer", "data-kind": "failed", "data-enter": enter,
                        EmptyState { form: EmptyForm::Failure, title, description, onretry }
                    }
                },
            }
        }
    }
}
