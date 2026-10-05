//! The pure dbusmenu data as D-Bus values.

use ds::components::menus::export::{DbusItem, DbusLayout, Prop};
use serde::Serialize;
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Type, Value};

/// `(ia{sv}av)`: an item, its properties and its rows (each a variant of the same shape).
#[derive(Debug, Type, Serialize, Value, OwnedValue)]
#[zvariant(crate = "zbus::zvariant")]
pub(super) struct Layout {
    id: i32,
    props: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

/// A value that could not be made a D-Bus value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WireError(pub String);

impl From<zbus::zvariant::Error> for WireError {
    fn from(error: zbus::zvariant::Error) -> Self {
        WireError(error.to_string())
    }
}

pub(super) type Props = HashMap<String, OwnedValue>;

fn owned<'a>(value: impl Into<Value<'a>>) -> Result<OwnedValue, WireError> {
    Ok(OwnedValue::try_from(value.into())?)
}

/// The wire value of one property.
fn value(prop: &Prop) -> Result<OwnedValue, WireError> {
    use ds::base::vocab::Availability;
    use ds::components::menus::export::{ChildrenDisplay, ItemType, ToggleType};
    match prop {
        Prop::Type(ItemType::Standard) => owned("standard"),
        Prop::Type(ItemType::Separator) => owned("separator"),
        Prop::Label(label) => owned(label.as_str()),
        Prop::Enabled(availability) => owned(*availability == Availability::Enabled),
        Prop::ToggleType(ToggleType::Checkmark) => owned("checkmark"),
        Prop::ToggleState(state) => owned(state.wire()),
        Prop::Shortcut(chord) => owned(vec![chord.0.clone()]),
        Prop::ChildrenDisplay(ChildrenDisplay::Submenu) => owned("submenu"),
        Prop::CommandId(command) => owned(command.0.as_str()),
    }
}

/// `a{sv}`.
pub(super) fn props(props: &[Prop]) -> Result<Props, WireError> {
    props
        .iter()
        .map(|prop| Ok((prop.name().to_owned(), value(prop)?)))
        .collect()
}

/// One row of `GetGroupProperties` or `ItemsPropertiesUpdated`: `(ia{sv})`.
pub(super) fn item(item: &DbusItem) -> Result<(i32, Props), WireError> {
    Ok((item.id.0, props(&item.props)?))
}

/// The layout as the method returns it.
pub(super) fn layout(layout: &DbusLayout) -> Result<Layout, WireError> {
    let children = layout
        .children
        .iter()
        .map(|child| owned(self::layout(child)?))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Layout {
        id: layout.item.id.0,
        props: props(&layout.item.props)?,
        children,
    })
}
