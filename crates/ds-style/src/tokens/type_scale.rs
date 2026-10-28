//! The five face jobs and the size ramp (design/02-TYPE.md sections 2 and 4).
//!
//! The plan names the ends, `--fs-micro` 9.5 and `--fs-display` 26; the steps between are named
//! here by role (design/02-TYPE.md open decision 3), one per distinct size in the ramp.

use crate::tokens::token::{Token, TokenScope};
use ds_core::word::Word;

/// A type family, by job (design/02-TYPE.md section 2). A look may name other faces (the
/// Editorial look Extension names mail's Bricolage Grotesque, Karla and Space Mono).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "font-", kind = fixed)]
pub enum Family {
    /// `--font-display`: Inter Display. Headings,
    /// names, initials, big numbers.
    #[token(value = "\"Inter Display\",\"Inter\",system-ui,sans-serif")]
    Display,
    /// `--font-ui`: Inter. Body text and every control.
    #[token(value = "\"Inter\",system-ui,sans-serif")]
    Ui,
    /// `--font-data`: Inter, always tabular. Anything
    /// machine-shaped: times, counts, chips, eyebrows, section headers.
    #[token(value = "\"Inter\",system-ui,sans-serif")]
    Data,
    /// `--font-serif`: Noto Serif. A message a person writes in a serif, and the control that
    /// offers it; never the interface's own text.
    #[token(value = "\"Noto Serif\",Georgia,\"Times New Roman\",serif")]
    Serif,
    /// `--font-code`: Space Mono. Only where a fixed pitch carries meaning:
    /// code, `KeyEquivalent` caps, aligned logs.
    #[token(value = "\"Space Mono\",\"Inter\",ui-monospace,\"SFMono-Regular\",Menlo,monospace")]
    Code,
}

impl Family {
    /// The family name the stack leads with: the name its face registers as.
    pub fn face_name(self) -> String {
        let value = self.css_value(TokenScope::BASE);
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
    /// `--fs-dial` 10 (the floor): a medium world clock dial's numerals
    /// (design/23-WIDGETS.md section 4.2).
    #[token(value = "10px")]
    Dial,
    /// `--fs-nano` 10: pin count, favicon letter.
    #[token(value = "10px")]
    Nano,
    /// `--fs-micro` 10: chip, via, group header.
    #[token(value = "10px")]
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
    /// `--fs-dial-large` 16.5: a small clock widget's dial numerals (design/23 section 4.2).
    #[token(value = "16.5px")]
    DialLarge,
    /// `--fs-widget-figure` 18: a battery's percentage under its ring (design/23 section 4.1).
    #[token(value = "18px")]
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
    /// `--fs-widget-hero` 42.5: a widget's hero value, a small battery's percentage (design/23-WIDGETS.md section 3.2).
    #[token(value = "42.5px")]
    WidgetHero,
    /// `--fs-emoji-preview` 96: one emoji in a preview pane.
    #[token(value = "96px")]
    EmojiPreview,
    /// `--fs-lock-clock` 127: the lock screen's time, the largest type the shell draws
    /// (design/20-SURFACES.md section 1.9; design/04-COMPONENTS.md section 42).
    #[token(value = "127px")]
    LockClock,
}

impl FontSize {
    /// The smallest UI text size, in px: the Mac's 10 pt floor (design/27-HIG-PARITY.md
    /// section 3.16, `--fs-min`). The lint `MinFontSize` rejects a size below it.
    pub const MIN_PX: f32 = 10.0;

    /// The size as CSS: `13px`.
    pub fn css(self) -> String {
        self.css_value(TokenScope::BASE).to_string()
    }

    /// The size in px, for a check against [`Self::MIN_PX`].
    pub fn px(self) -> f32 {
        self.css()
            .trim_end_matches("px")
            .parse()
            .unwrap_or(f32::NAN)
    }
}
