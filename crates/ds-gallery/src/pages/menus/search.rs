//! SearchField with its suggestions panel, and the rows a result list draws: marks over the
//! characters a query matched, an avatar, a second line.

use crate::axes::{Axes, Showcase};
use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::content::avatar::{
    AvatarFace, AvatarShape, AvatarSize, AvatarTone, person_hue,
};
use ds::host::measure::Anchor;
use ds::prelude::*;

const PEOPLE: [(&str, &str); 3] = [
    ("Dana Okafor", "dana@example.com"),
    ("Sam Lindqvist", "sam@example.com"),
    ("Priya Raman", "priya@example.com"),
];

fn avatar(name: &str) -> MenuImage {
    MenuImage::Avatar(AvatarFace {
        initial: name.chars().next().unwrap_or('?'),
        size: AvatarSize::Size22,
        tone: AvatarTone::Person(person_hue(name)),
        shape: AvatarShape::Round,
    })
}

/// The suggestions for `query`: a top hit with a second line, the people whose names match.
fn sections(query: &str) -> Vec<SuggestionSection<String>> {
    let title = "Invoice from Dana";
    let hit = MenuItem::new("hit".to_owned(), title)
        .with_image(MenuImage::Icon(Icon::Search))
        .with_marks(Marks::of_query(title, query))
        .with_subtitle("Re: March invoice");
    let people = PEOPLE
        .iter()
        .filter(|(name, _)| query.is_empty() || !Marks::of_query(name, query).is_empty())
        .map(|&(name, address)| {
            MenuItem::new(address.to_owned(), name)
                .with_image(avatar(name))
                .with_marks(Marks::of_query(name, query))
                .with_subtitle(address)
        })
        .collect();
    vec![
        SuggestionSection::titled("Top hit", vec![hit]),
        SuggestionSection::titled("People", people),
    ]
}

/// A search field whose suggestions panel it owns, and the rows drawn inline.
#[component]
pub fn SearchSuggestions() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut query = use_signal(|| "dana".to_owned());
    let mut picked = use_signal(|| "nothing yet".to_owned());
    rsx! {
        Section {
            title: "SearchField",
            note: "NSSearchField with its suggestions menu: the panel is owned by the field and hangs under it at its width, flipping above when there is no room. The keyboard never leaves the field: Up and Down move the highlight, Enter picks it (or submits the text when none is highlighted), Escape closes the panel first and clears the field second, Tab leaves. A press on the field is inside the panel; a press elsewhere closes it. Rows can mark the characters the query matched, lead with an avatar and carry a second line.",
            div { class: "g-row", style: "align-items:flex-start",
                div { style: "width:300px",
                    SearchField::<String> {
                        label: "Search",
                        value: query(),
                        placeholder: "Search mail, people",
                        suggestions: sections(&query()),
                        oninput: move |next: String| query.set(next),
                        onpick: move |value: String| picked.set(format!("picked {value}")),
                        onsubmit: move |text: String| picked.set(format!("searched for {text}")),
                    }
                }
            }
            // Room under the field for the panel, inside the page.
            div { style: "height:300px" }
            p { class: "g-note", "Last action: {picked}" }
        }
        if showcase == Showcase::Posed {
            Section {
                title: "Result rows",
                note: "A row's title with the matched characters in semibold, an avatar in the image column and a second line in the faint ink; a plain row beside them is unchanged.",
                div { class: "g-stage-row",
                    div { class: "g-stage", "data-wide": "true",
                        div { class: "g-menu-card",
                            Menu::<String> {
                                placement: MenuPlacement::Popup,
                                anchor: Anchor::Point(Point::default()),
                                items: result_rows(),
                                flow: Flow::Inline,
                                active: MenuCursor::Controlled(Some(1)),
                                onpick: |_| {},
                                onclose: |_| {},
                            }
                        }
                        p { class: "g-note g-stage-caption", "Marks, avatar, second line" }
                    }
                }
            }
        }
    }
}

/// The posed rows: the suggestions for "dan" and a plain row beside them.
fn result_rows() -> Vec<MenuItem<String>> {
    let mut rows: Vec<MenuItem<String>> = sections("dan")
        .into_iter()
        .flat_map(|section| section.items)
        .collect();
    rows.push(MenuItem::new("plain".to_owned(), "A plain row"));
    rows
}
