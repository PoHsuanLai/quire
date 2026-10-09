//! The standard shortcuts as data (design/27-HIG-PARITY.md section 6.2; design/06-INTERACTIONS.md
//! section 2): every combination the Mac reserves, and the few this desktop reserves for
//! itself, each bound to what it does. A binding is written in Mac terms (Command is
//! [`ShortcutKey::Super`], drawn `⌘`); the platform layer resolves Command (Toshy maps it to Ctrl).
//!
//! [`Shortcut::standard`] binds one of these; [`Shortcut::custom`] refuses any combination in
//! the table, so an app cannot repurpose Cmd+S for a sidebar again.

use crate::vocab::{Shortcut, ShortcutKey};

/// A standard action and its reserved keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StandardAction {
    /// ⌘Space: the launcher (Spotlight's key).
    Launcher,
    /// ⌃Space: the next input source.
    NextInputSource,
    /// ⌃⌘Space: Emoji & Symbols.
    EmojiAndSymbols,
    /// ⌘Tab: the next app.
    NextApp,
    /// ⇧⌘Tab: the previous app.
    PreviousApp,
    /// ⌘\`: the app's next window.
    NextWindow,
    /// ⌘,: the app's settings.
    Settings,
    /// ⌘.: cancel the operation.
    Cancel,
    /// ⌘?: the app's help.
    Help,
    /// ⌘A: select all.
    SelectAll,
    /// ⌘C: copy (a launcher row's copy action too).
    Copy,
    /// ⌘X: cut.
    Cut,
    /// ⌘V: paste.
    Paste,
    /// ⌥⇧⌘V: paste and match style.
    PasteAndMatchStyle,
    /// ⌘Z: undo.
    Undo,
    /// ⇧⌘Z: redo.
    Redo,
    /// ⌘F: find.
    Find,
    /// ⌘G: find the next match.
    FindNext,
    /// ⇧⌘G: find the previous match.
    FindPrevious,
    /// ⌘E: use the selection for find.
    UseSelectionForFind,
    /// ⌘J: jump to the selection.
    JumpToSelection,
    /// ⌘B: bold.
    Bold,
    /// ⌘I: italic.
    Italic,
    /// ⌘U: underline.
    Underline,
    /// ⌘T: show the fonts (in an app with tabs, a new tab).
    ShowFonts,
    /// ⇧⌘C: show the colours.
    ShowColors,
    /// ⌘+: bigger.
    Bigger,
    /// ⌘-: smaller.
    Smaller,
    /// ⌘H: hide the app.
    Hide,
    /// ⌥⌘H: hide the other apps.
    HideOthers,
    /// ⌘M: minimize the window.
    Minimize,
    /// ⌥⌘M: minimize the app's windows.
    MinimizeAll,
    /// ⌘N: new.
    New,
    /// ⌘O: open.
    Open,
    /// ⌘S: save.
    Save,
    /// ⇧⌘S: save as (duplicate).
    SaveAs,
    /// ⌘P: print.
    Print,
    /// ⇧⌘P: page setup.
    PageSetup,
    /// ⌘W: close the window.
    Close,
    /// ⌥⌘W: close the app's windows.
    CloseAll,
    /// ⌘Q: quit the app.
    Quit,
    /// ⌥⌘T: show or hide the toolbar.
    ToggleToolbar,
    /// ⌃⌘S: show or hide the sidebar (the prototype's Mod+S, re-mapped; design/06 section 2).
    ToggleSidebar,
    /// ⌃⌘F: full screen.
    FullScreen,
    /// ⌥⌘D: show or hide the dock.
    ToggleDock,
    /// ⌘Y: Quick Look, the preview.
    QuickLook,
    /// ⌘R: reveal the item in Files (Spotlight's).
    Reveal,
    /// ⇧⌘3: a screenshot of the screen.
    ScreenshotScreen,
    /// ⇧⌘4: a screenshot of a selection.
    ScreenshotSelection,
    /// ⇧⌘5: the screenshot tools.
    ScreenshotTools,
    /// ⌥⌘Esc: Force Quit.
    ForceQuit,
    /// ⌃⌘Q: lock the screen.
    LockScreen,
    /// ⇧⌘Q: log out.
    LogOut,
    /// ⌃↑: Mission Control, every window.
    MissionControl,
    /// ⌃↓: the app's windows.
    AppWindows,
    /// ⌃←: the Space to the left.
    SpaceLeft,
    /// ⌃→: the Space to the right.
    SpaceRight,
    /// ⌃1 to ⌃9: switch to Space n (design/27 section 8 decision 5; Cmd+1..9 stay the apps').
    SwitchToSpace(SpaceNumber),
}

/// A Space's number for [`StandardAction::SwitchToSpace`]: 1 to 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpaceNumber(u8);

impl SpaceNumber {
    /// Space `n`, when `n` is 1 to 9.
    pub fn new(n: u8) -> Option<Self> {
        (1..=9).contains(&n).then_some(SpaceNumber(n))
    }

    /// The number, 1 to 9.
    pub fn get(self) -> u8 {
        self.0
    }

    /// The digit key that switches to it.
    fn digit(self) -> char {
        char::from(b'0' + self.0)
    }
}

/// A combination [`Shortcut::custom`] refused: the standard action that owns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reserved(pub StandardAction);

impl std::fmt::Display for Reserved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is reserved for {:?}",
            Shortcut::standard(self.0).glyphs(),
            self.0
        )
    }
}

impl std::error::Error for Reserved {}

impl StandardAction {
    /// Every standard action, the nine Space switches included.
    pub const ALL: [StandardAction; 66] = [
        StandardAction::Launcher,
        StandardAction::NextInputSource,
        StandardAction::EmojiAndSymbols,
        StandardAction::NextApp,
        StandardAction::PreviousApp,
        StandardAction::NextWindow,
        StandardAction::Settings,
        StandardAction::Cancel,
        StandardAction::Help,
        StandardAction::SelectAll,
        StandardAction::Copy,
        StandardAction::Cut,
        StandardAction::Paste,
        StandardAction::PasteAndMatchStyle,
        StandardAction::Undo,
        StandardAction::Redo,
        StandardAction::Find,
        StandardAction::FindNext,
        StandardAction::FindPrevious,
        StandardAction::UseSelectionForFind,
        StandardAction::JumpToSelection,
        StandardAction::Bold,
        StandardAction::Italic,
        StandardAction::Underline,
        StandardAction::ShowFonts,
        StandardAction::ShowColors,
        StandardAction::Bigger,
        StandardAction::Smaller,
        StandardAction::Hide,
        StandardAction::HideOthers,
        StandardAction::Minimize,
        StandardAction::MinimizeAll,
        StandardAction::New,
        StandardAction::Open,
        StandardAction::Save,
        StandardAction::SaveAs,
        StandardAction::Print,
        StandardAction::PageSetup,
        StandardAction::Close,
        StandardAction::CloseAll,
        StandardAction::Quit,
        StandardAction::ToggleToolbar,
        StandardAction::ToggleSidebar,
        StandardAction::FullScreen,
        StandardAction::ToggleDock,
        StandardAction::QuickLook,
        StandardAction::Reveal,
        StandardAction::ScreenshotScreen,
        StandardAction::ScreenshotSelection,
        StandardAction::ScreenshotTools,
        StandardAction::ForceQuit,
        StandardAction::LockScreen,
        StandardAction::LogOut,
        StandardAction::MissionControl,
        StandardAction::AppWindows,
        StandardAction::SpaceLeft,
        StandardAction::SpaceRight,
        StandardAction::SwitchToSpace(SpaceNumber(1)),
        StandardAction::SwitchToSpace(SpaceNumber(2)),
        StandardAction::SwitchToSpace(SpaceNumber(3)),
        StandardAction::SwitchToSpace(SpaceNumber(4)),
        StandardAction::SwitchToSpace(SpaceNumber(5)),
        StandardAction::SwitchToSpace(SpaceNumber(6)),
        StandardAction::SwitchToSpace(SpaceNumber(7)),
        StandardAction::SwitchToSpace(SpaceNumber(8)),
        StandardAction::SwitchToSpace(SpaceNumber(9)),
    ];

    /// The keys, modifiers in the Mac's order (⌃⌥⇧⌘) then the key.
    pub fn keys(self) -> Vec<ShortcutKey> {
        use ShortcutKey::{Alt as Opt, Ctrl, Shift, Super as Cmd};
        let (mods, key): (&[ShortcutKey], ShortcutKey) = match self {
            StandardAction::Launcher => (&[Cmd], ShortcutKey::Space),
            StandardAction::NextInputSource => (&[Ctrl], ShortcutKey::Space),
            StandardAction::EmojiAndSymbols => (&[Ctrl, Cmd], ShortcutKey::Space),
            StandardAction::NextApp => (&[Cmd], ShortcutKey::Tab),
            StandardAction::PreviousApp => (&[Shift, Cmd], ShortcutKey::Tab),
            StandardAction::NextWindow => (&[Cmd], ShortcutKey::Char('`')),
            StandardAction::Settings => (&[Cmd], ShortcutKey::Char(',')),
            StandardAction::Cancel => (&[Cmd], ShortcutKey::Char('.')),
            StandardAction::Help => (&[Cmd], ShortcutKey::Char('?')),
            StandardAction::SelectAll => (&[Cmd], ShortcutKey::Char('a')),
            StandardAction::Copy => (&[Cmd], ShortcutKey::Char('c')),
            StandardAction::Cut => (&[Cmd], ShortcutKey::Char('x')),
            StandardAction::Paste => (&[Cmd], ShortcutKey::Char('v')),
            StandardAction::PasteAndMatchStyle => (&[Opt, Shift, Cmd], ShortcutKey::Char('v')),
            StandardAction::Undo => (&[Cmd], ShortcutKey::Char('z')),
            StandardAction::Redo => (&[Shift, Cmd], ShortcutKey::Char('z')),
            StandardAction::Find => (&[Cmd], ShortcutKey::Char('f')),
            StandardAction::FindNext => (&[Cmd], ShortcutKey::Char('g')),
            StandardAction::FindPrevious => (&[Shift, Cmd], ShortcutKey::Char('g')),
            StandardAction::UseSelectionForFind => (&[Cmd], ShortcutKey::Char('e')),
            StandardAction::JumpToSelection => (&[Cmd], ShortcutKey::Char('j')),
            StandardAction::Bold => (&[Cmd], ShortcutKey::Char('b')),
            StandardAction::Italic => (&[Cmd], ShortcutKey::Char('i')),
            StandardAction::Underline => (&[Cmd], ShortcutKey::Char('u')),
            StandardAction::ShowFonts => (&[Cmd], ShortcutKey::Char('t')),
            StandardAction::ShowColors => (&[Shift, Cmd], ShortcutKey::Char('c')),
            StandardAction::Bigger => (&[Cmd], ShortcutKey::Char('+')),
            StandardAction::Smaller => (&[Cmd], ShortcutKey::Char('-')),
            StandardAction::Hide => (&[Cmd], ShortcutKey::Char('h')),
            StandardAction::HideOthers => (&[Opt, Cmd], ShortcutKey::Char('h')),
            StandardAction::Minimize => (&[Cmd], ShortcutKey::Char('m')),
            StandardAction::MinimizeAll => (&[Opt, Cmd], ShortcutKey::Char('m')),
            StandardAction::New => (&[Cmd], ShortcutKey::Char('n')),
            StandardAction::Open => (&[Cmd], ShortcutKey::Char('o')),
            StandardAction::Save => (&[Cmd], ShortcutKey::Char('s')),
            StandardAction::SaveAs => (&[Shift, Cmd], ShortcutKey::Char('s')),
            StandardAction::Print => (&[Cmd], ShortcutKey::Char('p')),
            StandardAction::PageSetup => (&[Shift, Cmd], ShortcutKey::Char('p')),
            StandardAction::Close => (&[Cmd], ShortcutKey::Char('w')),
            StandardAction::CloseAll => (&[Opt, Cmd], ShortcutKey::Char('w')),
            StandardAction::Quit => (&[Cmd], ShortcutKey::Char('q')),
            StandardAction::ToggleToolbar => (&[Opt, Cmd], ShortcutKey::Char('t')),
            StandardAction::ToggleSidebar => (&[Ctrl, Cmd], ShortcutKey::Char('s')),
            StandardAction::FullScreen => (&[Ctrl, Cmd], ShortcutKey::Char('f')),
            StandardAction::ToggleDock => (&[Opt, Cmd], ShortcutKey::Char('d')),
            StandardAction::QuickLook => (&[Cmd], ShortcutKey::Char('y')),
            StandardAction::Reveal => (&[Cmd], ShortcutKey::Char('r')),
            StandardAction::ScreenshotScreen => (&[Shift, Cmd], ShortcutKey::Char('3')),
            StandardAction::ScreenshotSelection => (&[Shift, Cmd], ShortcutKey::Char('4')),
            StandardAction::ScreenshotTools => (&[Shift, Cmd], ShortcutKey::Char('5')),
            StandardAction::ForceQuit => (&[Opt, Cmd], ShortcutKey::Escape),
            StandardAction::LockScreen => (&[Ctrl, Cmd], ShortcutKey::Char('q')),
            StandardAction::LogOut => (&[Shift, Cmd], ShortcutKey::Char('q')),
            StandardAction::MissionControl => (&[Ctrl], ShortcutKey::Up),
            StandardAction::AppWindows => (&[Ctrl], ShortcutKey::Down),
            StandardAction::SpaceLeft => (&[Ctrl], ShortcutKey::Left),
            StandardAction::SpaceRight => (&[Ctrl], ShortcutKey::Right),
            StandardAction::SwitchToSpace(n) => (&[Ctrl], ShortcutKey::Char(n.digit())),
        };
        mods.iter().copied().chain([key]).collect()
    }

    /// The standard action that owns `keys`, in any order and any letter case.
    pub fn owning(keys: &[ShortcutKey]) -> Option<StandardAction> {
        let wanted = normalized(keys.iter().copied());
        StandardAction::ALL
            .into_iter()
            .find(|action| normalized(action.keys()) == wanted)
    }
}

impl Shortcut {
    /// The keys of a standard action: the only way to bind one.
    pub fn standard(action: StandardAction) -> Shortcut {
        Shortcut(action.keys())
    }

    /// A shortcut of the app's own, modifiers put in the Mac's order; refused with the owner
    /// when the combination is a standard one (design/27 section 6.2: never repurpose one).
    pub fn custom(keys: impl IntoIterator<Item = ShortcutKey>) -> Result<Shortcut, Reserved> {
        let keys = normalized(keys);
        match StandardAction::owning(&keys) {
            Some(action) => Err(Reserved(action)),
            None => Ok(Shortcut(keys)),
        }
    }
}

/// `keys` with its modifiers first, deduplicated, in the Mac's order (⌃⌥⇧⌘), then the rest in
/// the order given, letters lower-cased.
pub(crate) fn normalized(keys: impl IntoIterator<Item = ShortcutKey>) -> Vec<ShortcutKey> {
    let keys: Vec<ShortcutKey> = keys
        .into_iter()
        .map(|key| match key {
            ShortcutKey::Char(c) => ShortcutKey::Char(c.to_ascii_lowercase()),
            other => other,
        })
        .collect();
    let mut mods: Vec<(u8, ShortcutKey)> = keys
        .iter()
        .filter_map(|key| key.modifier_rank().map(|rank| (rank, *key)))
        .collect();
    mods.sort_by_key(|(rank, _)| *rank);
    mods.dedup();
    let rest = keys
        .iter()
        .copied()
        .filter(|key| key.modifier_rank().is_none());
    mods.into_iter().map(|(_, key)| key).chain(rest).collect()
}
