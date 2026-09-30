//! A scope for a case that renders without a `Ds` above it: a component that reads one (a
//! switch, a spinner, a disclosure) finds light, Postmark and Reduced motion.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_core::vocab::Activity;
use ds_core::vocab::InputModality;
use ds_style::appearance::blur::BlurState;
use ds_style::appearance::resolve::Resolved;
use ds_style::scope::Scope;

/// The scope a component reads when nothing above it is a `Ds`: light, Postmark, Reduced motion
/// (every settle 94 ms).
pub fn scope() -> Scope {
    Scope {
        resolved: Resolved {
            scheme: Scheme::Light,
            accent: Accent::Blue,
            motion: MotionLevel::Reduced,
        },
        scheme: Scheme::Light,
        material: Material::Window,
        blur: BlurState::default(),
        modality: InputModality::Pointer,
        activity: Activity::Active,
    }
}

/// `children` under that scope. It draws nothing of its own.
#[component]
pub fn Scoped(children: Element) -> Element {
    use_context_provider(|| Signal::new(scope()));
    children
}
