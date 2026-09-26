//! The moving parts of a persona (design/24-PERSONA.md sections 3.1 and 4): the eyes, the
//! brows and the mouth as each mood draws them, the accessory worn over them, and the sleeping
//! `z`. Each is its own layer so the stylesheet can move it.

use super::geometry::{Head, Spacing, circle, ellipse, pt};
use super::mark::Mark;
use super::mood::{Detail, Mood};
use super::palette;
use super::spec::{Accessory, Brows, Creature, Eyes, Mouth, PersonaSpec};

/// Where the eyes are: their height and the two centres.
fn eye_centres(spec: &PersonaSpec) -> [(f64, f64); 2] {
    let head = Head::of(spec.head);
    let spacing = match spec.eyes {
        Eyes::Wide => Spacing::Far,
        Eyes::Dot | Eyes::Oval | Eyes::Shine => Spacing::Near,
    };
    let (y, dx) = head.eyes(spacing);
    [(head.cx - dx, y), (head.cx + dx, y)]
}

/// The eyes' height as a share of the canvas: the origin a blink closes towards.
pub(crate) fn eye_line(spec: &PersonaSpec) -> f64 {
    eye_centres(spec)[0].1
}

/// The eyes as `mood` draws them.
pub(crate) fn eyes(spec: &PersonaSpec, mood: Mood, detail: Detail) -> Vec<Mark> {
    let ink = palette::ink().hex();
    let grow = match detail {
        Detail::Fine => 1.0,
        Detail::Coarse => 1.25,
    };
    let mut marks = Vec::new();
    for (side, (x, y)) in [-1.0, 1.0].into_iter().zip(eye_centres(spec)) {
        match mood {
            Mood::Idle | Mood::Attentive => marks.extend(open((x, y), spec.eyes, grow, detail)),
            Mood::Happy => marks.push(Mark::stroke(
                format!(
                    "M{}Q{} {}",
                    pt((x - 5.0, y + 1.5)),
                    pt((x, y - 5.0)),
                    pt((x + 5.0, y + 1.5))
                ),
                ink.clone(),
                3.2 * grow,
            )),
            Mood::Asleep => marks.push(Mark::stroke(
                format!(
                    "M{}Q{} {}",
                    pt((x - 5.0, y - 0.5)),
                    pt((x, y + 4.5)),
                    pt((x + 5.0, y - 0.5))
                ),
                ink.clone(),
                3.0 * grow,
            )),
            Mood::Wince => marks.push(Mark::stroke(
                format!(
                    "M{}L{}L{}",
                    pt((x + side * 4.0, y - 4.0)),
                    pt((x - side * 3.5, y)),
                    pt((x + side * 4.0, y + 4.0))
                ),
                ink.clone(),
                3.0 * grow,
            )),
        }
    }
    marks
}

/// One open eye.
fn open(at: (f64, f64), eyes: Eyes, grow: f64, detail: Detail) -> Vec<Mark> {
    let ink = palette::ink().hex();
    match eyes {
        Eyes::Dot | Eyes::Wide => vec![Mark::fill(circle(at, 3.7 * grow), ink)],
        Eyes::Oval => vec![Mark::fill(ellipse(at, 3.0 * grow, 4.7 * grow), ink)],
        Eyes::Shine => {
            let mut marks = vec![Mark::fill(circle(at, 5.0 * grow), ink)];
            if detail == Detail::Fine {
                marks.push(Mark::fill(
                    circle((at.0 - 1.7, at.1 - 1.9), 1.7),
                    palette::fleck().hex(),
                ));
            }
            marks
        }
    }
}

/// The brows as `mood` draws them; none at 28 px.
pub(crate) fn brows(spec: &PersonaSpec, mood: Mood, detail: Detail) -> Vec<Mark> {
    if detail == Detail::Coarse || spec.brows == Brows::Hidden {
        return Vec::new();
    }
    let ink = palette::ink().hex();
    let lift = match spec.eyes {
        Eyes::Shine => 11.5,
        Eyes::Oval => 11.0,
        Eyes::Dot | Eyes::Wide => 9.5,
    };
    let mut marks = Vec::new();
    for (side, (x, y)) in [-1.0, 1.0].into_iter().zip(eye_centres(spec)) {
        let by = y - lift;
        // `inner` is the end nearer the nose: a wince raises it.
        let (outer, inner) = ((x + side * 4.5, by), (x - side * 4.5, by));
        let d = match (mood, spec.brows) {
            (Mood::Wince, _) => format!(
                "M{}L{}",
                pt((outer.0, outer.1 + 1.0)),
                pt((inner.0, inner.1 - 2.2))
            ),
            (_, Brows::Straight) => format!(
                "M{}L{}",
                pt((outer.0 - side * 1.0, outer.1 - 0.6)),
                pt((inner.0 + side * 1.0, inner.1 - 0.6))
            ),
            (_, Brows::Soft | Brows::Hidden) => format!(
                "M{}Q{} {}",
                pt((outer.0, outer.1 + 1.2)),
                pt((x, by - 2.6)),
                pt((inner.0, inner.1 + 1.2))
            ),
        };
        marks.push(Mark::stroke(d, ink.clone(), 2.6));
    }
    marks
}

/// The mouth as `mood` draws it.
pub(crate) fn mouth(spec: &PersonaSpec, mood: Mood, detail: Detail) -> Vec<Mark> {
    let head = Head::of(spec.head);
    let drop = match spec.creature {
        Creature::Bear => 0.52,
        Creature::Cat | Creature::Bunny => 0.47,
        Creature::Person | Creature::Blob => 0.44,
    };
    let (mx, my) = head.at(0.0, drop);
    let ink = palette::ink().hex();
    let width = match detail {
        Detail::Fine => 2.8,
        Detail::Coarse => 3.4,
    };
    let line = |d: String| Mark::stroke(d, ink.clone(), width);
    let shape = match mood {
        Mood::Idle => spec.mouth,
        Mood::Attentive => Mouth::Small,
        Mood::Happy => return open_smile((mx, my), spec),
        Mood::Wince => {
            return vec![line(format!(
                "M{}Q{} {}Q{} {}",
                pt((mx - 6.0, my + 1.5)),
                pt((mx - 3.0, my - 1.8)),
                pt((mx, my + 1.0)),
                pt((mx + 3.0, my + 3.8)),
                pt((mx + 6.0, my + 0.5))
            ))];
        }
        Mood::Asleep => return vec![Mark::fill(ellipse((mx, my + 1.0), 2.1, 2.5), ink)],
    };
    match shape {
        Mouth::Smile => vec![line(format!(
            "M{}Q{} {}",
            pt((mx - 6.0, my - 1.5)),
            pt((mx, my + 5.0)),
            pt((mx + 6.0, my - 1.5))
        ))],
        Mouth::Cat => vec![line(format!(
            "M{}Q{} {}Q{} {}",
            pt((mx - 6.0, my - 1.0)),
            pt((mx - 3.0, my + 4.0)),
            pt((mx, my)),
            pt((mx + 3.0, my + 4.0)),
            pt((mx + 6.0, my - 1.0))
        ))],
        Mouth::Small => vec![line(format!(
            "M{}Q{} {}",
            pt((mx - 3.6, my - 0.8)),
            pt((mx, my + 2.8)),
            pt((mx + 3.6, my - 0.8))
        ))],
        Mouth::Grin => vec![Mark::rounded(
            format!(
                "M{}Q{} {}Z",
                pt((mx - 6.5, my - 2.0)),
                pt((mx, my + 9.0)),
                pt((mx + 6.5, my - 2.0))
            ),
            ink,
            1.6,
        )],
    }
}

/// Happy's open smile, with a tongue.
fn open_smile((mx, my): (f64, f64), spec: &PersonaSpec) -> Vec<Mark> {
    vec![
        Mark::rounded(
            format!(
                "M{}Q{} {}Z",
                pt((mx - 8.0, my - 2.5)),
                pt((mx, my + 12.0)),
                pt((mx + 8.0, my - 2.5))
            ),
            palette::ink().hex(),
            1.8,
        ),
        Mark::fill(
            ellipse((mx, my + 3.0), 3.4, 1.7),
            palette::blush(spec.tone).hex(),
        ),
    ]
}

/// What is worn over the face: glasses or a bow.
pub(crate) fn worn(spec: &PersonaSpec) -> Vec<Mark> {
    let head = Head::of(spec.head);
    match spec.accessory {
        Accessory::Glasses => {
            let [left, right] = eye_centres(spec);
            let ink = palette::ink().hex();
            let r = 7.4;
            vec![
                Mark::stroke(circle(left, r), ink.clone(), 2.2),
                Mark::stroke(circle(right, r), ink.clone(), 2.2),
                Mark::stroke(
                    format!(
                        "M{}Q{} {}",
                        pt((left.0 + r, left.1 - 1.0)),
                        pt((head.cx, left.1 - 4.0)),
                        pt((right.0 - r, right.1 - 1.0))
                    ),
                    ink,
                    2.2,
                ),
            ]
        }
        Accessory::Bow => {
            let at = head.at(0.60, -0.80);
            let colour = palette::bow(spec.backdrop).hex();
            let wing = |sign: f64| {
                Mark::rounded(
                    format!(
                        "M{}L{}L{}Z",
                        pt(at),
                        pt((at.0 + sign * 9.0, at.1 - 5.5)),
                        pt((at.0 + sign * 9.0, at.1 + 5.5))
                    ),
                    colour.clone(),
                    3.0,
                )
                .turned(25.0, at)
            };
            vec![
                wing(-1.0),
                wing(1.0),
                Mark::fill(circle(at, 3.0), colour.clone()),
            ]
        }
        Accessory::Plain | Accessory::Freckles => Vec::new(),
    }
}

/// The sleeping `z`, above the head on the right; drawn in `currentColor`.
pub(crate) fn snooze(spec: &PersonaSpec) -> String {
    let head = Head::of(spec.head);
    let (x, y) = (head.cx + 0.72 * head.rx, head.top() + 2.0);
    format!(
        "M{}L{}L{}L{}",
        pt((x, y)),
        pt((x + 7.0, y)),
        pt((x, y + 8.0)),
        pt((x + 7.0, y + 8.0))
    )
}
