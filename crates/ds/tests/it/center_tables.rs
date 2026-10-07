//! design/26: the pieces the control center modules draw with: the filled moon.

use ds::prelude::*;
use ds_style::icon::shape::Shape;

#[test]
fn the_filled_moon_is_the_outline_moon_filled() {
    let [Shape::Path(outline)] = Icon::Moon.shapes() else {
        panic!("the moon is one path");
    };
    assert_eq!(Icon::MoonFilled.shapes(), &[Shape::Solid(outline)][..]);
    assert!(Icon::SHELL.contains(&Icon::MoonFilled) && Icon::ALL.contains(&Icon::MoonFilled));
}
