//! The role of an element: its `role` attribute, else the one its tag implies.

use blitz_dom::ElementData;

/// `element`'s role, `None` for one that is no control (a plain `div`, a `role=none` wrapper).
pub(super) fn role_of(element: &ElementData) -> Option<String> {
    let named =
        attr(element, "role").and_then(|text| text.split_whitespace().next().map(str::to_owned));
    match named.as_deref() {
        Some("none" | "presentation" | "generic") => None,
        Some(_) => named,
        None => implied(element).map(str::to_owned),
    }
}

/// The value of attribute `name`.
pub(super) fn attr<'a>(element: &'a ElementData, name: &str) -> Option<&'a str> {
    element
        .attrs()
        .iter()
        .find(|found| &*found.name.local == name)
        .map(|found| &*found.value)
}

fn implied(element: &ElementData) -> Option<&'static str> {
    match &*element.name.local {
        "button" => Some("button"),
        "a" if attr(element, "href").is_some() => Some("link"),
        "textarea" => Some("textbox"),
        "select" => Some("combobox"),
        "option" => Some("option"),
        "img" => Some("img"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "input" => Some(match attr(element, "type").unwrap_or("text") {
            "button" | "submit" | "reset" => "button",
            "checkbox" => "checkbox",
            "radio" => "radio",
            "range" => "slider",
            "hidden" => return None,
            _ => "textbox",
        }),
        _ => None,
    }
}
