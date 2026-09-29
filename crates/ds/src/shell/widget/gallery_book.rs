//! What each layout the host hands Edit Widgets added: the gallery keeps no layout,
//! so it learns that an Add landed by seeing the new placement in the next layout, never from
//! the click itself (a refused edit, a full desktop, lands nothing and shows no check). The book
//! remembers the identities it saw last, the Add the person asked for, and, per widget and
//! surface, how many adds have landed and who caused the latest: the Add button settles to a
//! check on each new landing, and the new row rises in.

use crate::motion::detail::touch::Touch;
use crate::shell::widget::contract::WidgetKind;
use crate::shell::widget::kind::WidgetHost;
use crate::shell::widget::layout::WidgetLayout;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::shell::catalog::placement::PlacementId;

/// The Add the person pressed, waiting for its placement to appear.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Asked {
    /// The widget.
    pub(crate) kind: WidgetKind,
    /// The surface.
    pub(crate) host: WidgetHost,
    /// The press.
    pub(crate) touch: Touch,
}

/// How many adds of one widget to one surface have landed, and who caused the latest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Landing {
    /// Landings counted from 1; 0 is none yet.
    pub(crate) serial: u32,
    /// Who caused the latest: the person's press, or anything else.
    pub(crate) touch: Touch,
}

/// The gallery's memory between layouts.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Book {
    seen: Option<BTreeSet<PlacementId>>,
    asked: Option<Asked>,
    landed: HashMap<(WidgetKind, WidgetHost), Landing>,
}

impl Book {
    /// The person pressed an Add.
    pub(crate) fn ask(&mut self, asked: Asked) {
        self.asked = Some(asked);
    }

    /// Read `layout`: the placements that are new since the last one, each with who caused it
    /// (the person's press when it is the widget and the surface they asked for, else remote).
    /// The first layout read adds nothing: what was placed before the gallery opened is not new.
    /// Any change to the placements settles the ask, landed or not.
    pub(crate) fn read(&mut self, layout: &WidgetLayout) -> BTreeMap<PlacementId, Touch> {
        let now: BTreeSet<PlacementId> = layout.items().iter().map(|item| item.id).collect();
        let Some(before) = self.seen.replace(now.clone()) else {
            return BTreeMap::new();
        };
        if before == now {
            return BTreeMap::new();
        }
        let asked = self.asked.take();
        let mut arrived = BTreeMap::new();
        for item in layout
            .items()
            .iter()
            .filter(|item| !before.contains(&item.id))
        {
            let host = item.at.host();
            let touch = match &asked {
                Some(asked) if asked.kind == item.kind && asked.host == host => asked.touch,
                Some(_) | None => Touch::Remote,
            };
            let landing = self.landed.entry((item.kind.clone(), host)).or_default();
            *landing = Landing {
                serial: landing.serial + 1,
                touch,
            };
            arrived.insert(item.id, touch);
        }
        arrived
    }

    /// The landings of `kind` on `host` so far.
    pub(crate) fn landing(&self, kind: &WidgetKind, host: WidgetHost) -> Landing {
        self.landed
            .get(&(kind.clone(), host))
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::{Asked, Book, Landing};
    use crate::motion::detail::touch::{Contact, Touch};
    use crate::shell::widget::kind::{WidgetHost, WidgetSize};
    use crate::shell::widget::layout::{DesktopGrid, WidgetEdit, WidgetLayout, apply};
    use crate::shell::widget::{battery::BatteryWidget, calendar::MonthWidget, contract::Widget};

    const GRID: DesktopGrid = DesktopGrid {
        columns: 4,
        rows: 3,
    };

    fn add(
        layout: &WidgetLayout,
        kind: crate::shell::widget::contract::WidgetKind,
        host: WidgetHost,
    ) -> WidgetLayout {
        let edit = WidgetEdit::Add {
            kind,
            size: WidgetSize::Small,
            host,
        };
        apply(layout.clone(), edit, GRID).unwrap_or_else(|_| layout.clone())
    }

    #[test]
    fn only_a_placement_new_since_the_last_layout_arrives() {
        let mut book = Book::default();
        let opened = add(
            &WidgetLayout::default(),
            BatteryWidget::kind(),
            WidgetHost::Desktop,
        );
        assert!(
            book.read(&opened).is_empty(),
            "placed before the gallery opened"
        );
        assert!(book.read(&opened).is_empty(), "the same layout again");
        let contact = Touch::Contact(Contact::for_tests());
        book.ask(Asked {
            kind: MonthWidget::kind(),
            host: WidgetHost::Tile,
            touch: contact,
        });
        let next = add(&opened, MonthWidget::kind(), WidgetHost::Tile);
        let arrived = book.read(&next);
        assert_eq!(arrived.len(), 1);
        assert_eq!(
            arrived.values().next(),
            Some(&contact),
            "the press it answers"
        );
        assert_eq!(
            book.landing(&MonthWidget::kind(), WidgetHost::Tile),
            Landing {
                serial: 1,
                touch: contact
            }
        );
        assert_eq!(
            book.landing(&MonthWidget::kind(), WidgetHost::Desktop),
            Landing::default(),
            "another surface has its own count"
        );
        let remote = add(&next, BatteryWidget::kind(), WidgetHost::Tile);
        assert_eq!(
            book.read(&remote).values().next(),
            Some(&Touch::Remote),
            "added from elsewhere: no press to answer"
        );
    }

    #[test]
    fn a_refused_add_lands_nothing() {
        let mut book = Book::default();
        let layout = WidgetLayout::default();
        assert!(book.read(&layout).is_empty());
        book.ask(Asked {
            kind: BatteryWidget::kind(),
            host: WidgetHost::Desktop,
            touch: Touch::Remote,
        });
        assert!(book.read(&layout).is_empty());
        assert_eq!(
            book.landing(&BatteryWidget::kind(), WidgetHost::Desktop)
                .serial,
            0
        );
    }
}
