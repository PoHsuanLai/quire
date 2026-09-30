//! The props every component takes (design/30 rule R8): an `id`, the consumer's own `data-*`
//! and class, the accessible name, and the element's `mounted` handle.
//!
//! One struct so the five travel together and a component names them once
//! (`#[props(default)] common: Common`), instead of five props each spelt its own way.

use crate::root::pass_through::{DataAttr, ExtraClass, attributes, class_list};
use dioxus::core::Attribute;
use dioxus::prelude::*;
use std::fmt;

/// What a consumer may put on any quire component's own element.
///
/// `data` and `extra_class` are checked when built ([`DataName::parse`], [`ExtraClass::parse`]):
/// a `ds-` name or class, or a `data-*` name quire writes itself, is refused, so nothing added
/// here can restyle the element through quire's rules.
///
/// [`DataName::parse`]: crate::root::pass_through::DataName::parse
/// [`ExtraClass::parse`]: crate::root::pass_through::ExtraClass::parse
#[derive(Clone, PartialEq, Default)]
pub struct Common {
    /// The element's `id`.
    pub id: Option<String>,
    /// The consumer's own `data-*` attributes.
    pub data: Vec<DataAttr>,
    /// The consumer's own classes, appended after quire's.
    pub extra_class: Option<ExtraClass>,
    /// The accessible name, where the component's visible label does not give one.
    pub aria_label: Option<String>,
    /// Hears the element once it is mounted, for a menu or popover anchored to it.
    pub mounted: Option<EventHandler<MountedEvent>>,
}

impl fmt::Debug for Common {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Common")
            .field("id", &self.id)
            .field("data", &self.data)
            .field("extra_class", &self.extra_class)
            .field("aria_label", &self.aria_label)
            .field("mounted", &self.mounted.is_some())
            .finish()
    }
}

impl Common {
    /// `own` (quire's class list) with the consumer's classes after it.
    pub fn class(&self, own: &str) -> String {
        class_list(own, self.extra_class.as_ref())
    }

    /// The consumer's `data-*` attributes, for an element's spread.
    pub fn data_attributes(&self) -> Vec<Attribute> {
        attributes(&self.data)
    }

    /// Hand `event` to the consumer's `mounted` handler, if it gave one.
    pub fn mounted(&self, event: MountedEvent) {
        if let Some(mounted) = &self.mounted {
            mounted.call(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Common;
    use crate::root::pass_through::{DataAttr, DataName, ExtraClass};

    #[test]
    fn the_class_list_puts_the_consumers_after_quires() {
        let extra = ExtraClass::parse("fold-more");
        let cases = [
            (Common::default(), "ds-button"),
            (
                Common {
                    extra_class: extra.ok(),
                    ..Common::default()
                },
                "ds-button fold-more",
            ),
        ];
        for (common, want) in cases {
            assert_eq!(common.class("ds-button"), want);
        }
    }

    #[test]
    fn the_data_attributes_are_the_consumers_own() {
        let common = Common {
            data: DataName::parse("folder")
                .map(|name| vec![DataAttr::new(name, "INBOX")])
                .unwrap_or_default(),
            ..Common::default()
        };
        let written: Vec<_> = common
            .data_attributes()
            .into_iter()
            .map(|attribute| attribute.name)
            .collect();
        assert_eq!(written, ["data-folder"]);
    }
}
