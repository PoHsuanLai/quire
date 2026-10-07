//! The glow's data: its looks are words the compositor and quire agree on.

use ds_core::geometry::units::Px;
use ds_style::tokens::glow::{GlowLook, GlowSpot};

#[test]
fn an_acting_glow_may_name_the_spot_it_acts_at() {
    let spot = GlowSpot {
        x: Px(120.0),
        y: Px(48.0),
        radius: Px(64.0),
    };
    let with = GlowLook::Acting { spot: Some(spot) };
    let without = GlowLook::Acting { spot: None };
    assert_ne!(with, without);
    assert_ne!(GlowLook::Working, GlowLook::Waiting);
    assert_ne!(GlowLook::None, without);
}
