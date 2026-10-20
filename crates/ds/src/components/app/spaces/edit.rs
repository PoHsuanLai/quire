//! Edit Space...: the Space's name and colour in one sheet, with Cancel and Save.

use super::controller::SpacesHandle;
use crate::components::app::space_editor::DotIndex;
use crate::components::app::space_editor::colour::SpaceColour;
use crate::components::content::label::{Label, LabelStyle};
use crate::components::controls::button::Button;
use crate::components::controls::button_model::Answers;
use crate::components::fields::field_row::FieldRow;
use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::forms::form::Form;
use crate::components::forms::form_section::FormSection;
use crate::components::overlays::sheet::Sheet;
use crate::focus::request::use_focus_request;
use dioxus::prelude::*;
use ds_core::press::{PointerButton, Press};
use ds_style::scope::use_scope;
use ds_style::space::list::SpaceId;
use ds_style::space::look::SpaceLook;
use ds_style::tokens::control_size::ControlSize;

/// The sheet the Space menu's Edit Space... opens: a title, the Space's name in a field that
/// takes the keyboard (selected, so typing replaces it), its colour (`SpaceColour`: the field,
/// stops, grain and presets, in the scheme the window is drawn in), and Cancel and Save.
///
/// Each edit shows at once, as the menu's parts did: the name in the sidebar head, the colour in
/// the frame. Save, and Return anywhere in the sheet, keep them; Cancel and Escape put back the
/// name and look the sheet opened on. Either way it closes through
/// [`SpacesHandle::close_menu`], which writes the list once. `SpaceMenu` draws it while
/// `Showing::Edit` is open (its Edit Space... row, or a host's own
/// `handle.show(space, at, Showing::Edit)`, as a settings pane's Edit button would); a host that
/// draws it outside `SpaceMenu` opens it with that `show` first, so the Space is held as being
/// edited (no dot click switches away), and draws it while `handle.open()` says `Edit`.
#[component]
pub fn EditSpaceSheet<P, R>(space: SpaceId, handle: SpacesHandle<P, R>) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let scheme = use_scope().scheme;
    let mut active = use_signal(DotIndex::default);
    let focus = FieldFocus::Controlled(use_focus_request().with_select_all());
    let opened_on = use_hook(|| {
        handle
            .spaces()
            .peek()
            .get(space)
            .map(|one| (one.name.clone(), one.look.clone()))
    });
    let Some((name, look)) = handle
        .spaces()
        .read()
        .get(space)
        .map(|one| (one.name.clone(), one.look.clone()))
    else {
        return rsx! {};
    };
    let cancel = EventHandler::new(move |()| {
        if let Some((name, look)) = opened_on.clone() {
            handle.change(space, move |one| {
                one.name = name;
                one.look = look;
            });
        }
        handle.close_menu();
    });
    let save = EventHandler::new(move |()| handle.close_menu());
    let primary = |then: EventHandler<()>| {
        move |press: Press| {
            if press.button == PointerButton::Primary {
                then.call(());
            }
        }
    };
    rsx! {
        Sheet {
            label: "Edit Space",
            onclose: cancel,
            on_return: save,
            div { class: "ds-space-edit",
                div { class: "ds-space-edit-title",
                    Label { text: "Edit Space", style: LabelStyle::Title }
                }
                Form {
                    FormSection {
                        FieldRow { label: "Name",
                            TextField {
                                label: "Space name",
                                value: name,
                                focus,
                                oninput: move |typed: String| handle.change(space, |one| one.name = typed),
                            }
                        }
                    }
                    FormSection {
                        div { class: "ds-space-edit-colour",
                            SpaceColour {
                                look,
                                scheme,
                                active_dot: active(),
                                onchange: move |look: SpaceLook| handle.change(space, |one| one.look = look),
                                on_active_dot: move |dot: DotIndex| active.set(dot),
                            }
                        }
                    }
                }
                div { class: "ds-space-edit-acts",
                    Button {
                        size: ControlSize::Large,
                        label: "Cancel",
                        answers: Answers::Escape,
                        onclick: primary(cancel),
                    }
                    Button {
                        size: ControlSize::Large,
                        label: "Save",
                        answers: Answers::Return,
                        onclick: primary(save),
                    }
                }
            }
        }
    }
}
