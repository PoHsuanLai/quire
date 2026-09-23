//! The three faces and the size ramp (design/02-TYPE.md sections 2 and 4).
//!
//! The plan names the ends, `--fs-micro` 9.5 and `--fs-display` 26; the steps between are named
//! here by role (design/02-TYPE.md open decision 3), one per distinct size in the ramp.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
}

impl Family {
    /// The custom property: `--font-display`, …
    pub fn var(self) -> VarName {
        todo!()
    }

    /// The `font-family` stack, face first, then its fallbacks.
    pub fn stack(self) -> &'static str {
        todo!()
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
}

impl FontSize {
    /// Every step, smallest first.
    pub const ALL: [FontSize; 22] = [
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
    ];

    /// The custom property: `--fs-micro`, …
    pub fn var(self) -> VarName {
        todo!()
    }

    /// The size in CSS: `9.5px`.
    pub fn css(self) -> &'static str {
        todo!()
    }
}
