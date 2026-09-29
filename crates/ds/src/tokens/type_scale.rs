//! The five face jobs and the size ramp (design/02-TYPE.md sections 2 and 4).
//!
//! The plan names the ends, `--fs-micro` 9.5 and `--fs-display` 26; the steps between are named
//! here by role (design/02-TYPE.md open decision 3), one per distinct size in the ramp.

use super::name::VarName;
use crate::appearance::typeface::Typeface;

/// A type family, by job. Which face does each job depends on the root's [`Typeface`]
/// (design/02-TYPE.md section 2): the `.ds` block names the System faces and the
/// `.ds[data-typeface=editorial]` block the Editorial ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// `--font-display`: Inter Display (System) or Bricolage Grotesque (Editorial). Headings,
    /// names, initials, big numbers.
    Display,
    /// `--font-ui`: Inter (System) or Karla (Editorial). Body text and every control.
    Ui,
    /// `--font-data`: Inter, always tabular (System), or Space Mono (Editorial). Anything
    /// machine-shaped: times, counts, chips, eyebrows, section headers.
    Data,
    /// `--font-serif`: Noto Serif. A message a person writes in a serif, and the control that
    /// offers it; never the interface's own text.
    Serif,
    /// `--font-code`: Space Mono in either typeface. Only where a fixed pitch carries meaning:
    /// code, `Kbd`, aligned logs.
    Code,
}

impl Family {
    /// Every face, in the order the stylesheet declares them.
    pub const ALL: [Family; 5] = [
        Family::Display,
        Family::Ui,
        Family::Data,
        Family::Serif,
        Family::Code,
    ];

    /// The custom property: `--font-display`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            Family::Display => "--font-display",
            Family::Ui => "--font-ui",
            Family::Data => "--font-data",
            Family::Serif => "--font-serif",
            Family::Code => "--font-code",
        })
    }

    /// The `font-family` stack under the default typeface ([`Typeface::System`]).
    pub fn stack(self) -> &'static str {
        self.stack_in(Typeface::System)
    }

    /// The `font-family` stack under `typeface`, face first, then its fallbacks.
    pub fn stack_in(self, typeface: Typeface) -> &'static str {
        match (self, typeface) {
            (Family::Display, Typeface::System) => {
                "\"Inter Display\",\"Inter\",system-ui,sans-serif"
            }
            (Family::Ui | Family::Data, Typeface::System) => "\"Inter\",system-ui,sans-serif",
            (Family::Display, Typeface::Editorial) => {
                "\"Bricolage Grotesque\",\"Trebuchet MS\",system-ui,sans-serif"
            }
            (Family::Ui, Typeface::Editorial) => "\"Karla\",\"Segoe UI\",system-ui,sans-serif",
            (Family::Data, Typeface::Editorial) | (Family::Code, _) => {
                "\"Space Mono\",ui-monospace,\"SFMono-Regular\",Menlo,monospace"
            }
            (Family::Serif, _) => "\"Noto Serif\",Georgia,\"Times New Roman\",serif",
        }
    }

    /// The family name the stack leads with under `typeface`: the name its face registers as.
    pub fn face_name(self, typeface: Typeface) -> &'static str {
        let stack = self.stack_in(typeface);
        stack
            .strip_prefix('"')
            .and_then(|rest| rest.split('"').next())
            .unwrap_or(stack)
    }
}

/// Whether a size step is the same under both typefaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voiced {
    /// One value, declared once on `.ds`.
    Fixed,
    /// A value per typeface, declared on each typeface's block.
    PerTypeface,
}

/// One step of the size ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSize {
    /// `--fs-pico` 7.5: in-row provider mark.
    Pico,
    /// `--fs-dial` 9 (10 under System, the floor): a medium world clock dial's numerals
    /// (design/23-WIDGETS.md section 4.2).
    Dial,
    /// `--fs-nano` 9 (10 under System): pin count, favicon letter.
    Nano,
    /// `--fs-micro` 9.5 (10 under System): chip, via, group header.
    Micro,
    /// `--fs-caption` 10: row time, section header, shortcut.
    Caption,
    /// `--fs-note` 10.5: kbd, count, hover-card sub.
    Note,
    /// `--fs-eyebrow` 11: eyebrow, link pill.
    Eyebrow,
    /// `--fs-help` 11.5: menu help, command snippet, view switch.
    Help,
    /// `--fs-small` 12: mini, segmented, tooltip.
    Small,
    /// `--fs-meta` 12.5: snippet, toast, chip person.
    Meta,
    /// `--fs-control` 13: command pill, menu item, button.
    Control,
    /// `--fs-body` 13.5: row name and subject, sidebar item, input.
    Body,
    /// `--fs-reading` 14: reader body, hover-card title.
    Reading,
    /// `--fs-compose` 14.5: composer body.
    Compose,
    /// `--fs-base` 15: the base text.
    Base,
    /// `--fs-subhead` 15.5: parsed-body subheading.
    Subhead,
    /// `--fs-title` 16: list title, command input.
    Title,
    /// `--fs-heading-3` 16.5: composer `h3`.
    Heading3,
    /// `--fs-dial-large` 18: a small clock widget's dial numerals (design/23 section 4.2).
    DialLarge,
    /// `--fs-widget-figure` 20: a battery's percentage under its ring (design/23 section 4.1).
    WidgetFigure,
    /// `--fs-subject` 20: reader subject.
    Subject,
    /// `--fs-heading` 21: parsed-body and composer headings.
    Heading,
    /// `--fs-amount` 22: receipt amount.
    Amount,
    /// `--fs-day` 24: event day number.
    Day,
    /// `--fs-display` 26: composer subject.
    Display,
    /// `--fs-emoji-cell` 30: an emoji in an emoji grid's 56 px cell.
    EmojiCell,
    /// `--fs-widget-hero` 47: a widget's hero value, a small battery's percentage (design/23-WIDGETS.md section 3.2).
    WidgetHero,
    /// `--fs-emoji-preview` 96: one emoji in a preview pane.
    EmojiPreview,
    /// `--fs-lock-clock` 140: the lock screen's time, the largest type the shell draws
    /// (design/20-SURFACES.md section 1.9; design/04-COMPONENTS.md section 42).
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

    /// Every step, smallest first.
    pub const ALL: [FontSize; 29] = [
        FontSize::Pico,
        FontSize::Dial,
        FontSize::Nano,
        FontSize::Micro,
        FontSize::Caption,
        FontSize::Note,
        FontSize::Eyebrow,
        FontSize::Help,
        FontSize::Small,
        FontSize::Meta,
        FontSize::Control,
        FontSize::Body,
        FontSize::Reading,
        FontSize::Compose,
        FontSize::Base,
        FontSize::Subhead,
        FontSize::Title,
        FontSize::Heading3,
        FontSize::DialLarge,
        FontSize::WidgetFigure,
        FontSize::Subject,
        FontSize::Heading,
        FontSize::Amount,
        FontSize::Day,
        FontSize::Display,
        FontSize::EmojiCell,
        FontSize::WidgetHero,
        FontSize::EmojiPreview,
        FontSize::LockClock,
    ];

    /// The custom property: `--fs-micro`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            FontSize::Pico => "--fs-pico",
            FontSize::Dial => "--fs-dial",
            FontSize::Nano => "--fs-nano",
            FontSize::Micro => "--fs-micro",
            FontSize::Caption => "--fs-caption",
            FontSize::Note => "--fs-note",
            FontSize::Eyebrow => "--fs-eyebrow",
            FontSize::Help => "--fs-help",
            FontSize::Small => "--fs-small",
            FontSize::Meta => "--fs-meta",
            FontSize::Control => "--fs-control",
            FontSize::Body => "--fs-body",
            FontSize::Reading => "--fs-reading",
            FontSize::Compose => "--fs-compose",
            FontSize::Base => "--fs-base",
            FontSize::Subhead => "--fs-subhead",
            FontSize::Title => "--fs-title",
            FontSize::Heading3 => "--fs-heading-3",
            FontSize::DialLarge => "--fs-dial-large",
            FontSize::WidgetFigure => "--fs-widget-figure",
            FontSize::Subject => "--fs-subject",
            FontSize::Heading => "--fs-heading",
            FontSize::Amount => "--fs-amount",
            FontSize::Day => "--fs-day",
            FontSize::Display => "--fs-display",
            FontSize::EmojiCell => "--fs-emoji-cell",
            FontSize::EmojiPreview => "--fs-emoji-preview",
            FontSize::WidgetHero => "--fs-widget-hero",
            FontSize::LockClock => "--fs-lock-clock",
        })
    }

    /// The size in CSS under the default typeface ([`Typeface::System`]): `9.5px`.
    pub fn css(self) -> &'static str {
        self.css_in(Typeface::System)
    }

    /// The size in CSS under `typeface`. Every step is the same in both but the five fitted to a
    /// measured cap height (design/23 section 1.1, the display face's cap at .66 em): under
    /// System those are the Editorial size x .66 / .7275 (Inter Display's cap height), rounded
    /// to .5 px, so the drawn caps keep the measured heights (design/02 open decision 8); and
    /// the [`Self::FLOORED`] steps, which System raises to the 10 px floor (design/27 section
    /// 3.16; the dial's fitted 8 px included).
    pub fn css_in(self, typeface: Typeface) -> &'static str {
        match (self, typeface) {
            (FontSize::Dial | FontSize::Nano | FontSize::Micro, Typeface::System) => "10px",
            (FontSize::DialLarge, Typeface::System) => "16.5px",
            (FontSize::WidgetFigure, Typeface::System) => "18px",
            (FontSize::WidgetHero, Typeface::System) => "42.5px",
            (FontSize::LockClock, Typeface::System) => "127px",
            _ => self.editorial_css(),
        }
    }

    /// The size in px under `typeface`, for a check against [`Self::MIN_PX`].
    pub fn px_in(self, typeface: Typeface) -> f32 {
        self.css_in(typeface)
            .trim_end_matches("px")
            .parse()
            .unwrap_or(f32::NAN)
    }

    /// Whether the size differs between the typefaces: declared on each typeface's block
    /// rather than once.
    pub fn follows_typeface(self) -> Voiced {
        if self.css_in(Typeface::System) == self.css_in(Typeface::Editorial) {
            Voiced::Fixed
        } else {
            Voiced::PerTypeface
        }
    }

    /// The size under Editorial: the ramp as design/02 section 4.2 gives it.
    fn editorial_css(self) -> &'static str {
        match self {
            FontSize::Pico => "7.5px",
            FontSize::Dial => "9px",
            FontSize::Nano => "9px",
            FontSize::Micro => "9.5px",
            FontSize::Caption => "10px",
            FontSize::Note => "10.5px",
            FontSize::Eyebrow => "11px",
            FontSize::Help => "11.5px",
            FontSize::Small => "12px",
            FontSize::Meta => "12.5px",
            FontSize::Control => "13px",
            FontSize::Body => "13.5px",
            FontSize::Reading => "14px",
            FontSize::Compose => "14.5px",
            FontSize::Base => "15px",
            FontSize::Subhead => "15.5px",
            FontSize::Title => "16px",
            FontSize::Heading3 => "16.5px",
            FontSize::DialLarge => "18px",
            FontSize::WidgetFigure => "20px",
            FontSize::Subject => "20px",
            FontSize::Heading => "21px",
            FontSize::Amount => "22px",
            FontSize::Day => "24px",
            FontSize::Display => "26px",
            FontSize::EmojiCell => "30px",
            FontSize::EmojiPreview => "96px",
            FontSize::WidgetHero => "47px",
            FontSize::LockClock => "140px",
        }
    }
}
