//! AccountPicker: a pop-up button listing the accounts an app may use, with "Add Account..." under
//! them (design/31 section 5.1). The chosen account is the host's.

use super::adapter::{ChoiceMenu, Disc};
use super::model::{AccountChoice, ChoiceKey, PickerChoice};
use dioxus::prelude::*;
use ds::components::content::avatar::AvatarSize;
use ds::components::menus::item::item::MenuItem;

/// The menu's lines: one per account, a rule, then "Add Account...".
pub(crate) fn menu_items(choices: &[AccountChoice]) -> Vec<MenuItem<PickerChoice>> {
    choices
        .iter()
        .map(|choice| {
            MenuItem::new(
                PickerChoice::Account(choice.key.clone()),
                choice.label.clone(),
            )
        })
        .chain([
            MenuItem::Separator,
            MenuItem::new(PickerChoice::Add, "Add Account\u{2026}"),
        ])
        .collect()
}

/// The pop-up, led by the round mark of the account it shows. `chosen` is that account; `on_pick` hears an account or "Add Account...".
#[component]
pub fn AccountPicker(
    accounts: Vec<AccountChoice>,
    #[props(default)] chosen: Option<ChoiceKey>,
    on_pick: EventHandler<PickerChoice>,
) -> Element {
    let items = menu_items(&accounts);
    let provider = chosen
        .as_ref()
        .and_then(|key| accounts.iter().find(|choice| &choice.key == key))
        .map(|choice| choice.provider);
    rsx! {
        span { class: "ds-acc-badge",
            if let Some(provider) = provider {
                Disc { provider, size: AvatarSize::Size28 }
            }
            ChoiceMenu { items, value: chosen.map(PickerChoice::Account), title: "Account", onpick: on_pick }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::menu_items;
    use crate::accounts::model::{AccountChoice, ChoiceKey, PickerChoice};
    use ds::components::content::provider_mark::MarkProvider;
    use ds::components::menus::item::item::MenuItem;

    #[test]
    fn the_menu_lists_each_account_then_a_rule_then_add() {
        let choices = ["a@example.org", "b@example.org"].map(|label| AccountChoice {
            key: ChoiceKey(label.to_owned()),
            label: label.to_owned(),
            provider: MarkProvider::Imap,
        });
        let items = menu_items(&choices);
        let shape: Vec<String> = items
            .iter()
            .map(|item| match item {
                MenuItem::Item { value, title, .. } => format!("{value:?} {title}"),
                MenuItem::Separator => "rule".to_owned(),
                other => format!("{other:?}"),
            })
            .collect();
        assert_eq!(
            shape,
            [
                "Account(ChoiceKey(\"a@example.org\")) a@example.org",
                "Account(ChoiceKey(\"b@example.org\")) b@example.org",
                "rule",
                "Add Add Account\u{2026}",
            ]
        );
        assert!(matches!(
            items.last(),
            Some(MenuItem::Item {
                value: PickerChoice::Add,
                ..
            })
        ));
    }
}
