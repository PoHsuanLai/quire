//! Where an app's menu is found: the bus name and object path its exported menu is served at,
//! derived from the app's Wayland `app_id` alone. The shell reads a focused window's `app_id`
//! from the compositor, derives the same address and asks the bus whether anyone owns it; no
//! registration, no registry to lose when the shell restarts, and the bus clears the name when
//! the app exits. (A compositor that offers `org_kde_kwin_appmenu` hands the address over per
//! window instead; the address is the same one, design/27 section 5.2.)

/// The bus name and object path of one app's `com.canonical.dbusmenu`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppMenuAddress {
    /// The well-known name: [`AppMenuAddress::SERVICE_PREFIX`] and the sanitised `app_id`.
    pub service: String,
    /// The object path ([`AppMenuAddress::PATH`]).
    pub path: String,
}

/// An `app_id` no address can be made from.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AddressError {
    /// The id is empty.
    #[error("an empty app id has no menu address")]
    Empty,
    /// The name would pass the bus's 255-byte limit.
    #[error("the app id {0:?} makes a bus name over 255 bytes")]
    TooLong(String),
}

impl AppMenuAddress {
    /// What every exported menu's bus name starts with: list the bus's names for it to find
    /// every app that exports a menu.
    pub const SERVICE_PREFIX: &'static str = "org.quire.AppMenu.";
    /// The object every exporter serves the menu at.
    pub const PATH: &'static str = "/org/quire/AppMenu";

    /// The address of the app whose Wayland `app_id` is `app_id`. Each dot-separated element of
    /// the id is made a legal bus-name element: other characters become `_`, an empty element
    /// is `_`, and one that starts with a digit gets a `_` before it. Ids that only differ in
    /// such characters share an address; apps of one id are one app.
    pub fn for_app_id(app_id: &str) -> Result<AppMenuAddress, AddressError> {
        if app_id.trim().is_empty() {
            return Err(AddressError::Empty);
        }
        let elements: Vec<String> = app_id.split('.').map(element).collect();
        let service = format!("{}{}", Self::SERVICE_PREFIX, elements.join("."));
        if service.len() > 255 {
            return Err(AddressError::TooLong(app_id.to_owned()));
        }
        Ok(AppMenuAddress {
            service,
            path: Self::PATH.to_owned(),
        })
    }
}

fn element(raw: &str) -> String {
    let legal: String = raw
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' => c,
            _ => '_',
        })
        .collect();
    match legal.chars().next() {
        None => "_".to_owned(),
        Some('0'..='9') => format!("_{legal}"),
        Some(_) => legal,
    }
}
