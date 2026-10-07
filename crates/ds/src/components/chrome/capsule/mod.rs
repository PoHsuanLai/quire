//! Capsule: the floating pill of controls over content (design/30 section 2.7a): the viewer's
//! hover controls, a mini player's, a photo's zoom bar. Toolbar buttons and readouts in a
//! `Material::Osd` card, bottom centre of the content it floats over, faded in and out by its
//! owner's `shown`. Too narrow a stage drops its lower-priority slots (`fit`).

pub mod fit;
pub mod model;
pub mod priority;
pub mod view;
