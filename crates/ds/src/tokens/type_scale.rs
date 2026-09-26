//! The four faces and the size ramp (design/02-TYPE.md sections 2 and 4).
//!
//! The plan names the ends, `--fs-micro` 9.5 and `--fs-display` 26; the steps between are named
//! here by role (design/02-TYPE.md open decision 3), one per distinct size in the ramp.

use super::name::VarName;

/// A type family, by job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// `--font-display`: Bricolage Grotesque. Headings, names, initials.
    Display,
    /// `--font-ui`: Karla. Body text and every control.
    Ui,
    /// `--font-data`: Space Mono. Anything machine-shaped.
    Data,
    /// `--font-serif`: Noto Serif. A message a person writes in a serif, and the control that
    /// offers it (mailo gaps 3); never the interface's own text.
    Serif,
}

impl Family {
    /// Every face, in the order the stylesheet declares them.
    pub const ALL: [Family; 4] = [Family::Display, Family::Ui, Family::Data, Family::Serif];

    /// The custom property: `--font-display`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            Family::Display => "--font-display",
            Family::Ui => "--font-ui",
            Family::Data => "--font-data",
            Family::Serif => "--font-serif",
        })
    }

    /// The `font-family` stack, face first, then its fallbacks.
    pub fn stack(self) -> &'static str {
        match self {
            Family::Display => "\"Bricolage Grotesque\",\"Trebuchet MS\",system-ui,sans-serif",
            Family::Ui => "\"Karla\",\"Segoe UI\",system-ui,sans-serif",
            Family::Data => "\"Space Mono\",ui-monospace,\"SFMono-Regular\",Menlo,monospace",
            Family::Serif => "\"Noto Serif\",Georgia,\"Times New Roman\",serif",
        }
    }
}

/// One step of the size ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSize {
    /// `--fs-pico` 7.5: in-row provider mark.
    Pico,
    /// `--fs-nano` 9: pin count, favicon letter.
    Nano,
    /// `--fs-micro` 9.5: chip, via, group header.
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
    /// `--fs-widget-hero` 44: a widget's hero value (design/23-WIDGETS.md section 3.2).
    WidgetHero,
    /// `--fs-lock-clock` 140: the lock screen's time, the largest type the shell draws
    /// (design/20-SURFACES.md section 1.9; design/04-COMPONENTS.md section 42).
    LockClock,
}

impl FontSize {
    /// Every step, smallest first.
    pub const ALL: [FontSize; 24] = [
        FontSize::Pico,
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
        FontSize::Subject,
        FontSize::Heading,
        FontSize::Amount,
        FontSize::Day,
        FontSize::Display,
        FontSize::WidgetHero,
        FontSize::LockClock,
    ];

    /// The custom property: `--fs-micro`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            FontSize::Pico => "--fs-pico",
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
            FontSize::Subject => "--fs-subject",
            FontSize::Heading => "--fs-heading",
            FontSize::Amount => "--fs-amount",
            FontSize::Day => "--fs-day",
            FontSize::Display => "--fs-display",
            FontSize::WidgetHero => "--fs-widget-hero",
            FontSize::LockClock => "--fs-lock-clock",
        })
    }

    /// The size in CSS: `9.5px`.
    pub fn css(self) -> &'static str {
        match self {
            FontSize::Pico => "7.5px",
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
            FontSize::Subject => "20px",
            FontSize::Heading => "21px",
            FontSize::Amount => "22px",
            FontSize::Day => "24px",
            FontSize::Display => "26px",
            FontSize::WidgetHero => "44px",
            FontSize::LockClock => "140px",
        }
    }
}
