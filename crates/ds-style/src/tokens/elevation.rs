//! Shadows (design/03-COLOR.md section 10 and 17.2): the two elevation tokens, the plan's
//! `--shadow-pop` and `--shadow-sheet`, C's `--shadow-drag`, and the one-off literals
//! design/04-COMPONENTS.md O-3 says the table must absorb.
//!
//! Values are design/03-COLOR.md section 10 (`S`) and C's `--shadow-drag` (section 16);
//! `--shadow-pop` and `--shadow-sheet` take section 17.2's proposed values, and the names past
//! the plan's four are proposed.

use crate::appearance::theme::Scheme;
use crate::tokens::elevation_layers::layers_css;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One shadow token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "shadow-", kind = fixed, css = shadow_css)]
pub enum Shadow {
    /// `--shadow-1`: resting raised things.
    #[token(name = "1")]
    One,
    /// `--shadow-2`: hovered rows, the strip, toasts.
    #[token(name = "2")]
    Two,
    /// `--shadow-edge`: the half-pixel edge that replaces a 1 px `--line` border on a filled
    /// control, field or group (design/34-MODERN-LOOK.md section 2.6): a ring, no drop.
    Edge,
    /// `--shadow-pop`: menus, hover cards, popovers: a hairline edge and a soft drop.
    Pop,
    /// `--shadow-sheet`: peek, sheets, the command menu: a hairline edge and a long soft drop.
    Sheet,
    /// `--shadow-drag`: the drag ghost.
    Drag,
    /// `--shadow-current`: the current sidebar item and the pressed tile.
    Current,
    /// `--shadow-pill-inset`: the command pill's highlight.
    PillInset,
    /// `--shadow-window`: an app window, a contact and a wide ambient drop.
    Window,
    /// `--shadow-card`: the card.
    Card,
    /// `--shadow-handle`: the editor handle and the slider thumb.
    Handle,
    /// `--shadow-mark`: a provider mark's ring.
    Mark,
    /// `--shadow-mark-tile`: a provider mark on an account tile.
    MarkTile,
}

/// A shadow as the stylesheet writes it: [`Shadow::Window`] and [`Shadow::Drag`] from their
/// layers (`elevation_layers.rs`), the rest from the literal each keeps because it reads a custom
/// property.
fn shadow_css(token: Shadow, scope: TokenScope) -> CssValue {
    let layers = token.layers(scope.scheme);
    if !layers.is_empty() {
        return CssValue::computed(layers_css(layers));
    }
    CssValue::fixed(match token {
        Shadow::One => match scope.scheme {
            Scheme::Light => {
                "0 var(--hair) 0 rgba(255,255,255,.7) inset,0 1px 2px rgba(26,30,26,.1)"
            }
            Scheme::Dark => "0 var(--hair) 0 rgba(255,255,255,.05) inset,0 2px 4px rgba(0,0,0,.45)",
        },
        Shadow::Two => match scope.scheme {
            Scheme::Light => {
                "0 var(--hair) 0 rgba(255,255,255,.7) inset,0 6px 16px -6px rgba(26,30,26,.3)"
            }
            Scheme::Dark => {
                "0 var(--hair) 0 rgba(255,255,255,.05) inset,0 10px 22px -8px rgba(0,0,0,.7)"
            }
        },
        Shadow::Edge => match scope.scheme {
            Scheme::Light => "0 0 0 var(--hairline) rgba(0,0,0,.12)",
            Scheme::Dark => "0 0 0 var(--hairline) rgba(255,255,255,.12)",
        },
        Shadow::Pop => match scope.scheme {
            Scheme::Light => {
                "0 0 0 var(--hairline) rgba(0,0,0,.12),0 10px 30px -8px rgba(0,0,0,.28)"
            }
            Scheme::Dark => {
                "0 0 0 var(--hairline) rgba(255,255,255,.14),0 10px 30px -8px rgba(0,0,0,.55)"
            }
        },
        Shadow::Sheet => match scope.scheme {
            Scheme::Light => {
                "0 0 0 var(--hairline) rgba(0,0,0,.14),0 24px 60px -12px rgba(0,0,0,.35)"
            }
            Scheme::Dark => {
                "0 0 0 var(--hairline) rgba(255,255,255,.14),0 24px 60px -12px rgba(0,0,0,.6)"
            }
        },
        Shadow::Current => {
            "0 var(--hair) 0 rgba(255,255,255,.4) inset,0 2px 6px -3px rgba(0,0,0,.25)"
        }
        Shadow::PillInset => "0 var(--hair) 0 rgba(255,255,255,.35) inset",
        Shadow::Card => "0 0 0 var(--hair) rgba(0,0,0,.06),0 10px 30px -14px rgba(0,0,0,.45)",
        Shadow::Handle => "0 0 0 var(--hair) rgba(0,0,0,.25),0 3px 8px rgba(0,0,0,.35)",
        Shadow::Mark => "0 0 0 var(--hair) rgba(0,0,0,.08)",
        Shadow::MarkTile => "0 0 0 1.5px var(--f-pill),0 1px 2px rgba(0,0,0,.2)",
        Shadow::Window | Shadow::Drag => "",
    })
}
