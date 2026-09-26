//! EditSurface: a focusable block that hosts an app's own rendered text and hands the app its
//! input (FINDINGS "Edit surface"). mailo's composer renders its paragraphs and objects as the
//! children, marked with `data-edit-node`; the surface owns the focus and the IME, turns keys,
//! IME composition and clipboard gestures into [`EditInput`]s, resolves pointer presses to
//! [`TextPosition`](crate::TextPosition)s through the host, and never draws a caret or a
//! selection: the app does, from its [`EditHandle`]'s rects.

use crate::components::edit_surface_ctx::SurfaceCtx;
use crate::components::edit_surface_focus::{
    blur_surface, focus_surface, focused_in, focused_out, listen_soon,
};
use crate::components::edit_surface_keys::key_down;
use crate::components::edit_surface_pointer::{captured, moved, point_of_mouse, press, released};
use crate::components::edit_surface_spell::{SpellCtx, respell, touch};
use crate::components::edit_surface_spell_menu::{
    Asked, Opened, SpellLayer, SpellLink, at_caret, at_pointer,
};
use crate::components::edit_surface_state::{SurfaceState, write_soon};
use crate::components::pass_through::{DataAttr, ExtraClass, attributes, class_list};
use crate::edit::composition::on_ime;
use crate::edit::handle::{EditHandle, SurfaceHooks};
use crate::edit::host::{HostEdit, ImeEvent};
use crate::edit::input::EditInput;
use crate::edit::pointer::{CapturedPointer, EditFocus, EditPointer};
use crate::edit::position::TextPosition;
use crate::geometry::Rect;
use crate::spell::lang::Spell;
use crate::spell::marks::SpellReplace;
use dioxus::prelude::*;
use std::rc::Rc;

/// An editing surface over the app's own content.
///
/// - `on_input`: text, keys, IME composition steps, paste, cut and copy, in order.
/// - `on_pointer`: presses, drags and releases of the primary button, with the text position
///   under the pointer; a drag keeps reporting outside the surface until the release.
/// - `on_focus`: the surface took or lost the keyboard (a press, Tab, or the handle's
///   `focus`/`blur`).
/// - `handle`: the app's handle for caret and selection rects and focus (`use_edit_handle`).
/// - `ime_area`: where the IME's candidate window should sit (the caret's rect); applied while
///   the surface has the keyboard, and again each time it takes it.
/// - `extra_class`, `data`: the app's own class and `data-*` on the surface's element, so the
///   surface can be the app's styled body itself (`ExtraClass` and `DataAttr` refuse `ds-`).
/// - `spell`: [`Spell::On`] checks the spelling through the host's `HostSpell` and marks each
///   misspelt word with a dotted underline; `Off` (the default) changes nothing. `caret` is the
///   app's caret, so the word being typed stays unmarked until the caret leaves it and the
///   context-menu key knows which word it is on; `on_replace` hears a suggestion picked from the
///   spelling menu, to apply as one undoable edit (design/04-COMPONENTS.md section 50).
#[component]
pub fn EditSurface(
    on_input: EventHandler<EditInput>,
    #[props(default)] on_pointer: Option<EventHandler<EditPointer>>,
    #[props(default)] on_focus: Option<EventHandler<EditFocus>>,
    #[props(default)] handle: Option<EditHandle>,
    #[props(default)] ime_area: Option<Rect>,
    #[props(into, default)] label: Option<String>,
    #[props(into, default)] id: Option<String>,
    #[props(default)] extra_class: Option<ExtraClass>,
    #[props(default)] data: Vec<DataAttr>,
    #[props(default)] spell: Spell,
    #[props(default)] caret: Option<TextPosition>,
    #[props(default)] on_replace: Option<EventHandler<SpellReplace>>,
    children: Element,
) -> Element {
    let host = use_hook(try_consume_context::<HostEdit>);
    let state = use_hook(|| Rc::new(SurfaceState::default()));
    let checker = SpellCtx::use_new(host);
    let menu = use_signal(|| None::<Opened>);
    let on_input = {
        let checker = checker.clone();
        use_callback(move |input: EditInput| {
            on_input.call(input);
            touch(&checker);
        })
    };
    let ctx = SurfaceCtx {
        state: Rc::clone(&state),
        host,
        on_input,
        on_pointer,
        on_focus,
    };
    let heard = {
        let ctx = ctx.clone();
        use_callback(move |event: ImeEvent| {
            let (next, inputs) = on_ime(ctx.state.composing.get(), event);
            ctx.state.composing.set(next);
            ctx.tell(inputs);
        })
    };
    let told = {
        let ctx = ctx.clone();
        use_callback(move |()| focused_in(&ctx))
    };
    let hooks = {
        let (focusing, blurring) = (ctx.clone(), ctx.clone());
        SurfaceHooks {
            focus: use_callback(move |()| focus_surface(&focusing, told)),
            blur: use_callback(move |()| blur_surface(&blurring)),
        }
    };
    let capture_sink = {
        let ctx = ctx.clone();
        use_callback(move |pointer: CapturedPointer| captured(&ctx, pointer))
    };
    {
        let state = Rc::clone(&state);
        use_effect(use_reactive!(|ime_area| {
            state.ime_area.set(ime_area);
            if let (Some(host), Some(element), Some(area), EditFocus::In) =
                (host, state.element(), ime_area, state.focus.get())
            {
                write_soon(host, element, move |host, element| {
                    (host.set_ime_cursor_area)(element, area)
                });
            }
        }));
    }
    {
        let checker = checker.clone();
        use_effect(use_reactive!(|spell| respell(&checker, spell)));
    }
    {
        let checker = checker.clone();
        use_effect(use_reactive!(|caret| {
            checker.state.caret.replace(caret);
            touch(&checker);
        }));
    }
    {
        let state = Rc::clone(&state);
        use_drop(move || {
            if let (Some(host), Some(listener)) = (host, state.listener.take()) {
                (host.forget)(listener);
            }
        });
    }

    let mounted = {
        let ctx = ctx.clone();
        let checker = checker.clone();
        move |event: MountedEvent| {
            let element = event.data();
            ctx.state.element.replace(Some(Rc::clone(&element)));
            checker.state.surface.replace(Some(Rc::clone(&element)));
            touch(&checker);
            if let Some(handle) = handle {
                handle.set(Rc::clone(&element), hooks);
            }
            listen_soon(&ctx, element, heard);
        }
    };
    let (keys, pressing, moving, releasing, blurred) =
        (ctx.clone(), ctx.clone(), ctx.clone(), ctx.clone(), ctx);
    let (pointed, keyed) = (checker.clone(), checker.clone());
    let layer = (spell != Spell::Off).then(|| SpellLink {
        ctx: checker,
        on_replace,
        refocus: hooks.focus,
    });
    let class = class_list("ds-edit", extra_class.as_ref());
    let data = attributes(&data);
    rsx! {
        div {
            class,
            id,
            role: "textbox",
            "aria-multiline": "true",
            "aria-label": label,
            tabindex: "0",
            onmounted: mounted,
            onkeydown: move |event: KeyboardEvent| {
                if menu_key(&event) && at_caret(&keyed, menu) == Asked::Opened {
                    event.prevent_default();
                    return;
                }
                key_down(&keys, &event)
            },
            oncontextmenu: move |event: MouseEvent| {
                if at_pointer(&pointed, menu, point_of_mouse(&event)) == Asked::Opened {
                    event.prevent_default();
                    event.stop_propagation();
                }
            },
            onpointerdown: move |event: PointerEvent| press(&pressing, &event, capture_sink, told),
            onpointermove: move |event: PointerEvent| moved(&moving, &event),
            onpointerup: move |event: PointerEvent| released(&releasing, &event),
            onclick: move |event: MouseEvent| event.prevent_default(),
            onfocus: move |_: FocusEvent| told.call(()),
            onblur: move |_: FocusEvent| focused_out(&blurred),
            // The app's own `data-*`, last: a spread follows the named attributes.
            ..data,
            {children}
            if let Some(link) = layer {
                SpellLayer { boxes: link.ctx.boxes, menu, link }
            }
        }
    }
}

/// The context-menu key, or Shift+F10, its stand-in on keyboards without one.
fn menu_key(event: &KeyboardEvent) -> bool {
    match event.key() {
        Key::ContextMenu => true,
        Key::F10 => event.modifiers() == Modifiers::SHIFT,
        _ => false,
    }
}
