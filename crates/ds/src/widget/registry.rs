//! The widget registry (design/23-WIDGETS.md section 9.4): every widget a host can place, by
//! kind, with what a picker needs to show it (its name, its sizes, its placeholder drawn on the
//! card) without knowing its type. A host builds one ([`WidgetRegistry::quire`] holds quire's
//! own), adds its own and the apps' widgets, and provides it with
//! [`provide_widget_registry`]; a picker reads it with [`use_widget_registry`].

use crate::components::text_runs::Text;
use crate::components::widget_kind::{Lift, WidgetHost, WidgetSize};
use crate::widget::battery::BatteryWidget;
use crate::widget::calendar::MonthWidget;
use crate::widget::card::WidgetCard;
use crate::widget::clock::WorldClockWidget;
use crate::widget::contract::{Widget, WidgetKind};
use crate::widget::timeline::Timeline;
use dioxus::prelude::*;

/// One registered widget, its type erased.
#[derive(Clone)]
pub struct WidgetInfo {
    /// Its kind.
    pub kind: WidgetKind,
    /// Its name in a picker.
    pub name: Text,
    /// Its one line in a picker.
    pub description: Text,
    /// The sizes it draws.
    pub sizes: &'static [WidgetSize],
    preview: fn(WidgetSize, WidgetHost, Lift) -> Element,
}

impl std::fmt::Debug for WidgetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetInfo")
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("sizes", &self.sizes)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WidgetInfo {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.name == other.name
            && self.description == other.description
            && self.sizes == other.sizes
    }
}

impl WidgetInfo {
    /// `W`'s entry in a registry.
    pub fn of<W: Widget>() -> Self {
        WidgetInfo {
            kind: W::kind(),
            name: W::name(),
            description: W::description(),
            sizes: W::sizes(),
            preview: preview::<W>,
        }
    }

    /// Its card at `size` in `host`, showing its preview entry, lifted or at rest: what a
    /// picker shows.
    pub fn preview(&self, size: WidgetSize, host: WidgetHost, lift: Lift) -> Element {
        (self.preview)(size, host, lift)
    }
}

/// `W`'s card showing its preview entry.
fn preview<W: Widget>(size: WidgetSize, host: WidgetHost, lift: Lift) -> Element {
    rsx! {
        WidgetCard::<W> { widget: W::default(), timeline: Timeline::now(W::preview(size)), size, host, lift }
    }
}

/// A kind registered twice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the widget kind {0:?} is already registered")]
pub struct TakenKind(pub WidgetKind);

/// The widgets a host can place, in the order they were added.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WidgetRegistry {
    infos: Vec<WidgetInfo>,
}

impl WidgetRegistry {
    /// quire's own widgets: the batteries, the world clock and the month.
    pub fn quire() -> Self {
        WidgetRegistry::default()
            .with::<BatteryWidget>()
            .and_then(WidgetRegistry::with::<WorldClockWidget>)
            .and_then(WidgetRegistry::with::<MonthWidget>)
            .unwrap_or_default()
    }

    /// This registry with `W` added, or the kind it already holds.
    pub fn with<W: Widget>(self) -> Result<Self, TakenKind> {
        let info = WidgetInfo::of::<W>();
        if self.get(&info.kind).is_some() {
            return Err(TakenKind(info.kind));
        }
        let mut infos = self.infos;
        infos.push(info);
        Ok(WidgetRegistry { infos })
    }

    /// The widget of `kind`.
    pub fn get(&self, kind: &WidgetKind) -> Option<&WidgetInfo> {
        self.infos.iter().find(|info| info.kind == *kind)
    }

    /// Every widget, in the order added.
    pub fn all(&self) -> &[WidgetInfo] {
        &self.infos
    }
}

/// Provide `registry` to the subtree.
pub fn provide_widget_registry(registry: WidgetRegistry) -> WidgetRegistry {
    use_context_provider(|| registry)
}

/// The registry an ancestor provided, or quire's own when none did.
pub fn use_widget_registry() -> WidgetRegistry {
    try_use_context::<WidgetRegistry>().unwrap_or_else(WidgetRegistry::quire)
}

#[cfg(test)]
mod tests {
    use super::{TakenKind, WidgetRegistry};
    use crate::widget::battery::BatteryWidget;
    use crate::widget::contract::{Widget, WidgetKind};

    #[test]
    fn quire_registers_its_three_widgets_once_each() {
        let registry = WidgetRegistry::quire();
        let kinds: Vec<&str> = registry
            .all()
            .iter()
            .map(|info| info.kind.as_str())
            .collect();
        assert_eq!(kinds, ["quire.battery", "quire.world-clock", "quire.month"]);
        assert_eq!(
            registry
                .get(&WidgetKind::fixed("quire.battery"))
                .map(|info| info.sizes),
            Some(BatteryWidget::sizes())
        );
        assert_eq!(
            registry.with::<BatteryWidget>(),
            Err(TakenKind(WidgetKind::fixed("quire.battery")))
        );
    }
}
