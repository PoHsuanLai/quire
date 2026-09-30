//! The HIG guardrails on rendered markup (design/27-HIG-PARITY.md section 7, H0), both
//! [`super::Severity::Warning`] today: [`Rule::UnnamedControl`] (every interactive element has
//! an accessible name, section 3.1) and [`Rule::ThreeDots`] ("…", never "...", in text and
//! labels, design/02-TYPE.md section 13).
//!
//! One pass over the markup keeps a stack of open elements: text inside a control names it,
//! as does a descendant's `aria-label` or `alt`; an `aria-hidden="true"` subtree names nothing
//! and is not checked. A control is judged when it closes.

use super::markup::{attr_value, element_selector, line_col_at, tag_end, tag_name};
use super::rule::{Offence, Rule};

/// Elements that never have content.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Elements whose content is not text a person reads.
const RAW_TEXT: &[&str] = &["style", "script"];

/// Roles that make an element take input.
const INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "link",
    "checkbox",
    "radio",
    "switch",
    "tab",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "slider",
    "spinbutton",
    "combobox",
    "textbox",
    "searchbox",
    "treeitem",
];

/// Attributes that name an element outright.
const NAMING_ATTRS: &[&str] = &["aria-label", "aria-labelledby", "title", "alt"];

/// Attributes whose text a person reads.
const LABEL_ATTRS: &[&str] = &["aria-label", "title", "alt", "placeholder"];

/// Every guardrail offence in `html`.
pub(super) fn offences(html: &str) -> Vec<Offence> {
    let labelled = label_targets(html);
    let mut scan = Scan {
        html,
        labelled,
        open: Vec::new(),
        out: Vec::new(),
    };
    scan.run();
    scan.out
}

/// Whether an element's subtree is read by assistive technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Audible {
    /// It is.
    Heard,
    /// `aria-hidden="true"` on it or an ancestor.
    Hidden,
}

/// Whether an interactive element has found its name yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Naming {
    /// Not an interactive element.
    NotAControl,
    /// A control with no name so far.
    Unnamed,
    /// A control with a name.
    Named,
}

/// One open element.
struct Open {
    name: String,
    selector: String,
    line: u32,
    column: u32,
    heard: Audible,
    named: Naming,
}

struct Scan<'a> {
    html: &'a str,
    labelled: Vec<String>,
    open: Vec<Open>,
    out: Vec<Offence>,
}

impl Scan<'_> {
    fn run(&mut self) {
        let mut cursor = 0usize;
        while cursor < self.html.len() {
            let Some(relative) = self.html[cursor..].find('<') else {
                self.text(cursor, self.html.len());
                break;
            };
            let start = cursor + relative;
            self.text(cursor, start);
            cursor = self.markup_at(start);
        }
        while let Some(element) = self.open.pop() {
            self.judge(element);
        }
    }

    /// Handles the markup starting at `start`; returns where scanning resumes.
    fn markup_at(&mut self, start: usize) -> usize {
        let rest = &self.html[start..];
        if rest.starts_with("<!--") {
            return rest
                .find("-->")
                .map_or(self.html.len(), |end| start + end + 3);
        }
        let Some(end) = tag_end(self.html, start) else {
            return self.html.len();
        };
        let tag = &self.html[start..end];
        if let Some(closing) = tag.strip_prefix("</") {
            self.close(closing.trim_end_matches('>').trim());
            return end;
        }
        if tag.starts_with("<!") {
            return end;
        }
        let name = tag_name(tag).to_ascii_lowercase();
        self.open_tag(tag, start);
        if RAW_TEXT.contains(&name.as_str()) {
            let close = format!("</{name}");
            return self.html[end..]
                .to_ascii_lowercase()
                .find(&close)
                .map_or(self.html.len(), |at| end + at);
        }
        end
    }

    fn open_tag(&mut self, tag: &str, start: usize) {
        let (line, column) = line_col_at(self.html, start);
        let name = tag_name(tag).to_ascii_lowercase();
        let selector = element_selector(tag);
        let hidden = attr_value(tag, "aria-hidden").is_some_and(|(value, _)| value == "true");
        let heard = match (self.heard(), hidden) {
            (Audible::Heard, false) => Audible::Heard,
            _ => Audible::Hidden,
        };
        if heard == Audible::Heard {
            self.three_dots_in_attrs(tag, &selector, line, column);
            if names_by_attr(tag) {
                self.name_open();
            }
        }
        let named = match (heard, is_control(&name, tag)) {
            (Audible::Heard, true) if names_by_attr(tag) || self.is_labelled(tag) => Naming::Named,
            (Audible::Heard, true) => Naming::Unnamed,
            _ => Naming::NotAControl,
        };
        let element = Open {
            name,
            selector,
            line,
            column,
            heard,
            named,
        };
        if VOID.contains(&element.name.as_str()) || tag.trim_end_matches('>').ends_with('/') {
            self.judge(element);
        } else {
            self.open.push(element);
        }
    }

    fn close(&mut self, name: &str) {
        let name = name.to_ascii_lowercase();
        let Some(at) = self.open.iter().rposition(|element| element.name == name) else {
            return;
        };
        let closed: Vec<Open> = self.open.drain(at..).collect();
        for element in closed.into_iter().rev() {
            self.judge(element);
        }
    }

    fn judge(&mut self, element: Open) {
        if element.named == Naming::Unnamed {
            self.out.push(Offence {
                rule: Rule::UnnamedControl,
                selector: element.selector.clone(),
                line: element.line,
                column: element.column,
                text: format!(
                    "<{}>: no accessible name (text, aria-label, aria-labelledby, title or alt)",
                    element.name
                ),
            });
        }
    }

    fn text(&mut self, from: usize, to: usize) {
        let text = &self.html[from..to];
        if text.trim().is_empty() || self.heard() == Audible::Hidden {
            return;
        }
        self.name_open();
        if let Some(at) = text.find("...") {
            let (line, column) = line_col_at(self.html, from + at);
            let selector = self
                .open
                .last()
                .map(|element| element.selector.clone())
                .unwrap_or_default();
            self.out
                .push(three_dots(selector, line, column, text.trim()));
        }
    }

    fn three_dots_in_attrs(&mut self, tag: &str, selector: &str, line: u32, column: u32) {
        for attr in LABEL_ATTRS {
            if let Some((value, _)) = attr_value(tag, attr)
                && value.contains("...")
            {
                let excerpt = format!("{attr}=\"{value}\"");
                self.out
                    .push(three_dots(selector.to_owned(), line, column, &excerpt));
            }
        }
    }

    /// Every open control is named: something inside it says what it is.
    fn name_open(&mut self) {
        for element in &mut self.open {
            if element.named == Naming::Unnamed {
                element.named = Naming::Named;
            }
        }
    }

    /// Whether the innermost open element is heard.
    fn heard(&self) -> Audible {
        self.open
            .last()
            .map_or(Audible::Heard, |element| element.heard)
    }

    /// Whether a `<label for>` names `tag` by its `id`.
    fn is_labelled(&self, tag: &str) -> bool {
        attr_value(tag, "id").is_some_and(|(id, _)| self.labelled.iter().any(|target| target == id))
    }
}

fn three_dots(selector: String, line: u32, column: u32, excerpt: &str) -> Offence {
    let short: String = excerpt.chars().take(60).collect();
    Offence {
        rule: Rule::ThreeDots,
        selector,
        line,
        column,
        text: format!("\"...\" where \"…\" belongs: {short}"),
    }
}

/// Whether `tag` (named `name`) takes input: a button, a link with an `href`, a form control
/// that is not hidden, or an element with an interactive `role`.
fn is_control(name: &str, tag: &str) -> bool {
    let by_role = attr_value(tag, "role").is_some_and(|(role, _)| {
        INTERACTIVE_ROLES
            .iter()
            .any(|interactive| role.eq_ignore_ascii_case(interactive))
    });
    let by_element = match name {
        "button" | "select" | "textarea" => true,
        "a" => attr_value(tag, "href").is_some(),
        "input" => attr_value(tag, "type").is_none_or(|(kind, _)| kind != "hidden"),
        _ => false,
    };
    by_role || by_element
}

/// Whether `tag` carries a non-empty naming attribute (an `input`'s `placeholder` too: the
/// accessible-name algorithm's last resort).
fn names_by_attr(tag: &str) -> bool {
    let named =
        |attr: &str| attr_value(tag, attr).is_some_and(|(value, _)| !value.trim().is_empty());
    NAMING_ATTRS.iter().any(|attr| named(attr))
        || (tag_name(tag).eq_ignore_ascii_case("input") && named("placeholder"))
}

/// Every `id` a `<label for>` in `html` points at.
fn label_targets(html: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = html[cursor..].find("<label") {
        let start = cursor + relative;
        let Some(end) = tag_end(html, start) else {
            break;
        };
        if let Some((target, _)) = attr_value(&html[start..end], "for") {
            targets.push(target.to_owned());
        }
        cursor = end;
    }
    targets
}
