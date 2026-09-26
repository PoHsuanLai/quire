//! The still layer of a persona (design/24-PERSONA.md section 3.1): ears and the hair behind
//! the head, the head, the muzzle and nose, cheeks, and the hair in front, back to front.

use super::geometry::{Head, circle, ellipse, polygon, pt};
use super::mark::Mark;
use super::mood::Detail;
use super::palette;
use super::spec::{Accessory, Cheeks, Creature, PersonaSpec, Top};

/// Every still mark, back to front.
pub(crate) fn still(spec: &PersonaSpec, detail: Detail) -> Vec<Mark> {
    let head = Head::of(spec.head);
    let skin = palette::tone(spec.tone).hex();
    let mut marks = ears(spec, head);
    marks.extend(hair_behind(spec, head));
    marks.push(Mark::fill(head.outline(0.0), skin));
    marks.extend(muzzle(spec, head, detail));
    marks.extend(cheeks(spec, head, detail));
    marks.extend(hair_front(spec, head));
    marks
}

/// Ears behind the head.
fn ears(spec: &PersonaSpec, head: Head) -> Vec<Mark> {
    let fur = palette::tone(spec.tone).hex();
    let inner = palette::soft(spec.tone).hex();
    let mut marks = Vec::new();
    for side in [-1.0, 1.0] {
        match spec.creature {
            Creature::Bear => {
                let at = head.at(side * 0.70, -0.80);
                marks.push(Mark::fill(circle(at, 10.5), fur.clone()));
                marks.push(Mark::fill(circle(at, 5.5), inner.clone()));
            }
            Creature::Cat => {
                let base_out = head.at(side * 0.94, -0.36);
                let base_in = head.at(side * 0.30, -0.93);
                let tip = head.at(side * 0.84, -1.0);
                let tip = (tip.0, tip.1 - 11.0);
                marks.push(Mark::rounded(
                    polygon(&[base_out, tip, base_in]),
                    fur.clone(),
                    5.0,
                ));
                let inner_tri = [
                    lerp(base_out, tip, 0.35, base_in, 0.2),
                    lerp(tip, base_out, 0.2, base_in, 0.2),
                    lerp(base_in, tip, 0.35, base_out, 0.2),
                ];
                marks.push(Mark::rounded(polygon(&inner_tri), inner.clone(), 3.0));
            }
            Creature::Bunny => {
                let at = (head.cx + side * 11.0, head.top() - 5.0);
                let pivot = (at.0, at.1 + 14.0);
                let lean = side * 9.0;
                marks.push(Mark::fill(ellipse(at, 7.5, 16.0), fur.clone()).turned(lean, pivot));
                marks.push(
                    Mark::fill(ellipse((at.0, at.1 + 1.5), 3.6, 11.0), inner.clone())
                        .turned(lean, pivot),
                );
            }
            Creature::Person | Creature::Blob => {}
        }
    }
    marks
}

/// A point `a`, pulled towards `b` by `share_b` and towards `c` by `share_c`.
fn lerp(a: (f64, f64), b: (f64, f64), share_b: f64, c: (f64, f64), share_c: f64) -> (f64, f64) {
    (
        a.0 + (b.0 - a.0) * share_b + (c.0 - a.0) * share_c,
        a.1 + (b.1 - a.1) * share_b + (c.1 - a.1) * share_c,
    )
}

/// Hair behind the head: a bob's length, a bun.
fn hair_behind(spec: &PersonaSpec, head: Head) -> Vec<Mark> {
    let hair = palette::hair(spec.hair).hex();
    match top(spec) {
        Top::Bob => {
            let (left, right) = (head.cx - head.rx - 4.5, head.cx + head.rx + 4.5);
            let (upper, lower) = (head.top() - 3.5, head.cy + 0.55 * head.ry);
            let d = format!(
                "M{}L{}Q{} {}L{}Q{} {}L{}Q{} {}Z",
                pt((left, lower)),
                pt((left, head.cy - 0.2 * head.ry)),
                pt((left, upper)),
                pt((head.cx, upper)),
                pt((head.cx, upper)),
                pt((right, upper)),
                pt((right, head.cy - 0.2 * head.ry)),
                pt((right, lower)),
                pt((head.cx, lower + 3.0)),
                pt((left, lower)),
            );
            vec![Mark::rounded(d, hair, 4.0)]
        }
        Top::Bun => vec![Mark::fill(circle((head.cx, head.top() - 4.5), 10.0), hair)],
        Top::Bare | Top::Tuft | Top::Crop | Top::Fringe | Top::Curly => Vec::new(),
    }
}

/// The top as drawn: a blob always wears its sprout.
fn top(spec: &PersonaSpec) -> Top {
    match spec.creature {
        Creature::Blob => Top::Tuft,
        Creature::Person | Creature::Bear | Creature::Cat | Creature::Bunny => spec.top,
    }
}

/// Hair over the head.
fn hair_front(spec: &PersonaSpec, head: Head) -> Vec<Mark> {
    let hair = palette::hair(spec.hair).hex();
    match top(spec) {
        Top::Bare => Vec::new(),
        Top::Tuft => vec![tuft(head, hair)],
        Top::Crop | Top::Bun => vec![cap(head, head.cy - 0.40 * head.ry, -5.0, hair)],
        Top::Fringe => vec![fringe(head, hair)],
        Top::Bob => vec![swept(head, hair)],
        Top::Curly => curls(head, hair),
    }
}

/// A single curl on the crown.
fn tuft(head: Head, colour: String) -> Mark {
    let (cx, top) = (head.cx, head.top());
    let d = format!(
        "M{}C{} {} {}C{} {} {}Z",
        pt((cx - 4.0, top + 4.0)),
        pt((cx - 7.0, top - 8.0)),
        pt((cx + 5.0, top - 14.0)),
        pt((cx + 10.0, top - 9.0)),
        pt((cx + 4.0, top - 7.5)),
        pt((cx + 1.0, top - 2.0)),
        pt((cx + 3.5, top + 4.0)),
    );
    Mark::rounded(d, colour, 1.5)
}

/// The crown 2 units out above `cut`, closed by a curve that dips `sag` below it (a negative
/// sag lifts the hairline in the middle, as short hair does).
fn cap(head: Head, cut: f64, sag: f64, colour: String) -> Mark {
    let crown = head.crown(cut, 2.0);
    let first = crown[0];
    let mut d = crown_path(&crown);
    d.push_str(&format!("Q{} {}Z", pt((head.cx, cut + sag)), pt(first)));
    Mark::rounded(d, colour, 2.0)
}

/// Bangs: a cap to just above the eyes, its edge three rounded locks.
fn fringe(head: Head, colour: String) -> Mark {
    let cut = head.cy - 0.12 * head.ry;
    let crown = head.crown(cut, 2.0);
    let (left, right) = (crown[0], crown[crown.len() - 1]);
    let mut d = crown_path(&crown);
    let locks = 3.0;
    let lift = 7.0;
    for lock in 0..3 {
        let from = f64::from(lock);
        let x0 = right.0 + (left.0 - right.0) * from / locks;
        let x1 = right.0 + (left.0 - right.0) * (from + 1.0) / locks;
        let y1 = if lock == 2 { left.1 } else { cut - lift };
        d.push_str(&format!(
            "Q{} {}",
            pt(((x0 + x1) / 2.0, cut + 4.0)),
            pt((x1, y1))
        ));
    }
    d.push('Z');
    Mark::rounded(d, colour, 2.0)
}

/// A fringe swept to one side, over a bob.
fn swept(head: Head, colour: String) -> Mark {
    let cut = head.cy - 0.05 * head.ry;
    let crown = head.crown(cut, 2.0);
    let left = crown[0];
    let mut d = crown_path(&crown);
    d.push_str(&format!(
        "Q{} {}Q{} {}Z",
        pt(head.at(0.35, -0.55)),
        pt(head.at(-0.15, -0.42)),
        pt(head.at(-0.62, -0.30)),
        pt(left)
    ));
    Mark::rounded(d, colour, 2.0)
}

/// Curls: a short cap edged with round curls over the crown.
fn curls(head: Head, colour: String) -> Vec<Mark> {
    let cut = head.cy - 0.30 * head.ry;
    let mut marks = vec![cap(head, cut, 3.0, colour.clone())];
    let (t0, t1) = head.span_above(cut + 3.0, 0.0);
    for step in 0..7 {
        let t = t0 + (t1 - t0) * f64::from(step) / 6.0;
        marks.push(Mark::fill(circle(head.point(t, 1.0), 6.8), colour.clone()));
    }
    marks
}

/// The crown's points as an open path, left to right.
fn crown_path(points: &[(f64, f64)]) -> String {
    let mut d = polygon(points);
    d.pop();
    d
}

/// A muzzle and nose for a bear; a nose for a cat or a rabbit.
fn muzzle(spec: &PersonaSpec, head: Head, detail: Detail) -> Vec<Mark> {
    let ink = palette::ink().hex();
    let nose = head.at(0.0, 0.30);
    match spec.creature {
        Creature::Bear => vec![
            Mark::fill(
                ellipse(head.at(0.0, 0.40), 0.36 * head.rx, 0.26 * head.ry),
                palette::soft(spec.tone).hex(),
            ),
            Mark::fill(ellipse(nose, 3.6, 2.5), ink),
        ],
        Creature::Cat => {
            let mut marks = vec![Mark::rounded(
                polygon(&[
                    (nose.0 - 3.0, nose.1 - 1.5),
                    (nose.0 + 3.0, nose.1 - 1.5),
                    (nose.0, nose.1 + 1.8),
                ]),
                ink.clone(),
                1.6,
            )];
            if detail == Detail::Fine {
                marks.extend(whiskers(head, ink));
            }
            marks
        }
        Creature::Bunny => vec![Mark::fill(
            ellipse(nose, 2.8, 2.0),
            palette::blush(spec.tone).hex(),
        )],
        Creature::Person | Creature::Blob => Vec::new(),
    }
}

/// Two whiskers each side, reaching past the cheek.
fn whiskers(head: Head, ink: String) -> Vec<Mark> {
    let mut marks = Vec::new();
    for side in [-1.0, 1.0] {
        for (from_dy, to_dy) in [(0.46, 0.40), (0.54, 0.62)] {
            let from = head.at(side * 0.66, from_dy);
            let to = (head.cx + side * (head.rx + 5.0), head.cy + to_dy * head.ry);
            marks.push(Mark::stroke(
                format!("M{}L{}", pt(from), pt(to)),
                ink.clone(),
                1.5,
            ));
        }
    }
    marks
}

/// Blush and freckles.
fn cheeks(spec: &PersonaSpec, head: Head, detail: Detail) -> Vec<Mark> {
    let mut marks = Vec::new();
    for side in [-1.0, 1.0] {
        let at = head.at(side * 0.62, 0.34);
        if spec.cheeks == Cheeks::Blush {
            marks.push(
                Mark::fill(ellipse(at, 5.2, 3.3), palette::blush(spec.tone).hex()).faded("0.85"),
            );
        }
        if spec.accessory == Accessory::Freckles && detail == Detail::Fine {
            let deep = palette::deep(spec.tone).hex();
            for (dx, dy) in [(-2.4, -3.6), (0.6, -4.6), (2.8, -3.2)] {
                marks.push(Mark::fill(
                    circle((at.0 + dx, at.1 + dy), 0.95),
                    deep.clone(),
                ));
            }
        }
    }
    marks
}
