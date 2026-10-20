//! The closed page list Settings had before pages were declared in schemas, kept as a typed
//! layer over [`PageId`] (design/22-SETTINGS.md section 9.3).
//!
//! New code names a page by [`PageId`]. A key still carries a [`Page`], so a schema written before
//! pages were declared reads unchanged, and a page nobody here has heard of (a newer program's)
//! reads as `Page::App(<its name>)` instead of failing the schema.

use serde::{Deserialize, Serialize};

use super::page_id::PageId;

/// The Settings app's old fixed page list, plus one page per other id: a third-party program's
/// page, or a page a schema declares (`Page::declared("wallpaper")`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "v",
    rename_all = "snake_case",
    from = "PageWire"
)]
pub enum Page {
    Appearance,
    Dock,
    MouseAndGestures,
    KeyboardAndShortcuts,
    Notifications,
    Spaces,
    Accounts,
    /// What programs may reach: the per-app permission rows the compositor publishes
    /// (design/22-SETTINGS.md section 5), as macOS's "Privacy & Security" does.
    Privacy,
    /// sill's idle service (design/22-SETTINGS.md section 3.24 `idle.*`, section 3.19
    /// `session.lock_grace_s`).
    Power,
    Apps,
    /// The companion, its models, memory and computer use (design/22-SETTINGS.md sections 3.26 to
    /// 3.30 `ai.*`, `agent.*`, `memory.*`, `cua.*`, `companion.*`).
    Intelligence,
    /// The file viewer's timings, steps, history, export defaults and decoder limits
    /// (design/22-SETTINGS.md section 3.31 `viewer.*`).
    Viewer,
    /// Any other page, named by its id: a third-party program's app id, a declared page such as
    /// `wallpaper`, or one this build has never heard of.
    App(String),
}

/// A page as the wire spells it: `{ kind = "dock" }` or `{ kind = "app", v = "<id>" }`. Reading
/// through this keeps a kind this build does not know instead of refusing the whole schema.
#[derive(Deserialize)]
struct PageWire {
    kind: String,
    #[serde(default)]
    v: Option<String>,
}

impl From<PageWire> for Page {
    fn from(wire: PageWire) -> Page {
        if wire.kind == "app" {
            return Page::declared(wire.v.as_deref().unwrap_or_default());
        }
        let id = PageId(wire.kind.replace('_', "-"));
        Page::fixed()
            .into_iter()
            .find(|page| page.id() == id)
            .unwrap_or(Page::App(wire.kind))
    }
}

impl Page {
    /// The twelve fixed pages, in the order Settings listed them.
    pub fn fixed() -> [Page; 12] {
        [
            Page::Appearance,
            Page::Dock,
            Page::MouseAndGestures,
            Page::KeyboardAndShortcuts,
            Page::Notifications,
            Page::Spaces,
            Page::Accounts,
            Page::Privacy,
            Page::Power,
            Page::Apps,
            Page::Intelligence,
            Page::Viewer,
        ]
    }

    /// The page whose id is `id`: a fixed page when the id is one of theirs, else `Page::App(id)`.
    /// Use this to name a declared page (`Page::declared("wallpaper")`).
    pub fn declared(id: &str) -> Page {
        let id = PageId::new(id);
        Page::fixed()
            .into_iter()
            .find(|page| page.id() == id)
            .unwrap_or(Page::App(id.0))
    }

    /// The page's id: the one name every entrance to Settings uses for it.
    pub fn id(&self) -> PageId {
        PageId(match self {
            Page::Appearance => "appearance".to_owned(),
            Page::Dock => "dock".to_owned(),
            Page::MouseAndGestures => "mouse-and-gestures".to_owned(),
            Page::KeyboardAndShortcuts => "keyboard-and-shortcuts".to_owned(),
            Page::Notifications => "notifications".to_owned(),
            Page::Spaces => "spaces".to_owned(),
            Page::Accounts => "accounts".to_owned(),
            Page::Privacy => "privacy".to_owned(),
            Page::Power => "power".to_owned(),
            Page::Apps => "apps".to_owned(),
            Page::Intelligence => "intelligence".to_owned(),
            Page::Viewer => "viewer".to_owned(),
            Page::App(id) => id.clone(),
        })
    }

    /// The name a person reads for the page: a fixed page by its name in section 9.3, any other
    /// page by its id. The declared title of a page is [`PageTable::title_of`](super::PageTable).
    pub fn label(&self) -> &str {
        match self {
            Page::Appearance => "Appearance",
            Page::Dock => "Desktop & Dock",
            Page::MouseAndGestures => "Mouse & Gestures",
            Page::KeyboardAndShortcuts => "Keyboard & Shortcuts",
            Page::Notifications => "Notifications",
            Page::Spaces => "Spaces",
            Page::Accounts => "Accounts",
            Page::Privacy => "Privacy & Security",
            Page::Power => "Power",
            Page::Apps => "Apps",
            Page::Intelligence => "Intelligence",
            Page::Viewer => "Viewer",
            Page::App(id) => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Page, PageId};

    #[test]
    fn a_fixed_page_is_labelled_as_settings_names_it_and_keeps_its_wire_form() {
        const CASES: &[(Page, &str, &str, &str)] = &[
            (Page::Appearance, "Appearance", "appearance", "appearance"),
            (Page::Dock, "Desktop & Dock", "dock", "dock"),
            (
                Page::MouseAndGestures,
                "Mouse & Gestures",
                "mouse_and_gestures",
                "mouse-and-gestures",
            ),
            (
                Page::KeyboardAndShortcuts,
                "Keyboard & Shortcuts",
                "keyboard_and_shortcuts",
                "keyboard-and-shortcuts",
            ),
            (
                Page::Notifications,
                "Notifications",
                "notifications",
                "notifications",
            ),
            (Page::Spaces, "Spaces", "spaces", "spaces"),
            (Page::Accounts, "Accounts", "accounts", "accounts"),
            (Page::Privacy, "Privacy & Security", "privacy", "privacy"),
            (Page::Power, "Power", "power", "power"),
            (Page::Apps, "Apps", "apps", "apps"),
            (
                Page::Intelligence,
                "Intelligence",
                "intelligence",
                "intelligence",
            ),
            (Page::Viewer, "Viewer", "viewer", "viewer"),
        ];
        for (page, label, kind, id) in CASES {
            assert_eq!(page.label(), *label, "{page:?}");
            assert_eq!(page.id(), PageId::new(id), "{page:?}");
            assert_eq!(Page::declared(id), *page, "{id}");
            let wire = format!(r#"{{"kind":"{kind}"}}"#);
            assert_eq!(serde_json::to_string(page).ok(), Some(wire.clone()));
            assert_eq!(
                serde_json::from_str::<Page>(&wire).ok().as_ref(),
                Some(page)
            );
            let text = format!("kind = \"{kind}\"\n");
            assert_eq!(toml::from_str::<Page>(&text).ok().as_ref(), Some(page));
        }
        assert_eq!(Page::fixed().len(), CASES.len());
    }

    #[test]
    fn any_other_page_is_named_by_its_id_and_never_refused() {
        let cases: [(&str, &str, Page); 4] = [
            (
                "a third-party page",
                r#"{"kind":"app","v":"com.example.App"}"#,
                Page::App("com.example.App".to_owned()),
            ),
            (
                "a page this build never heard of",
                r#"{"kind":"wallpaper"}"#,
                Page::App("wallpaper".to_owned()),
            ),
            (
                "a declared page",
                r#"{"kind":"app","v":"wallpaper"}"#,
                Page::App("wallpaper".to_owned()),
            ),
            (
                "a fixed page written as an app page",
                r#"{"kind":"app","v":"dock"}"#,
                Page::Dock,
            ),
        ];
        for (name, text, want) in cases {
            assert_eq!(
                serde_json::from_str::<Page>(text).ok(),
                Some(want),
                "{name}"
            );
        }
        assert_eq!(
            serde_json::to_string(&Page::declared("wallpaper"))
                .ok()
                .as_deref(),
            Some(r#"{"kind":"app","v":"wallpaper"}"#)
        );
    }
}
