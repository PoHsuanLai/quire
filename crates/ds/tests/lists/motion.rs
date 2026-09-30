//! Lists driven by the roster and a drag driven by a `DragTracker`: the markup at each moment
//! of a row's exit, and the ghost at each moment of a drag.

use super::*;
use ds::prelude::*;
use ds_motion::drag::DRAG_THRESHOLD;

/// The list's markup at each moment of a key's arrival and exit, and each moment's presences.
#[test]
fn a_list_renders_each_moment_of_an_exit() {
    let moments = live::moments();
    let mut failures = Vec::new();
    for (name, html) in &moments {
        if let Err(why) = golden::check(&format!("lists/roster/{name}.html"), html) {
            failures.push(why);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The list stylesheet and the roster agree on the exit: the rule a leaving item matches plays
/// exactly the animation the roster settles.
#[test]
fn the_list_stylesheet_plays_what_the_roster_settles() {
    let css = include_str!("../../src/components/lists/list/list.css");
    let recipe = Anim::RowOut.recipe();
    let want = format!(
        "animation:{} {} {} forwards;",
        recipe.keyframes,
        recipe.duration.var().reference(),
        recipe.easing.var().reference()
    );
    let selector = format!(
        ".ds-list-item[*|data-presence=leaving][*|data-exit={}]{{",
        Exit::Row.slug()
    );
    let rule = css
        .lines()
        .find(|line| line.starts_with(&selector))
        .map(|line| line[selector.len()..].trim().to_owned());
    assert!(
        rule.as_deref().is_some_and(|rule| rule.starts_with(&want)),
        "{rule:?}, roster settles {want}"
    );
}

#[test]
fn a_live_list_moves_its_items_through_the_exit() {
    live::moments();
}

#[derive(Props, Clone, PartialEq)]
struct NoProps {}

/// A drag of thread 7 over one registered place, drawn as the consumer draws it: the ghost only
/// while the drag is live, at the pointer.
fn dragging(_: NoProps) -> Element {
    let drag: DragTracker<u32> = use_drag(DRAG_THRESHOLD);
    use_context_provider(|| drag);
    rsx! {
        if let DragPhase::Live { at, target, .. } = drag.phase() {
            DragGhost { title: "Notes from the sync review", sub: "Sam Lindqvist", at }
            if let Some(target) = target {
                p { "over {target}" }
            }
        }
    }
}

#[test]
fn the_ghost_appears_past_three_pixels_and_drops_on_its_target() {
    let mut dom = VirtualDom::new_with_props(dragging, NoProps {});
    dom.rebuild_in_place();
    let step = |dom: &mut VirtualDom, act: &dyn Fn(DragTracker<u32>)| {
        dom.in_scope(ScopeId::APP, || act(consume_context::<DragTracker<u32>>()));
        dom.render_immediate_to_vec();
        dioxus_ssr::render(dom)
    };
    let place = Rect {
        origin: Point {
            x: Px(0.0),
            y: Px(100.0),
        },
        size: Size {
            width: Px(200.0),
            height: Px(30.0),
        },
    };
    let at = |x: f32, y: f32| Point { x: Px(x), y: Px(y) };
    let armed = step(&mut dom, &|drag| {
        drag.set_targets(vec![place]);
        drag.down(7, at(300.0, 300.0));
        drag.moved(at(301.0, 302.0));
    });
    assert!(
        !armed.contains("ds-drag-ghost"),
        "2 px is not a drag: {armed}"
    );
    let live = step(&mut dom, &|drag| drag.moved(at(305.0, 303.0)));
    let over = step(&mut dom, &|drag| drag.moved(at(100.0, 110.0)));
    let failures: Vec<String> = [("live", &live), ("over-target", &over)]
        .into_iter()
        .filter_map(|(name, html)| golden::check(&format!("lists/drag/{name}.html"), html).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(live.contains("left:265px;top:285px"), "{live}");
    let dropped = dom.in_scope(ScopeId::APP, || consume_context::<DragTracker<u32>>().up());
    assert_eq!(dropped, Some((7, 0)), "dropped thread 7 on place 0");
}
