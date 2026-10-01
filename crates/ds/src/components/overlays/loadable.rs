//! Loadable: the pane or list that shows a placeholder while its content loads, the content when
//! it is ready, and a Failure `EmptyState` with a Retry when it could not be loaded (design/30
//! section 2.9: a composition of `ProgressIndicator`, `EmptyState` (Failure) and the cross-fade
//! primitive of section 1.3, not a new look). It is the one place that decides which of the three
//! is drawn, so every loading pane moves the same way.
//!
//! `phase` says which: [`Phase::Loading`] carries the [`Operation`] that runs (the default
//! placeholder, a `Spinner` at `ControlSize::Regular`, turns only while it is `Running`: R4),
//! [`Phase::Ready`] draws `children`, [`Phase::Failed`] draws the `EmptyState`.
//!
//! The swap is the cross-fade primitive (`use_cross_fade`, design/30 section 1.3), fed by a cue
//! `use_detail` makes from the phase's kind (the way a caller of `PreviewPane` feeds its
//! `preview_cue`): when the *kind* of phase changes, the layer that arrives fades in over
//! `--t-quick`, once per change; a render that leaves the kind alone (a new `Operation`, a new
//! title) plays nothing, and the first render plays nothing. A custom `placeholder` that is still
//! passed when the phase leaves `Loading` stays drawn over the arriving layer until it has faded
//! out over `--t-quick` (`menu-out`, a Presence exit), so a skeleton-to-content swap is a real
//! cross-fade; the default spinner is simply replaced. Under Reduced (design/26 R7) the swap is
//! the final state at once: nothing fades in and the placeholder goes at once.
//!
//! Markup: `div.ds-loadable[data-phase]` of one `div.ds-loadable-layer[data-kind]` (plus, while a
//! custom placeholder fades out, the loading layer with `data-presence="leaving"` over it). While
//! loading the root is `aria-busy="true"`; it is a `region` when `common.aria_label` names it.
//! The failure layer is the `EmptyState`'s own `role="status"`.

use super::empty_state::{EmptyForm, EmptyState};
use crate::components::content::text_runs::TextLine;
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::detail::{
    detailed::Detailed, level::use_level, moment::Moment, once::use_cross_fade,
    operation::Operation, touch::Touch, use_detail::use_detail,
};
use ds_motion::presence::{
    Exit, Presence,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::appearance::motion::MotionLevel;
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

/// What a change of kind means to the cross-fade primitive: a failure arrives as a Failure, any
/// other swap as a Change. The first frame is at rest (nothing plays on mount).
impl Detailed for Kind {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Kind::Loading | Kind::Ready | Kind::Failed, Kind::Failed) => Moment::Failure,
            (Kind::Loading | Kind::Failed, Kind::Ready)
            | (Kind::Ready | Kind::Failed, Kind::Loading) => Moment::Change,
            (Kind::Loading, Kind::Loading) | (Kind::Ready, Kind::Ready) => Moment::Rest,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Kind::Loading | Kind::Ready | Kind::Failed => Moment::Rest,
        }
    }
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
/// spinner; a list's `SkeletonRow`s, say); `onretry` adds the Retry button of the failure layer,
/// and `action` is the caller's own element beside it (a secondary "Connection Doctor…" button:
/// the failure `EmptyState`'s `action` slot, drawn before Retry);
/// `children` are what is shown when `Ready`. `common` puts the consumer's `id`, `data-*` and
/// classes on the root and `aria_label` names it.
#[component]
pub fn Loadable(
    phase: Phase,
    #[props(default)] placeholder: Option<Element>,
    #[props(default)] onretry: Option<EventHandler<()>>,
    #[props(default)] action: Option<Element>,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let kind = phase.kind();
    let cue = use_detail(kind, Touch::Remote).cue();
    let (arriving, pulse) = use_cross_fade(Some(cue)).wear("ds-loadable-layer");
    // The loading layer's own life, only so a custom placeholder can leave by a fade.
    let Presented { presence, .. } = use_presence(
        if kind == Kind::Loading {
            Shown::Visible
        } else {
            Shown::Hidden
        },
        PresenceSpec {
            enter: Anim::Fade,
            exit: Exit::Fade,
        },
        None,
    );
    let reduced = use_level().now() == MotionLevel::Reduced;
    let leaving = kind != Kind::Loading
        && placeholder.is_some()
        && !reduced
        && matches!(presence, Presence::Leaving(_));
    let operation = match &phase {
        Phase::Loading(operation) => *operation,
        Phase::Ready | Phase::Failed { .. } => Operation::Idle,
    };
    let class = common.class("ds-loadable");
    let data = common.data_attributes();
    let busy = (kind == Kind::Loading).then_some("true");
    let region = common.aria_label.as_ref().map(|_| "region");
    let (loading_class, loading_pulse) = if leaving {
        ("ds-loadable-layer".to_owned(), None)
    } else {
        (arriving.clone(), pulse)
    };
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
            if kind == Kind::Loading || leaving {
                div {
                    class: loading_class,
                    "data-kind": "loading",
                    "data-pulse": loading_pulse,
                    "data-presence": leaving.then_some("leaving"),
                    "aria-hidden": leaving.then_some("true"),
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
            }
            match phase {
                Phase::Loading(_) => rsx! {},
                Phase::Ready => rsx! {
                    div { class: arriving, "data-kind": "ready", "data-pulse": pulse, {children} }
                },
                Phase::Failed { title, description } => rsx! {
                    div { class: arriving, "data-kind": "failed", "data-pulse": pulse,
                        EmptyState { form: EmptyForm::Failure, title, description, action, onretry }
                    }
                },
            }
        }
    }
}
