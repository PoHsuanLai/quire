//! Symbol effects as markup (design/35-SYMBOL-EFFECTS.md): what the wrapper and the `svg` carry
//! for each effect, on a first render, after a change, and after a render that changes nothing.

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::prelude::*;
use ds_core::vocab::Shown;

static ICON: GlobalSignal<Icon> = Signal::global(|| Icon::Trash);
static EFFECT: GlobalSignal<SymbolEffect> = Signal::global(|| SymbolEffect::NONE);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            Symbol { icon: ICON(), size: IconSize::Base, effect: EFFECT() }
        }
    }
}

/// A page that has rendered `first`, and what it renders after each of `then`, each a re-render.
fn renders(icon: Icon, first: SymbolEffect, then: &[SymbolEffect]) -> Vec<String> {
    let mut dom = VirtualDom::new(Page);
    dom.in_runtime(|| {
        *ICON.write() = icon;
        *EFFECT.write() = first;
    });
    dom.rebuild_in_place();
    let mut all = vec![symbol(&dom)];
    for &effect in then {
        dom.in_runtime(|| *EFFECT.write() = effect);
        dom.render_immediate(&mut NoOpMutations);
        all.push(symbol(&dom));
    }
    all
}

/// The symbol's markup: from its wrapper to the end of the page.
fn symbol(dom: &VirtualDom) -> String {
    let page = dioxus_ssr::render(dom);
    let from = page.find("<span class=\"ds-symbol").expect("a symbol");
    page[from..].to_owned()
}

fn once(effect: OnceEffect, trigger: u32) -> SymbolEffect {
    SymbolEffect::Once(effect, Trigger(trigger))
}

fn looped(effect: LoopEffect, activity: Activity) -> SymbolEffect {
    SymbolEffect::While(effect, activity)
}

#[test]
fn a_once_effect_is_still_on_its_first_render_and_plays_when_its_trigger_changes() {
    // (effect, the class it plays, the icon): the part effects fall back on an icon with no part.
    const CASES: &[(OnceEffect, &str, Icon)] = &[
        (OnceEffect::Bounce, "a-bounce", Icon::Inbox),
        (OnceEffect::Pulse, "a-pulse", Icon::Inbox),
        (OnceEffect::Wiggle, "a-wiggle", Icon::Inbox),
        (OnceEffect::Breathe, "a-breathe", Icon::Inbox),
        (OnceEffect::Rotate, "a-rotate-once", Icon::Refresh),
        (OnceEffect::VariableColor, "a-pulse", Icon::Inbox),
        (OnceEffect::Part, "a-bounce", Icon::Inbox),
    ];
    for &(effect, class, icon) in CASES {
        let got = renders(
            icon,
            once(effect, 0),
            &[
                once(effect, 0),
                once(effect, 1),
                once(effect, 1),
                once(effect, 2),
            ],
        );
        let slug = effect.slug();
        assert!(
            got[0].contains(&format!("data-symbol=\"{slug}\"")),
            "{got:?}"
        );
        assert!(
            got[0].contains("data-run=\"once\""),
            "{effect:?}: {}",
            got[0]
        );
        // Nothing plays until the trigger changes, however many times it re-renders.
        assert!(
            !got[0].contains("ds-symbol a-") && !got[0].contains("data-pulse"),
            "{effect:?}: {}",
            got[0]
        );
        assert_eq!(got[1], got[0], "{effect:?}: a re-render changes nothing");
        assert!(
            got[2].contains(&format!("class=\"ds-symbol {class}\"")),
            "{effect:?}: {}",
            got[2]
        );
        assert!(
            got[2].contains("data-pulse=\"a\""),
            "{effect:?}: {}",
            got[2]
        );
        assert_eq!(
            got[3], got[2],
            "{effect:?}: the same trigger does not restart it"
        );
        assert!(
            got[4].contains("data-pulse=\"b\""),
            "{effect:?}: the next fire restarts it"
        );
    }
}

#[test]
fn a_while_effect_plays_while_active_and_holds_once_it_stops() {
    const CASES: &[(LoopEffect, &str, Icon)] = &[
        (LoopEffect::Bounce, "a-bounce-loop", Icon::Inbox),
        (LoopEffect::Pulse, "a-pulse-loop", Icon::Inbox),
        (LoopEffect::ScaleUp, "a-scale-up", Icon::Inbox),
        (LoopEffect::ScaleDown, "a-scale-down", Icon::Inbox),
        (LoopEffect::Wiggle, "a-wiggle-loop", Icon::Inbox),
        (LoopEffect::Breathe, "a-breathe-loop", Icon::Inbox),
        (LoopEffect::Rotate, "a-turn", Icon::Refresh),
        (LoopEffect::VariableColor, "a-pulse-loop", Icon::Inbox),
        (LoopEffect::Part, "a-bounce-loop", Icon::Inbox),
    ];
    for &(effect, class, icon) in CASES {
        let got = renders(
            icon,
            looped(effect, Activity::Idle),
            &[
                looped(effect, Activity::Active),
                looped(effect, Activity::Idle),
            ],
        );
        assert!(
            got[0].contains("data-run=\"while\"") && !got[0].contains("data-pulse"),
            "{effect:?}: {}",
            got[0]
        );
        assert!(
            got[1].contains(&format!("class=\"ds-symbol {class}\"")),
            "{effect:?}: {}",
            got[1]
        );
        assert!(
            got[2].contains("class=\"ds-symbol a-hold\""),
            "{effect:?}: {}",
            got[2]
        );
    }
}

#[test]
fn appear_and_disappear_stand_hidden_then_play_on_the_change() {
    use Shown::{Hidden, Visible};
    let appear = |shown| SymbolEffect::Transition(TransitionEffect::Appear, shown);
    let got = renders(Icon::Inbox, appear(Hidden), &[appear(Visible)]);
    assert!(
        got[0].contains("data-visibility=\"hidden\"") && !got[0].contains("a-appear"),
        "{}",
        got[0]
    );
    assert!(
        got[1].contains("a-appear") && !got[1].contains("data-visibility"),
        "{}",
        got[1]
    );
    let disappear = |shown| SymbolEffect::Transition(TransitionEffect::Disappear, shown);
    let got = renders(
        Icon::Inbox,
        disappear(Visible),
        &[disappear(Hidden), disappear(Visible)],
    );
    assert!(
        !got[0].contains("a-disappear") && !got[0].contains("data-visibility"),
        "{}",
        got[0]
    );
    assert!(got[1].contains("a-disappear"), "{}", got[1]);
    assert!(got[2].contains("a-hold"), "{}", got[2]);
}

#[test]
fn replace_is_the_glyph_morph_and_draw_on_writes_its_dash_on_the_svg() {
    let replace = SymbolEffect::Transition(TransitionEffect::Replace, Shown::Visible);
    let got = renders(Icon::Volume, replace, &[]);
    assert!(
        got[0].contains("ds-morph-glyph") && got[0].contains("data-symbol=\"replace\""),
        "{}",
        got[0]
    );
    let draw = |shown| SymbolEffect::Transition(TransitionEffect::DrawOn, shown);
    let got = renders(Icon::Trash, draw(Shown::Visible), &[]);
    assert!(
        got[0].contains("data-symbol=\"draw-on\"") && !got[0].contains("dasharray"),
        "{}",
        got[0]
    );
    let got = renders(Icon::Trash, draw(Shown::Hidden), &[]);
    assert!(
        got[0].contains("opacity=\"0\""),
        "nothing is drawn: {}",
        got[0]
    );
}

#[test]
fn a_part_effect_puts_the_parts_in_groups_inside_the_svg_and_the_rest_outside() {
    for (icon, groups) in [
        (Icon::Trash, 1),
        (Icon::Wifi, 4),
        (Icon::Volume2, 2),
        (Icon::BatteryFull, 3),
    ] {
        let effect = if groups > 1 {
            once(OnceEffect::VariableColor, 0)
        } else {
            once(OnceEffect::Part, 0)
        };
        let got = renders(icon, effect, &[]);
        assert_eq!(got[0].matches("<g").count(), groups, "{icon:?}: {}", got[0]);
        assert!(got[0].contains("class=\"ds-ic\""), "{}", got[0]);
    }
}

#[test]
fn reduced_motion_writes_the_same_markup_for_the_stylesheet_to_hold() {
    let effect = looped(LoopEffect::Wiggle, Activity::Active);
    let standard = renders(Icon::Bell, effect, &[]);
    let mut dom = VirtualDom::new(Page);
    dom.in_runtime(|| {
        *ICON.write() = Icon::Bell;
        *EFFECT.write() = effect;
        *MOTION.write() = Motion::Reduced;
    });
    dom.rebuild_in_place();
    assert_eq!(symbol(&dom), standard[0]);
}
