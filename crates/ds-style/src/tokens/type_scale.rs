//! The five face jobs and the size ramp (design/02-TYPE.md sections 2 and 4).
//!
//! The plan names the ends, `--fs-micro` 9.5 and `--fs-display` 26; the steps between are named
//! here by role (design/02-TYPE.md open decision 3), one per distinct size in the ramp.

use crate::appearance::typeface::Typeface;
use crate::tokens::token::{Token, TokenScope};
use ds_core::word::Word;

/// A type family, by job. Which face does each job depends on the root's [`Typeface`]
/// (design/02-TYPE.md section 2): the `.ds` block names the System faces and the
/// `.ds[data-typeface=editorial]` block the Editorial ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "font-", kind = fixed)]
pub enum Family {
    /// `--font-display`: Inter Display (System) or Bricolage Grotesque (Editorial). Headings,
    /// names, initials, big numbers.
    #[token(
        system = "\"Inter Display\",\"Inter\",system-ui,sans-serif",
        editorial = "\"Bricolage Grotesque\",\"Inter\",\"Trebuchet MS\",system-ui,sans-serif"
    )]
    Display,
    /// `--font-ui`: Inter (System) or Karla (Editorial). Body text and every control.
    #[token(
        system = "\"Inter\",system-ui,sans-serif",
        editorial = "\"Karla\",\"Inter\",\"Segoe UI\",system-ui,sans-serif"
    )]
    Ui,
    /// `--font-data`: Inter, always tabular (System), or Space Mono (Editorial). Anything
    /// machine-shaped: times, counts, chips, eyebrows, section headers.
    #[token(
        system = "\"Inter\",system-ui,sans-serif",
        editorial = "\"Space Mono\",\"Inter\",ui-monospace,\"SFMono-Regular\",Menlo,monospace"
    )]
    Data,
    /// `--font-serif`: Noto Serif. A message a person writes in a serif, and the control that
    /// offers it; never the interface's own text.
    #[token(value = "\"Noto Serif\",Georgia,\"Times New Roman\",serif")]
    Serif,
    /// `--font-code`: Space Mono in either typeface. Only where a fixed pitch carries meaning:
    /// code, `KeyEquivalent` caps, aligned logs.
    #[token(value = "\"Space Mono\",\"Inter\",ui-monospace,\"SFMono-Regular\",Menlo,monospace")]
    Code,
}

impl Family {
    /// The family name the stack leads with under `typeface`: the name its face registers as.
    pub fn face_name(self, typeface: Typeface) -> String {
        let value = self.css_value(TokenScope::BASE.in_typeface(typeface));
        let stack = value.as_str();
        stack
            .strip_prefix('"')
            .and_then(|rest| rest.split('"').next())
            .unwrap_or(stack)
            .to_owned()
    }
}

/// One step of the size ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "fs-", kind = fixed)]
pub enum FontSize {
    /// `--fs-pico` 7.5: in-row provider mark.
    #[token(value = "7.5px")]
    Pico,
    /// `--fs-dial` 9 (10 under System, the floor): a medium world clock dial's numerals
    /// (design/23-WIDGETS.md section 4.2).
    #[token(system = "10px", editorial = "9px")]
    Dial,
    /// `--fs-nano` 9 (10 under System): pin count, favicon letter.
    #[token(system = "10px", editorial = "9px")]
    Nano,
    /// `--fs-micro` 9.5 (10 under System): chip, via, group header.
    #[token(system = "10px", editorial = "9.5px")]
    Micro,
    /// `--fs-caption` 10: row time, section header, shortcut.
    #[token(value = "10px")]
    Caption,
    /// `--fs-note` 10.5: kbd, count, hover-card sub.
    #[token(value = "10.5px")]
    Note,
    /// `--fs-eyebrow` 11: eyebrow, link pill.
    #[token(value = "11px")]
    Eyebrow,
    /// `--fs-help` 11.5: menu help, command snippet, view switch.
    #[token(value = "11.5px")]
    Help,
    /// `--fs-small` 12: mini, segmented, tooltip.
    #[token(value = "12px")]
    Small,
    /// `--fs-meta` 12.5: snippet, toast, chip person.
    #[token(value = "12.5px")]
    Meta,
    /// `--fs-control` 13: command pill, menu item, button.
    #[token(value = "13px")]
    Control,
    /// `--fs-body` 13.5: row name and subject, sidebar item, input.
    #[token(value = "13.5px")]
    Body,
    /// `--fs-reading` 14: reader body, hover-card title.
    #[token(value = "14px")]
    Reading,
    /// `--fs-compose` 14.5: composer body.
    #[token(value = "14.5px")]
    Compose,
    /// `--fs-base` 15: the base text.
    #[token(value = "15px")]
    Base,
    /// `--fs-subhead` 15.5: parsed-body subheading.
    #[token(value = "15.5px")]
    Subhead,
    /// `--fs-title` 16: list title, command input.
    #[token(value = "16px")]
    Title,
    /// `--fs-title-2` 17: macOS Title 2 (design/34-MODERN-LOOK.md section 2.5), the alert title
    /// at 700. A new step: `--fs-title` stays 16 so no consumer moves.
    #[token(name = "title-2", value = "17px")]
    Title2,
    /// `--fs-heading-3` 16.5: composer `h3`.
    #[token(name = "heading-3", value = "16.5px")]
    Heading3,
    /// `--fs-dial-large` 18: a small clock widget's dial numerals (design/23 section 4.2).
    #[token(system = "16.5px", editorial = "18px")]
    DialLarge,
    /// `--fs-widget-figure` 20: a battery's percentage under its ring (design/23 section 4.1).
    #[token(system = "18px", editorial = "20px")]
    WidgetFigure,
    /// `--fs-subject` 20: reader subject.
    #[token(value = "20px")]
    Subject,
    /// `--fs-heading` 21: parsed-body and composer headings.
    #[token(value = "21px")]
    Heading,
    /// `--fs-amount` 22: receipt amount.
    #[token(value = "22px")]
    Amount,
    /// `--fs-day` 24: event day number.
    #[token(value = "24px")]
    Day,
    /// `--fs-display` 26: composer subject.
    #[token(value = "26px")]
    Display,
    /// `--fs-emoji-cell` 30: an emoji in an emoji grid's 56 px cell.
    #[token(value = "30px")]
    EmojiCell,
    /// `--fs-widget-hero` 47: a widget's hero value, a small battery's percentage (design/23-WIDGETS.md section 3.2).
    #[token(system = "42.5px", editorial = "47px")]
    WidgetHero,
    /// `--fs-emoji-preview` 96: one emoji in a preview pane.
    #[token(value = "96px")]
    EmojiPreview,
    /// `--fs-lock-clock` 140: the lock screen's time, the largest type the shell draws
    /// (design/20-SURFACES.md section 1.9; design/04-COMPONENTS.md section 42).
    #[token(system = "127px", editorial = "140px")]
    LockClock,
}

impl FontSize {
    /// The smallest UI text size under System, in px: the Mac's 10 pt floor (design/27-HIG-PARITY.md
    /// section 3.16, `--fs-min`). The lint `MinFontSize` rejects a size below it.
    pub const MIN_PX: f32 = 10.0;

    /// The steps the Editorial ramp draws below [`Self::MIN_PX`] and System raises to it
    /// (design/27 section 7, H0). `Dial` is also cap-fitted; the floor wins.
    pub const FLOORED: [FontSize; 3] = [FontSize::Dial, FontSize::Nano, FontSize::Micro];

    /// The steps fitted to a measured cap height, whose size follows the typeface.
    pub const CAP_FITTED: [FontSize; 5] = [
        FontSize::Dial,
        FontSize::DialLarge,
        FontSize::WidgetFigure,
        FontSize::WidgetHero,
        FontSize::LockClock,
    ];

    /// The size as CSS under `typeface`: `13px`.
    pub fn css_in(self, typeface: Typeface) -> String {
        self.css_value(TokenScope::BASE.in_typeface(typeface))
            .to_string()
    }

    /// The size in px under `typeface`, for a check against [`Self::MIN_PX`].
    pub fn px_in(self, typeface: Typeface) -> f32 {
        self.css_in(typeface)
            .trim_end_matches("px")
            .parse()
            .unwrap_or(f32::NAN)
    }
}
