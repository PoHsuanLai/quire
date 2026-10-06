//! How long a shape's stroke is, as a length that is never short, for drawing it on
//! (`stroke-dasharray` and `stroke-dashoffset`): a Bezier is measured by its control polygon and
//! an arc by its own radius and chord, both at least the curve's own length, so a dash of
//! this length always covers the whole stroke.

use super::shape::Shape;
use std::f32::consts::PI;

/// The stroke length of `shape` on the 24 grid, at least its true length.
pub(crate) fn stroke_length(shape: &Shape) -> f32 {
    match shape {
        Shape::Path(d) | Shape::Solid(d) => path_length(d),
        Shape::Circle { r, .. } => 2.0 * PI * number(r),
        Shape::Rect { width, height, .. } => 2.0 * (number(width) + number(height)),
    }
}

fn number(text: &str) -> f32 {
    text.parse().unwrap_or(0.0)
}

/// One path data token.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Command(char),
    Value(f32),
}

fn tokens(d: &str) -> Vec<Token> {
    let chars: Vec<char> = d.chars().collect();
    let mut out = Vec::new();
    let (mut command, mut position, mut index) = (' ', 0usize, 0usize);
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_ascii_alphabetic() {
            out.push(Token::Command(ch));
            (command, position) = (ch, 0);
            index += 1;
        } else if ch == '-' || ch == '.' || ch.is_ascii_digit() {
            // An arc's two flags are one character each and may run into the next number.
            let flag = command.eq_ignore_ascii_case(&'a') && matches!(position % 7, 3 | 4);
            let end = if flag {
                index + 1
            } else {
                number_end(&chars, index)
            };
            let text: String = chars[index..end].iter().collect();
            out.extend(text.parse().ok().map(Token::Value));
            position += 1;
            index = end;
        } else {
            index += 1;
        }
    }
    out
}

/// Where the number starting at `from` ends: a sign, digits and at most one point.
fn number_end(chars: &[char], from: usize) -> usize {
    let mut end = from + usize::from(chars[from] == '-');
    let mut point = false;
    while let Some(&ch) = chars.get(end) {
        match ch {
            '0'..='9' => {}
            '.' if !point => point = true,
            _ => break,
        }
        end += 1;
    }
    end.max(from + 1)
}

/// How many values one repeat of `command` takes.
fn arity(command: char) -> usize {
    match command.to_ascii_lowercase() {
        'h' | 'v' => 1,
        'm' | 'l' | 't' => 2,
        's' | 'q' => 4,
        'c' => 6,
        'a' => 7,
        _ => 0,
    }
}

/// An arc of radius `radius` across `chord`: its true length, or half a circle on the chord when
/// the radius is too short to reach (the radius then grows to the chord's half).
fn arc_length(chord: f32, radius: f32, large: bool) -> f32 {
    if chord <= f32::EPSILON {
        return 0.0;
    }
    if chord >= 2.0 * radius {
        return chord * PI / 2.0;
    }
    let small = 2.0 * (chord / (2.0 * radius)).asin();
    radius * if large { 2.0 * PI - small } else { small }
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// The length of path data `d`: the sum of its segments' bounds.
fn path_length(d: &str) -> f32 {
    let tokens = tokens(d);
    let (mut at, mut start, mut total) = ((0.0, 0.0), (0.0, 0.0), 0.0);
    let mut index = 0;
    while index < tokens.len() {
        let Token::Command(command) = tokens[index] else {
            index += 1;
            continue;
        };
        index += 1;
        let (n, relative) = (arity(command), command.is_ascii_lowercase());
        if n == 0 {
            total += distance(at, start);
            at = start;
            continue;
        }
        let mut first = true;
        loop {
            let args: Vec<f32> = tokens[index..]
                .iter()
                .map_while(|token| match token {
                    Token::Value(value) => Some(*value),
                    Token::Command(_) => None,
                })
                .take(n)
                .collect();
            if args.len() < n {
                break;
            }
            index += n;
            let base = if relative { at } else { (0.0, 0.0) };
            let point = |x: f32, y: f32| (base.0 + x, base.1 + y);
            let end = match command.to_ascii_lowercase() {
                'h' => (if relative { at.0 } else { 0.0 } + args[0], at.1),
                'v' => (at.0, if relative { at.1 } else { 0.0 } + args[0]),
                'a' => point(args[5], args[6]),
                _ => point(args[n - 2], args[n - 1]),
            };
            let moved = command.eq_ignore_ascii_case(&'m') && first;
            total += match command.to_ascii_lowercase() {
                _ if moved => 0.0,
                'c' | 'q' | 's' => {
                    let mut length = 0.0;
                    let mut last = at;
                    for pair in args.chunks(2) {
                        let next = point(pair[0], pair[1]);
                        length += distance(last, next);
                        last = next;
                    }
                    length
                }
                'a' => arc_length(
                    distance(at, end),
                    args[0].abs().max(args[1].abs()),
                    args[3] != 0.0,
                ),
                _ => distance(at, end),
            };
            if moved {
                start = end;
            }
            first = false;
            at = end;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{Shape, stroke_length};
    use crate::icon::Icon;

    #[test]
    fn a_measure_is_never_short_of_the_stroke() {
        // (path, the true length, at most this much over it as a share).
        const CASES: &[(&str, f32, f32)] = &[
            ("M3 6h18", 18.0, 0.0),
            ("M10 11v6", 6.0, 0.0),
            ("M2 2l20 20", 28.284, 0.001),
            ("M16 9a5 5 0 0 1 0 6", 6.435, 0.01),
            (
                "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8",
                24.79,
                0.02,
            ),
        ];
        for &(d, truth, over) in CASES {
            let got = stroke_length(&Shape::Path(d));
            assert!(got >= truth - 0.01, "{d}: {got} < {truth}");
            assert!(got <= truth * (1.0 + over) + 0.01, "{d}: {got} too long");
        }
    }

    #[test]
    fn a_circle_and_a_rect_are_exact() {
        let circle = Shape::Circle {
            cx: "12",
            cy: "12",
            r: "10",
        };
        assert!((stroke_length(&circle) - 62.83).abs() < 0.01);
        let rect = Shape::Rect {
            x: "2",
            y: "4",
            width: "20",
            height: "16",
            rx: "2",
        };
        assert_eq!(stroke_length(&rect), 72.0);
    }

    #[test]
    fn every_icon_shape_has_a_length() {
        for icon in Icon::ALL {
            for shape in icon.shapes() {
                assert!(stroke_length(shape) > 0.0, "{icon:?}: {shape:?}");
            }
        }
    }
}
