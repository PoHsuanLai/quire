//! The three sheets above the app's own in the cascade (customization design section 7.2): the
//! chosen look, the person's desktop-wide `style.css` and their `style.css` for this app. The
//! look is `@layer look`; the other two are unlayered, desktop-wide first, so the person's rules
//! beat the look and the app's beat the desktop's.

use super::custom_styles::CustomStyles;
use super::reach::{self, Cut};
use super::user_style::UserStyle;
use crate::css::layers;
use std::path::PathBuf;

/// The text of the chosen look's stylesheet, as the caller read it from the look Extension. It is
/// written without a layer; `Ds` puts it in `@layer look`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct LookStyle(pub String);

impl LookStyle {
    /// Whether there is no text worth drawing.
    pub fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

/// Which of the three sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The chosen look.
    Look,
    /// The person's desktop-wide `~/.config/quire/style.css`.
    Desktop,
    /// The person's `~/.config/<app>/style.css`.
    App,
}

impl Slot {
    /// Every slot, lowest in the cascade first.
    pub const ALL: [Slot; 3] = [Slot::Look, Slot::Desktop, Slot::App];

    /// The value of the `data-ds-style` attribute of the sheet's `<style>` element.
    pub const fn attribute(self) -> &'static str {
        match self {
            Slot::Look => "look",
            Slot::Desktop => "desktop",
            Slot::App => "app",
        }
    }
}

/// The folder each sheet may read files from (the look Extension's folder, the person's config
/// folders). `None` means none: that sheet may name `data:` and nothing else
/// (`reach`, design section 7.6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleHomes {
    /// The look Extension's folder.
    pub look: Option<PathBuf>,
    /// `~/.config/quire`.
    pub desktop: Option<PathBuf>,
    /// `~/.config/<app>`.
    pub app: Option<PathBuf>,
}

/// A sheet as it is drawn: which one, and the text for its `<style>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    /// Which sheet.
    pub slot: Slot,
    /// The text, with the look in its layer and references outside its folder cut.
    pub text: String,
}

/// A reference cut from one of the sheets, for the notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    /// Which sheet.
    pub slot: Slot,
    /// The line and target.
    pub cut: Cut,
}

/// A hash of the three sheets' text, the same on every run: what the crash guard compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub u64);

impl Fingerprint {
    /// Sixteen lower-case hex digits.
    pub fn hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

/// The look and the person's two sheets: what `Ds` takes in place of a lone `UserStyle`. Every
/// field is empty by default; the app fills the look (finding look Extensions is not `Ds`'s job).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Styles {
    /// The chosen look's text.
    pub look: LookStyle,
    /// `~/.config/quire/style.css`.
    pub desktop: UserStyle,
    /// `~/.config/<app>/style.css`.
    pub app: UserStyle,
    /// Where each may read files from.
    pub homes: StyleHomes,
}

impl Styles {
    fn text(&self, slot: Slot) -> &str {
        match slot {
            Slot::Look => &self.look.0,
            Slot::Desktop => &self.desktop.0,
            Slot::App => &self.app.0,
        }
    }

    fn home(&self, slot: Slot) -> Option<&std::path::Path> {
        match slot {
            Slot::Look => self.homes.look.as_deref(),
            Slot::Desktop => self.homes.desktop.as_deref(),
            Slot::App => self.homes.app.as_deref(),
        }
    }

    /// Whether all three are blank.
    pub fn is_blank(&self) -> bool {
        Slot::ALL
            .iter()
            .all(|slot| self.text(*slot).trim().is_empty())
    }

    /// The sheets to draw, lowest first: none when `custom` is `Off`, and none that is blank.
    /// Each has gone through the reach rule.
    pub fn drawn(&self, custom: CustomStyles) -> Vec<Drawn> {
        match custom {
            CustomStyles::Off => Vec::new(),
            CustomStyles::Use => Slot::ALL
                .into_iter()
                .filter(|slot| !self.text(*slot).trim().is_empty())
                .map(|slot| {
                    let (text, _) = reach::within(self.text(slot), self.home(slot));
                    let text = match slot {
                        Slot::Look => layers::look(&text),
                        Slot::Desktop | Slot::App => text,
                    };
                    Drawn { slot, text }
                })
                .collect(),
        }
    }

    /// What the reach rule cut from the three sheets, in cascade order.
    pub fn dropped(&self) -> Vec<Dropped> {
        Slot::ALL
            .into_iter()
            .flat_map(|slot| {
                let (_, cuts) = reach::within(self.text(slot), self.home(slot));
                cuts.into_iter().map(move |cut| Dropped { slot, cut })
            })
            .collect()
    }

    /// FNV-1a over the three texts, each followed by a byte no text holds.
    pub fn fingerprint(&self) -> Fingerprint {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for slot in Slot::ALL {
            for byte in self.text(slot).bytes().chain([0xff]) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        Fingerprint(hash)
    }
}
