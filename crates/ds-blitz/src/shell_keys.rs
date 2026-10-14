//! The keys blitz-shell answers itself, and the page zoom an app asks for instead.
//!
//! blitz-shell zooms the page on Ctrl or Super with `=`, `-` and `0`, and switches its layout
//! overlay, hover highlight and layout dump on Alt with `D`, `H` and `T`, before the document
//! hears the key and whatever else is held. An app whose keys those are (a terminal: Alt+D,
//! Alt+T and Ctrl+_ are the program's) launches with [`ShellKeys::App`]: every key is the
//! document's, and the app binds the standard Bigger, Smaller and Actual Size itself and asks
//! `WindowHost::page_zoom`, which [`zoomed`] steps.

use ds::window::vocab::PageZoom;

/// Who answers the keys blitz-shell has its own meaning for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShellKeys {
    /// blitz-shell does, before the document hears them: its page zoom and its overlays.
    #[default]
    Blitz,
    /// The app does: every key goes to the document only.
    App,
}

impl ShellKeys {
    /// blitz-shell's switch for this choice.
    pub(crate) fn builtin(self) -> blitz_shell::BuiltinKeys {
        match self {
            ShellKeys::Blitz => blitz_shell::BuiltinKeys::On,
            ShellKeys::App => blitz_shell::BuiltinKeys::Off,
        }
    }
}

/// The zooms a step goes between, as a browser's: from half size to three times.
const LADDER: [f32; 12] = [
    0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 3.0,
];

/// The zoom after `step` from `zoom`: the next rung up or down the ladder (a zoom between rungs
/// goes to the nearest one past it), held at either end, or 1 for Actual Size.
pub(crate) fn zoomed(zoom: f32, step: PageZoom) -> f32 {
    const EPS: f32 = 0.001;
    match step {
        PageZoom::ActualSize => 1.0,
        PageZoom::Bigger => LADDER
            .iter()
            .copied()
            .find(|rung| *rung > zoom + EPS)
            .unwrap_or(LADDER[LADDER.len() - 1]),
        PageZoom::Smaller => LADDER
            .iter()
            .rev()
            .copied()
            .find(|rung| *rung < zoom - EPS)
            .unwrap_or(LADDER[0]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_climbs_the_ladder_and_holds_at_its_ends() {
        // (from, step, to)
        let cases: &[(f32, PageZoom, f32)] = &[
            (1.0, PageZoom::Bigger, 1.1),
            (1.0, PageZoom::Smaller, 0.9),
            (1.1, PageZoom::Smaller, 1.0),
            (1.3, PageZoom::Bigger, 1.5),
            (1.3, PageZoom::Smaller, 1.25),
            (3.0, PageZoom::Bigger, 3.0),
            (0.5, PageZoom::Smaller, 0.5),
            (2.0, PageZoom::ActualSize, 1.0),
        ];
        for (from, step, to) in cases {
            assert_eq!(zoomed(*from, *step), *to, "{from} {step:?}");
        }
    }
}
