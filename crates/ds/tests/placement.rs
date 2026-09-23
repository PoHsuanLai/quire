//! `place()`: flip when the wanted side overflows, clamp 8 px inside the bounds, hover cards
//! never flip (design/01-LAYOUT.md section 8.2, design/06-INTERACTIONS.md sections 3 and 4).

use ds::{Align, Placed, Placement, Point, Px, Rect, Side, Size, place};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect {
        origin: Point { x: Px(x), y: Px(y) },
        size: Size {
            width: Px(w),
            height: Px(h),
        },
    }
}

fn size(w: f32, h: f32) -> Size {
    Size {
        width: Px(w),
        height: Px(h),
    }
}

fn at(x: f32, y: f32, side: Side) -> Placed {
    Placed {
        origin: Point { x: Px(x), y: Px(y) },
        side,
    }
}

/// A 1000 x 700 window at the origin.
const WINDOW: Rect = Rect {
    origin: Point {
        x: Px(0.0),
        y: Px(0.0),
    },
    size: Size {
        width: Px(1000.0),
        height: Px(700.0),
    },
};

const MENU: Placement = Placement::new(Side::Bottom, Align::Start);
const CARD: Placement = Placement::new(Side::Bottom, Align::Start).no_flip();
const SIDE_CARD: Placement = Placement::new(Side::Right, Align::Start).no_flip();

struct Case {
    name: &'static str,
    anchor: Rect,
    content: Size,
    bounds: Rect,
    want: Placement,
    gap: f32,
    expect: Placed,
}

#[test]
fn placement_flips_and_clamps() {
    let cases = [
        Case {
            name: "menu below its trigger, left edges aligned, 6 px gap",
            anchor: rect(100.0, 40.0, 60.0, 24.0),
            content: size(280.0, 200.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(100.0, 70.0, Side::Bottom),
        },
        Case {
            name: "menu that would cross the bottom margin flips above (top - height - 6)",
            anchor: rect(100.0, 600.0, 60.0, 24.0),
            content: size(280.0, 200.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(100.0, 394.0, Side::Top),
        },
        Case {
            name: "exactly touching the bottom margin does not flip",
            anchor: rect(100.0, 466.0, 60.0, 20.0),
            content: size(280.0, 200.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(100.0, 492.0, Side::Bottom),
        },
        Case {
            name: "menu at the right edge clamps to width - w - 8",
            anchor: rect(900.0, 40.0, 60.0, 24.0),
            content: size(280.0, 200.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(712.0, 70.0, Side::Bottom),
        },
        Case {
            name: "menu at the left edge clamps to 8",
            anchor: rect(2.0, 40.0, 60.0, 24.0),
            content: size(280.0, 200.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(8.0, 70.0, Side::Bottom),
        },
        Case {
            name: "a context menu from a point (zero-size anchor)",
            anchor: rect(500.0, 300.0, 0.0, 0.0),
            content: size(220.0, 150.0),
            bounds: WINDOW,
            want: MENU,
            gap: 0.0,
            expect: at(500.0, 300.0, Side::Bottom),
        },
        Case {
            name: "a context menu near the bottom-right corner flips up and clamps left",
            anchor: rect(950.0, 650.0, 0.0, 0.0),
            content: size(220.0, 150.0),
            bounds: WINDOW,
            want: MENU,
            gap: 0.0,
            expect: at(772.0, 500.0, Side::Top),
        },
        Case {
            name: "too tall for either side: stays on the roomier side, then clamps to 8",
            anchor: rect(100.0, 100.0, 60.0, 24.0),
            content: size(280.0, 690.0),
            bounds: WINDOW,
            want: MENU,
            gap: 6.0,
            expect: at(100.0, 8.0, Side::Bottom),
        },
        Case {
            name: "sender card below the name, never flips: slides up to the margin",
            anchor: rect(300.0, 640.0, 90.0, 18.0),
            content: size(300.0, 160.0),
            bounds: WINDOW,
            want: CARD,
            gap: 6.0,
            expect: at(300.0, 532.0, Side::Bottom),
        },
        Case {
            name: "side card right of a sidebar entry, never flips: slides left",
            anchor: rect(800.0, 100.0, 180.0, 30.0),
            content: size(260.0, 120.0),
            bounds: WINDOW,
            want: SIDE_CARD,
            gap: 10.0,
            expect: at(732.0, 100.0, Side::Right),
        },
        Case {
            name: "right placement flips left when it overflows and flipping is allowed",
            anchor: rect(800.0, 100.0, 180.0, 30.0),
            content: size(260.0, 120.0),
            bounds: WINDOW,
            want: Placement::new(Side::Right, Align::Start),
            gap: 10.0,
            expect: at(530.0, 100.0, Side::Left),
        },
        Case {
            name: "centred above",
            anchor: rect(400.0, 300.0, 100.0, 20.0),
            content: size(60.0, 30.0),
            bounds: WINDOW,
            want: Placement::new(Side::Top, Align::Center),
            gap: 8.0,
            expect: at(420.0, 262.0, Side::Top),
        },
        Case {
            name: "end-aligned below",
            anchor: rect(400.0, 300.0, 100.0, 20.0),
            content: size(60.0, 30.0),
            bounds: WINDOW,
            want: Placement::new(Side::Bottom, Align::End),
            gap: 4.0,
            expect: at(440.0, 324.0, Side::Bottom),
        },
        Case {
            name: "origin is relative to the bounds' top-left",
            anchor: rect(150.0, 140.0, 60.0, 24.0),
            content: size(100.0, 50.0),
            bounds: rect(100.0, 100.0, 400.0, 300.0),
            want: MENU,
            gap: 6.0,
            expect: at(50.0, 70.0, Side::Bottom),
        },
    ];
    for case in cases {
        let got = place(
            case.anchor,
            case.content,
            case.bounds,
            case.want,
            Px(case.gap),
        );
        assert_eq!(got, case.expect, "{}", case.name);
    }
}
