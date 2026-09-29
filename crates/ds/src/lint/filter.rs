//! Which CSS `filter` functions the pinned renderers paint, for [`super::Rule::FilterNotPainted`]
//! (FINDINGS "CSS filter"). Blitz hands every function to the backend as one filter node; what
//! the backend does with a node is the table below, measured by
//! `ds-native/tests/css_filter.rs`.

/// A renderer a quire document is painted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    /// `anyrender_vello_cpu`: the headless harness and the shell's `wl_shm` fallback.
    VelloCpu,
    /// `anyrender_vello_hybrid`: the GPU path of windows and shell surfaces.
    VelloHybrid,
}

impl Backend {
    const ALL: [Backend; 2] = [Backend::VelloCpu, Backend::VelloHybrid];

    fn name(self) -> &'static str {
        match self {
            Backend::VelloCpu => "vello_cpu",
            Backend::VelloHybrid => "vello_hybrid",
        }
    }

    /// Whether `function`, at `position` in the filter list, is painted.
    ///
    /// vello_cpu is pinned with `multithreading`, whose dispatcher has no filter support, so it
    /// is handed none. vello_hybrid paints a Gaussian blur and a drop shadow and nothing else
    /// (no colour matrix or component transfer), and anyrender hands the backend only the first
    /// node of a list.
    fn paints(self, function: &str, position: usize) -> bool {
        match self {
            Backend::VelloCpu => false,
            Backend::VelloHybrid => position == 0 && matches!(function, "blur" | "drop-shadow"),
        }
    }
}

/// The names of the functions written at the top level of a `filter` value, in order:
/// `blur(2px) contrast(1.2)` gives `blur`, `contrast`. `none` and words without arguments are
/// not functions.
fn functions(value: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut depth = 0_u32;
    let mut word_start = 0;
    for (at, c) in value.char_indices() {
        match c {
            '(' => {
                if depth == 0 {
                    names.push(value[word_start..at].trim());
                }
                depth += 1;
            }
            ')' => depth = depth.saturating_sub(1),
            c if c.is_whitespace() && depth == 0 => word_start = at + c.len_utf8(),
            _ => {}
        }
    }
    names
}

/// Why part of `value` (a `filter` declaration's value) will not paint, or `None` when all of it
/// does. Names each function that is dropped and on which backend.
pub fn not_painted(value: &str) -> Option<String> {
    let value = value.to_ascii_lowercase();
    let names = functions(&value);
    let gaps: Vec<String> = Backend::ALL
        .into_iter()
        .filter_map(|backend| {
            let dropped: Vec<String> = names
                .iter()
                .enumerate()
                // `var()` is whatever the token holds; it cannot be judged here.
                .filter(|(_, name)| **name != "var")
                .filter(|(position, name)| !backend.paints(name, *position))
                .map(|(_, name)| format!("{name}()"))
                .collect();
            (!dropped.is_empty()).then(|| format!("{} on {}", dropped.join(", "), backend.name()))
        })
        .collect();
    (!gaps.is_empty()).then(|| {
        format!(
            "not painted: {}; only blur() and drop-shadow(), first in the list, paint on \
             vello_hybrid, and vello_cpu paints none (FINDINGS \"CSS filter\")",
            gaps.join("; ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{functions, not_painted};

    const FUNCTION_CASES: &[(&str, &str, &[&str])] = &[
        ("one function", "blur(4px)", &["blur"]),
        (
            "a list keeps its order",
            "blur(4px) contrast(1.5)",
            &["blur", "contrast"],
        ),
        (
            "nested parentheses stay inside their function",
            "drop-shadow(0 0 4px rgb(0 0 0 / 50%)) blur(2px)",
            &["drop-shadow", "blur"],
        ),
        ("none is no function", "none", &[]),
        ("a var() is one function", "var(--soft)", &["var"]),
    ];

    #[test]
    fn functions_lists_the_top_level_calls() {
        for (name, value, expected) in FUNCTION_CASES {
            assert_eq!(&functions(value), expected, "{name}");
        }
    }

    /// A case: what the message must name, what it must not, or `None` when all paints.
    struct Case {
        name: &'static str,
        value: &'static str,
        message: Option<(&'static [&'static str], &'static [&'static str])>,
    }

    const CASES: &[Case] = &[
        Case {
            name: "blur is dropped on vello_cpu only",
            value: "blur(4px)",
            message: Some((
                &["blur() on vello_cpu"],
                &["on vello_hybrid;", "() on vello_hybrid"],
            )),
        },
        Case {
            name: "drop-shadow is dropped on vello_cpu only",
            value: "drop-shadow(0 2px 4px black)",
            message: Some((&["drop-shadow() on vello_cpu"], &["() on vello_hybrid"])),
        },
        Case {
            name: "saturate is dropped on both backends",
            value: "saturate(1.8)",
            message: Some((
                &["saturate() on vello_cpu", "saturate() on vello_hybrid"],
                &[],
            )),
        },
        Case {
            name: "a colour function after blur is dropped on hybrid, blur is not",
            value: "blur(4px) contrast(1.5)",
            message: Some((
                &[
                    "blur(), contrast() on vello_cpu",
                    "; contrast() on vello_hybrid",
                ],
                &[
                    "blur() on vello_hybrid",
                    "blur(), contrast() on vello_hybrid",
                ],
            )),
        },
        Case {
            name: "blur after a colour function is dropped on hybrid too",
            value: "contrast(1.5) blur(4px)",
            message: Some((
                &[
                    "contrast(), blur() on vello_cpu",
                    "contrast(), blur() on vello_hybrid",
                ],
                &[],
            )),
        },
        Case {
            name: "none paints",
            value: "none",
            message: None,
        },
        Case {
            name: "a token is not judged",
            value: "var(--soft)",
            message: None,
        },
        Case {
            name: "upper case is read",
            value: "SEPIA(1)",
            message: Some((&["sepia() on vello_hybrid"], &[])),
        },
    ];

    #[test]
    fn each_value_names_the_functions_that_do_not_paint() {
        for case in CASES {
            match (not_painted(case.value), case.message) {
                (None, None) => {}
                (Some(got), Some((present, absent))) => {
                    for fragment in present {
                        assert!(got.contains(fragment), "{}: {got}", case.name);
                    }
                    for fragment in absent {
                        assert!(!got.contains(fragment), "{}: {got}", case.name);
                    }
                }
                (got, expected) => panic!("{}: got {got:?}, expected {expected:?}", case.name),
            }
        }
    }
}
