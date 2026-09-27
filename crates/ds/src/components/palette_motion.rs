//! The command palette's list motions (design/26-DETAILS.md section 5.6): the rows' first-show
//! rise from the caller's [`RevealCue`] (sill Q372), and a group's Show More and Show Less
//! (Q373, section 5.4's group-expand grammar): the added rows rise in downward (`rise` with
//! `--stagger`, capped at 12), and on Show Less everything after the rows kept heals up by the
//! height the removed rows took (`heal`, which springs: the person ran the action). The palette
//! owns its rows and outlives its result sets, so it plays these itself; a later result set
//! replaces in place with no motion (R1, R12).
//!
//! The person's own Enter or click is not the only way a group's action runs (sill Q400): a
//! caller may run it itself (`sill debug launcher-key enter` stepping the keyboard machine
//! directly, a demo's own button) and just hand the palette the next `groups`, with no Enter or
//! click of its own for the palette to have seen. [`PaletteHandle::mark_group_action`] books that
//! change as though it were: the caller marks the group before making the change, and only a
//! resize of that same group plays; anything else (a new result set, or another group's) plays
//! nothing, as an unmarked caller-driven change always has.

use crate::components::menu_rows::RowsMotion;
use crate::components::palette_expand::{GroupResize, Resize, resized};
use crate::components::palette_group::GroupsKey;
use crate::components::palette_reveal::Reveal as Stops;
use crate::components::palette_shown::Change;
use crate::components::palette_stops::ShownGroup;
use crate::components::vocab::StaggerIndex;
use crate::detail::{RevealCue, Revealing, use_rise_on};
use crate::geometry::Px;
use crate::geometry::measure::{BUSY_ATTEMPTS, laid_out_rect};
use crate::motion::{Anim, MotionTimer, TimerPhase, use_motion_timer};
use crate::task::spawn_in;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::core::{current_scope_id, queue_effect};
use dioxus::prelude::*;

/// Whether a group's action ran since the results last changed: through the palette itself, or
/// (sill Q400) marked by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ran {
    Nothing,
    /// The palette's own Enter or click: any group's resize matches.
    Own,
    /// The caller marked this group (`PaletteHandle::mark_group_action`): only its resize
    /// matches; a resize of another group, or anything else, is a new result set.
    Marked(String),
}

/// Where the palette books a run of a group's action (Enter on it, or a click, or the caller's
/// mark): the next change of the results is that action's, and only then is a group's growing or
/// shrinking a Show More or Show Less rather than a new result set.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ActionBook(CopyValue<Ran>);

/// The palette's action book.
pub(crate) fn use_action_book() -> ActionBook {
    ActionBook(use_hook(|| CopyValue::new(Ran::Nothing)))
}

impl ActionBook {
    /// A group's action ran through the palette itself.
    pub(crate) fn ran(self) {
        self.book(Ran::Own);
    }

    /// The caller marked `group`'s action as about to run.
    fn ran_for(self, group: String) {
        self.book(Ran::Marked(group));
    }

    fn book(self, ran: Ran) {
        let mut slot = self.0;
        slot.set(ran);
    }

    /// Whether one ran since the last call, forgetting it.
    fn take(self) -> Ran {
        let mut ran = self.0;
        ran.replace(Ran::Nothing)
    }
}

/// A caller's mark on a `CommandPalette`'s next change of results (sill Q400): pass it as
/// `CommandPalette { handle: Some(handle) }`, then call [`mark_group_action`](Self::mark_group_action)
/// before making the change yourself (running a group's action without going through the
/// palette's own Enter or click, e.g. `sill debug launcher-key enter`, a demo's own button). The
/// palette plays that group's Show More or Show Less exactly as it would its own; any other
/// caller-driven change, unmarked, plays nothing, as before.
#[derive(Clone, Copy, PartialEq)]
pub struct PaletteHandle(ActionBook);

impl std::fmt::Debug for PaletteHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaletteHandle").finish_non_exhaustive()
    }
}

/// A handle a caller keeps across renders, like [`use_field_handle`](crate::use_field_handle)'s.
pub fn use_palette_handle() -> PaletteHandle {
    PaletteHandle(use_action_book())
}

impl PaletteHandle {
    /// Mark the next change of the palette's `groups` as `group`'s own action: its resize (Show
    /// More or Show Less) plays as though the person had run it through the palette itself. A
    /// change that turns out not to be that group resizing (a new result set, or another group's
    /// resize) plays nothing.
    pub fn mark_group_action(&self, group: impl Into<String>) {
        self.0.ran_for(group.into());
    }

    /// This handle's book, for the palette it names.
    pub(crate) fn book(self) -> ActionBook {
        self.0
    }
}

/// A group's rows moving, and by how far the rows after them heal.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupMotion {
    /// The group's header.
    pub title: String,
    /// Which of its rows take part.
    pub rows: RowsMotion,
    /// How far what follows the rows kept starts below its place (Show Less).
    pub dy: Option<Px>,
}

/// What the list plays this render.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ListMotion {
    /// The rows' first-show rise.
    pub reveal: Revealing,
    /// A group's Show More or Show Less.
    pub group: Option<GroupMotion>,
}

impl ListMotion {
    /// The `data-reveal` word while the rise plays (nothing otherwise).
    pub(crate) fn reveal_slug(&self) -> Option<&'static str> {
        match self.reveal {
            Revealing::Play => Some("play"),
            Revealing::Still => None,
        }
    }

    /// The list's inline `--dy` while what follows the rows kept heals.
    pub(crate) fn heal_style(&self) -> Option<String> {
        let dy = self.group.as_ref()?.dy?;
        Some(format!("--dy:{}px", dy.0))
    }

    /// The rows' part in `title`'s group.
    pub(crate) fn rows_of(&self, title: &str) -> RowsMotion {
        match &self.group {
            Some(group) if group.title == title => group.rows,
            Some(_) | None => RowsMotion::Still,
        }
    }
}

/// The palette's list motions: the rise `reveal` asks for (counting each opening `change`
/// reports), and a group's resize after the person ran its action (`book`), measured through
/// `stops`, the stops' mounted elements.
pub(crate) fn use_list_motion<T: Clone + PartialEq + 'static>(
    reveal: RevealCue,
    change: Change,
    key: &GroupsKey<T>,
    shown: &[ShownGroup<'_, T>],
    motion: Book,
) -> ListMotion {
    let openings = use_openings(change);
    let reveal = use_rise_on(reveal.key(openings));
    let resize = use_resize(key, motion.actions);
    ListMotion {
        reveal,
        group: use_group_motion(resize, shown, motion.stops),
    }
}

/// What the list's motions read from the palette: its action book and its stops' elements.
#[derive(Clone, Copy)]
pub(crate) struct Book {
    /// Where a run of a group's action is booked.
    pub actions: ActionBook,
    /// The stops' mounted elements.
    pub stops: Stops,
}

/// How many times the palette has been shown again since it mounted.
fn use_openings(change: Change) -> u32 {
    let mut openings = use_hook(|| CopyValue::new(0u32));
    if change == Change::Show {
        let next = openings.peek().wrapping_add(1);
        openings.set(next);
    }
    *openings.peek()
}

/// A change of the results this render: the group resize it is, when the person ran an action.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Results {
    /// As they were.
    Same,
    /// Changed: a resize after an action, or `None` for a new result set.
    Changed(Option<GroupResize>),
}

fn use_resize<T: Clone + PartialEq + 'static>(key: &GroupsKey<T>, book: ActionBook) -> Results {
    let mut last = use_hook(|| CopyValue::new(key.clone()));
    if *last.peek() == *key {
        return Results::Same;
    }
    let before = last.replace(key.clone());
    Results::Changed(match book.take() {
        Ran::Own => resized(&before, key),
        Ran::Marked(group) => resized(&before, key).filter(|resize| resize.title == group),
        Ran::Nothing => None,
    })
}

/// How tall a group's added rows were when they last rose: what Show Less heals by.
#[derive(Debug, Clone, PartialEq)]
struct Span {
    title: String,
    rows: usize,
    height: Px,
}

/// The group motion playing: started by a resize, kept while its timer runs, dropped by any
/// other change of the results.
fn use_group_motion<T>(
    results: Results,
    shown: &[ShownGroup<'_, T>],
    stops: Stops,
) -> Option<GroupMotion> {
    let rise = use_motion_timer(Anim::Rise);
    let heal = use_motion_timer(Anim::Heal);
    let mut playing = use_hook(|| CopyValue::new(None::<GroupMotion>));
    let spans = use_hook(|| CopyValue::new(None::<Span>));
    let scope = use_hook(current_scope_id);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let running = [rise.phase(), heal.phase()].contains(&TimerPhase::Running);
    let Results::Changed(resize) = results else {
        return running.then(|| playing.peek().clone()).flatten();
    };
    let timers = Timers {
        rise,
        heal,
        settled,
        stops,
        spans,
        scope,
    };
    let next = resize.and_then(|resize| {
        let first = shown
            .iter()
            .find(|group| group.group.title == resize.title)
            .map(|group| group.first);
        start(resize, first, timers)
    });
    if next.is_none() {
        queue_effect(move || {
            rise.cancel();
            heal.cancel();
        });
    }
    playing.set(next.clone());
    next
}

/// What a group motion starts and measures with.
#[derive(Clone, Copy)]
struct Timers {
    rise: MotionTimer,
    heal: MotionTimer,
    settled: EventHandler<()>,
    stops: Stops,
    spans: CopyValue<Option<Span>>,
    scope: ScopeId,
}

/// Start `resize`'s motion (its group's first stop `first`): the added rows rise, and their span
/// is measured once they are laid out; or what follows the rows kept heals by the span last
/// measured for this group (none measured: no heal, the list just closes up).
fn start(resize: GroupResize, first: Option<usize>, timers: Timers) -> Option<GroupMotion> {
    match resize.resize {
        Resize::Grew { kept, added } if added > 0 => {
            let rise = timers.rise;
            queue_effect(move || rise.start_staggered(StaggerIndex::new(added - 1)));
            if let Some(first) = first {
                measure(&resize.title, first + kept, added, timers);
            }
            Some(GroupMotion {
                title: resize.title,
                rows: RowsMotion::Rise { from: kept },
                dy: None,
            })
        }
        Resize::Shrank { kept, removed } if kept > 0 && removed > 0 => {
            let dy = healed_by(timers.spans.peek().as_ref(), &resize.title, removed)?;
            let (heal, settled) = (timers.heal, timers.settled);
            queue_effect(move || heal.start(settled));
            Some(GroupMotion {
                title: resize.title,
                rows: RowsMotion::HealAfter { at: kept - 1 },
                dy: Some(dy),
            })
        }
        Resize::Grew { .. } | Resize::Shrank { .. } => None,
    }
}

/// The height `removed` rows of `title` took, from the span last measured for it.
fn healed_by(span: Option<&Span>, title: &str, removed: usize) -> Option<Px> {
    let span = span.filter(|span| span.title == title && span.rows > 0)?;
    Some(Px(span.height.0 * removed as f32 / span.rows as f32))
}

/// Read, once laid out, how tall the `rows` added rows from stop `from` are, and keep it for
/// `title`'s Show Less.
fn measure(title: &str, from: usize, rows: usize, timers: Timers) {
    let title = title.to_string();
    let Timers {
        stops,
        mut spans,
        scope,
        ..
    } = timers;
    queue_effect(move || {
        spawn_in(scope, async move {
            if let Some(height) = span(stops, from, from + rows - 1).await {
                spans.set(Some(Span {
                    title,
                    rows,
                    height,
                }));
            }
        });
    });
}

/// From the top of stop `first` to the bottom of stop `last`, once both have mounted and laid out.
async fn span(stops: Stops, first: usize, last: usize) -> Option<Px> {
    for _ in 0..BUSY_ATTEMPTS {
        sleep(FRAME_SLACK).await;
        if let (Some(top), Some(bottom)) = (stops.element(first), stops.element(last)) {
            let top = laid_out_rect(&top.0).await?.top();
            let bottom = laid_out_rect(&bottom.0).await?.bottom();
            return Some(Px(bottom.0 - top.0));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Span, healed_by};
    use crate::geometry::Px;

    #[test]
    fn show_less_heals_by_the_share_of_the_span_it_removes() {
        let span = Span {
            title: "Apps".to_string(),
            rows: 4,
            height: Px(160.0),
        };
        let cases: &[(Option<&Span>, &str, usize, Option<Px>)] = &[
            (Some(&span), "Apps", 4, Some(Px(160.0))),
            (Some(&span), "Apps", 2, Some(Px(80.0))),
            (Some(&span), "Files", 4, None),
            (None, "Apps", 4, None),
        ];
        for &(span, title, removed, want) in cases {
            assert_eq!(healed_by(span, title, removed), want, "{title} {removed}");
        }
    }
}
