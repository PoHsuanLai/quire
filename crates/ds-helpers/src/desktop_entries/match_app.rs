//! Which entry a window belongs to. Pure: the keys come in, an index goes out.
//!
//! Every rule compares whole tokens: an id is compared whole or by whole dot-separated labels,
//! never by substring (`fox` must not match `org.mozilla.firefox`).

use std::borrow::Borrow;

/// What an entry says about the windows it owns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppKeys {
    /// The desktop id without `.desktop` ([`DesktopId::stem`](super::DesktopId::stem)).
    pub id: String,
    /// `StartupWMClass=`.
    pub startup_wm_class: Option<String>,
    /// `X-Flatpak=`: the Flatpak application id.
    pub flatpak: Option<String>,
}

/// The position in `apps` of the entry a toplevel with Wayland `app_id` (and, through XWayland,
/// `wm_class`) belongs to. The first rule that matches any entry wins; within a rule, the first
/// entry. The rules, in order:
///
/// 1. the id is `app_id`;
/// 2. the id is `app_id` in any case;
/// 3. `StartupWMClass` is `app_id` or `wm_class`, in any case;
/// 4. the id ends in `app_id` after a dot (`firefox` for `org.mozilla.firefox`), in any case;
/// 5. the Flatpak id is `app_id`, in any case.
pub fn match_app<K: Borrow<AppKeys>>(
    app_id: &str,
    wm_class: Option<&str>,
    apps: &[K],
) -> Option<usize> {
    let app_id = app_id.trim();
    if app_id.is_empty() && wm_class.is_none_or(|class| class.trim().is_empty()) {
        return None;
    }
    let lower = app_id.to_lowercase();
    let rules: [&dyn Fn(&AppKeys) -> bool; 5] = [
        &|app| app.id == app_id,
        &|app| app.id.to_lowercase() == lower,
        &|app| {
            app.startup_wm_class.as_deref().is_some_and(|class| {
                same_class(class, app_id) || wm_class.is_some_and(|w| same_class(class, w))
            })
        },
        &|app| !lower.is_empty() && is_label_suffix(&app.id.to_lowercase(), &lower),
        &|app| {
            let flatpak = app.flatpak.as_deref();
            flatpak.is_some_and(|id| !lower.is_empty() && id.to_lowercase() == lower)
        },
    ];
    rules.iter().find_map(|rule| {
        apps.iter()
            .position(|app| rule(Borrow::<AppKeys>::borrow(app)))
    })
}

/// `StartupWMClass` compares whole, ignoring case; an empty class matches nothing.
fn same_class(class: &str, given: &str) -> bool {
    let given = given.trim();
    !given.is_empty() && class.eq_ignore_ascii_case(given)
}

/// Whether `suffix` is the last whole dot-separated labels of `id`.
fn is_label_suffix(id: &str, suffix: &str) -> bool {
    id.len() > suffix.len() && id.ends_with(suffix) && id[..id.len() - suffix.len()].ends_with('.')
}
