//! Spaces: a toy notes app on the Spaces kit (design/21-SPACES.md section 13). Switch Space
//! with a dot or Command and a digit; right-click the Space's name or a dot for its menu; open a
//! note to see it under Today. Each Space opens where it was left.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::app::spaces::{
    SpaceHead, SpaceMenu, Spaces, SpacesFoot, SwitchChord, Switched, TodayHandle, TodayRow,
    TodaySection, use_spaces, use_today,
};
use ds::components::content::label::LabelStyle;
use ds::prelude::*;
use ds_core::vocab::RowState;

/// What a Space of the toy app holds: the folder it is about.
#[derive(Debug, Clone, PartialEq, Default)]
struct Folder {
    name: String,
}

const NOTES: [&str; 3] = ["Shopping", "Ideas", "Trip"];

fn first_spaces() -> Spaces<Folder, String> {
    let mut spaces = Spaces::first_run(
        [
            Folder {
                name: "Work".into(),
            },
            Folder {
                name: "Home".into(),
            },
        ],
        Folder::default,
    );
    let ids: Vec<_> = spaces.list().iter().map(|space| space.id).collect();
    for (id, name) in ids.into_iter().zip(["Work", "Home"]) {
        spaces.rename(id, name);
    }
    spaces
}

/// The Spaces page.
#[component]
pub fn SpacesPage() -> Element {
    let mut open = use_signal(String::new);
    let spaces = use_spaces(
        first_spaces,
        |_| {},
        move || open.peek().clone(),
        move |arrived: Switched<String>| open.set(arrived.restore),
    );
    let today: TodayHandle<String, ds::components::app::spaces::NoPark> =
        use_today(Default::default, |_| {});
    let new_payload = use_callback(|()| Folder::default());
    let scheme = use_scope().scheme;
    let slide = spaces.slide().map(|slide| slide.attribute());
    rsx! {
        Section { title: "Notes", note: "A toy notes app. The sidebar head names the Space (right-click for its menu); the foot has a dot per Space (right-click one for its menu), + makes a Space and opens its name, and Command and 1 to 9 switch. A note you open lands in Today for 12 hours, per Space; each Space reopens the note it was left on, and the page slides in from the side the Space is on.",
            Ds {
                appearance: Appearance { theme: if scheme == Scheme::Dark { Theme::Dark } else { Theme::Light }, ..Appearance::default() },
                look: spaces.look(),
                material: Material::Window,
                stylesheet: Inject::Host,
                chrome: Some(RootChrome::Painted),
                div { class: "g-spaces",
                    tabindex: "0",
                    onkeydown: move |event: KeyboardEvent| {
                        spaces.on_key(SwitchChord::Primary, &event);
                    },
                    div { class: "g-spaces-side",
                        SpaceHead { handle: spaces }
                        for note in NOTES {
                            Row {
                                key: "{note}",
                                title: TextLine::from(note),
                                leading: RowLeading::Icon(Icon::File),
                                state: RowState {
                                    selection: if open() == note { Selection::Selected } else { Selection::Unselected },
                                    ..RowState::default()
                                },
                                onclick: move |_| {
                                    open.set(note.to_owned());
                                    today.opened(spaces.current(), note.to_owned());
                                },
                            }
                        }
                        TodaySection {
                            spaces,
                            today,
                            row: use_callback(|note: String| Some(TodayRow {
                                title: note,
                                leading: RowLeading::Icon(Icon::File),
                            })),
                            on_pick: move |note: String| open.set(note),
                            selected: Some(open()),
                        }
                        SpacesFoot { handle: spaces, new_payload }
                    }
                    div { class: "g-spaces-page", "data-slide": slide,
                        Label {
                            text: if open().is_empty() { "No note open".to_owned() } else { open() },
                            style: LabelStyle::Title,
                        }
                    }
                    SpaceMenu::<Folder, String, ()> {
                        handle: spaces,
                        new_payload,
                        kept: "Your notes".to_owned(),
                        on_deleted: move |gone| today.drop_space(gone),
                    }
                }
            }
        }
    }
}
