//! A scope for a case that renders without a `Ds` above it: a component that reads one (a
//! switch, a spinner, a disclosure) finds light, Postmark and Reduced motion.

use dioxus::prelude::*;

/// The scope a component reads when nothing above it is a `Ds`: light, Postmark, Reduced motion
/// (every settle 94 ms).
pub fn scope() -> ds::Scope {
    ds::Scope {
        resolved: ds::Resolved {
            scheme: ds::Scheme::Light,
            accent: ds::Accent::Blue,
            motion: ds::MotionLevel::Reduced,
        },
        scheme: ds::Scheme::Light,
        material: ds::Material::Window,
        blur: ds::BlurState::default(),
        modality: ds::InputModality::Pointer,
        activity: ds::Activity::Active,
    }
}

/// `children` under that scope. It draws nothing of its own.
#[component]
pub fn Scoped(children: Element) -> Element {
    use_context_provider(|| Signal::new(scope()));
    children
}
