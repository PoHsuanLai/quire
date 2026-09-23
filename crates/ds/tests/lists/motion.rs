//! Lists driven by the roster and a drag driven by a `DragTracker`: the markup at each moment
//! of a row's exit, and the ghost at each moment of a drag.

use super::*;

const KEYS: [&str; 4] = ["a", "b", "c", "d"];
const PITCH: RowPitch = RowPitch(Px(79.0));

#[derive(Props, Clone, PartialEq)]
struct Moment {
    state: RosterState<&'static str>,
    list: ListPresence,
}

/// A roster drawn the way a consumer draws one: a keyed `ListRow` per entry, in its presence.
fn drawn(moment: Moment) -> Element {
    rsx! {
        AnimatedList { label: "Threads", presence: moment.list,
            for entry in moment.state.entries().iter().cloned() {
                Row { key: "{entry.key}", presence: entry.presence, emphasis: emphasis(entry.key), index: entry.index }
            }
        }
    }
}

/// Row `b` is unread, so it leaves with the heavy fold.
fn emphasis(key: &str) -> Emphasis {
    if key == "b" {
        Emphasis::Strong
    } else {
        Emphasis::Plain
    }
}

fn render_moment(state: RosterState<&'static str>, list: ListPresence) -> String {
    let mut dom = VirtualDom::new_with_props(drawn, Moment { state, list });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Every `data-presence` a rendered list's rows carry, in order.
fn presences(html: &str) -> Vec<String> {
    html.split("<li ")
        .skip(1)
        .filter_map(|li| li.split("data-presence=\"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_roster_renders_each_moment_of_an_exit() {
    let first = RosterState::first_show(&KEYS, PITCH);
    let rested = first.clone().rest();
    let (leaving, anim) = rested.clone().leave(&"b", Exit::Fold, Emphasis::Strong);
    assert_eq!(anim, Anim::FoldHeavy, "an unread fold is the heavy one");
    let healing = leaving.clone().settled(&"b");
    let healed = healing.clone().rest();
    let moments = [
        (
            "entering",
            first,
            ListPresence::Entering,
            ["entering"; 4].to_vec(),
        ),
        (
            "leaving",
            leaving,
            ListPresence::Present,
            vec!["present", "leaving", "present", "present"],
        ),
        (
            "healing",
            healing,
            ListPresence::Present,
            vec!["present", "healing", "healing"],
        ),
        ("healed", healed, ListPresence::Present, vec!["present"; 3]),
    ];
    let mut failures = Vec::new();
    for (name, state, list, want) in moments {
        let html = render_moment(state, list);
        if presences(&html) != want {
            failures.push(format!("{name}: {:?}, not {want:?}", presences(&html)));
        }
        if let Err(why) = golden::check(&format!("lists/roster/{name}.html"), &html) {
            failures.push(why);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_healing_row_starts_one_pitch_down_by_heal_step() {
    let (leaving, _) =
        RosterState::first_show(&KEYS, PITCH)
            .rest()
            .leave(&"a", Exit::Curl, Emphasis::Plain);
    let html = render_moment(leaving.settled(&"a"), ListPresence::Present);
    for d in 0..3 {
        let want = format!("--dy:79px;--d:{d}");
        assert!(html.contains(&want), "no {want} in {html}");
    }
    assert!(!html.contains("data-exit"), "nothing is leaving any more");
}

#[test]
fn entering_rows_stagger_up_to_the_cap() {
    let keys: Vec<&'static str> = (0..16)
        .map(|n| &*Box::leak(format!("k{n}").into_boxed_str()))
        .collect();
    let html = render_moment(
        RosterState::first_show(&keys, PITCH),
        ListPresence::Entering,
    );
    let indices: Vec<u8> = html
        .split("style=\"--i:")
        .skip(1)
        .filter_map(|rest| rest.split(['"', ';']).next())
        .filter_map(|n| n.parse().ok())
        .collect();
    let want: Vec<u8> = (0..16u8).map(|n| n.min(12)).collect();
    assert_eq!(indices, want);
}

#[test]
fn a_live_roster_moves_its_rows_through_the_exit() {
    live::exit_plays_through();
}

#[derive(Props, Clone, PartialEq)]
struct NoProps {}

/// A drag of thread 7 over one registered place, drawn as the consumer draws it: the ghost only
/// while the drag is live, at the pointer.
fn dragging(_: NoProps) -> Element {
    let drag: DragTracker<u32> = use_drag(Px(8.0));
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
fn the_ghost_appears_past_eight_pixels_and_drops_on_its_target() {
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
        drag.moved(at(304.0, 303.0));
    });
    assert!(
        !armed.contains("ds-drag-ghost"),
        "7 px is not a drag: {armed}"
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
