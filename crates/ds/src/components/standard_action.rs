//! The standard shortcuts as data (design/27-HIG-PARITY.md section 6.2; design/06-INTERACTIONS.md
//! section 2): every combination the Mac reserves, and the few this desktop reserves for
//! itself, each bound to what it does. A binding is written in Mac terms (Command is
//! [`Key::Super`], drawn `⌘`); the platform layer resolves Command (Toshy maps it to Ctrl).
//!
//! [`Shortcut::standard`] binds one of these; [`Shortcut::custom`] refuses any combination in
//! the table, so an app cannot repurpose Cmd+S for a sidebar again.

use super::vocab::{Key, Shortcut};

/// A standard action and its reserved keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn keys(self) -> Vec<Key> {
        use Key::{Alt as Opt, Ctrl, Shift, Super as Cmd};
        let (mods, key): (&[Key], Key) = match self {
            StandardAction::Launcher => (&[Cmd], Key::Space),
            StandardAction::NextInputSource => (&[Ctrl], Key::Space),
            StandardAction::EmojiAndSymbols => (&[Ctrl, Cmd], Key::Space),
            StandardAction::NextApp => (&[Cmd], Key::Tab),
            StandardAction::PreviousApp => (&[Shift, Cmd], Key::Tab),
            StandardAction::NextWindow => (&[Cmd], Key::Char('`')),
            StandardAction::Settings => (&[Cmd], Key::Char(',')),
            StandardAction::Cancel => (&[Cmd], Key::Char('.')),
            StandardAction::Help => (&[Cmd], Key::Char('?')),
            StandardAction::SelectAll => (&[Cmd], Key::Char('a')),
            StandardAction::Copy => (&[Cmd], Key::Char('c')),
            StandardAction::Cut => (&[Cmd], Key::Char('x')),
            StandardAction::Paste => (&[Cmd], Key::Char('v')),
            StandardAction::PasteAndMatchStyle => (&[Opt, Shift, Cmd], Key::Char('v')),
            StandardAction::Undo => (&[Cmd], Key::Char('z')),
            StandardAction::Redo => (&[Shift, Cmd], Key::Char('z')),
            StandardAction::Find => (&[Cmd], Key::Char('f')),
            StandardAction::FindNext => (&[Cmd], Key::Char('g')),
            StandardAction::FindPrevious => (&[Shift, Cmd], Key::Char('g')),
            StandardAction::UseSelectionForFind => (&[Cmd], Key::Char('e')),
            StandardAction::JumpToSelection => (&[Cmd], Key::Char('j')),
            StandardAction::Bold => (&[Cmd], Key::Char('b')),
            StandardAction::Italic => (&[Cmd], Key::Char('i')),
            StandardAction::Underline => (&[Cmd], Key::Char('u')),
            StandardAction::ShowFonts => (&[Cmd], Key::Char('t')),
            StandardAction::ShowColors => (&[Shift, Cmd], Key::Char('c')),
            StandardAction::Bigger => (&[Cmd], Key::Char('+')),
            StandardAction::Smaller => (&[Cmd], Key::Char('-')),
            StandardAction::Hide => (&[Cmd], Key::Char('h')),
            StandardAction::HideOthers => (&[Opt, Cmd], Key::Char('h')),
            StandardAction::Minimize => (&[Cmd], Key::Char('m')),
            StandardAction::MinimizeAll => (&[Opt, Cmd], Key::Char('m')),
            StandardAction::New => (&[Cmd], Key::Char('n')),
            StandardAction::Open => (&[Cmd], Key::Char('o')),
            StandardAction::Save => (&[Cmd], Key::Char('s')),
            StandardAction::SaveAs => (&[Shift, Cmd], Key::Char('s')),
            StandardAction::Print => (&[Cmd], Key::Char('p')),
            StandardAction::PageSetup => (&[Shift, Cmd], Key::Char('p')),
            StandardAction::Close => (&[Cmd], Key::Char('w')),
            StandardAction::CloseAll => (&[Opt, Cmd], Key::Char('w')),
            StandardAction::Quit => (&[Cmd], Key::Char('q')),
            StandardAction::ToggleToolbar => (&[Opt, Cmd], Key::Char('t')),
            StandardAction::ToggleSidebar => (&[Ctrl, Cmd], Key::Char('s')),
            StandardAction::FullScreen => (&[Ctrl, Cmd], Key::Char('f')),
            StandardAction::ToggleDock => (&[Opt, Cmd], Key::Char('d')),
            StandardAction::QuickLook => (&[Cmd], Key::Char('y')),
            StandardAction::Reveal => (&[Cmd], Key::Char('r')),
            StandardAction::ScreenshotScreen => (&[Shift, Cmd], Key::Char('3')),
            StandardAction::ScreenshotSelection => (&[Shift, Cmd], Key::Char('4')),
            StandardAction::ScreenshotTools => (&[Shift, Cmd], Key::Char('5')),
            StandardAction::ForceQuit => (&[Opt, Cmd], Key::Escape),
            StandardAction::LockScreen => (&[Ctrl, Cmd], Key::Char('q')),
            StandardAction::LogOut => (&[Shift, Cmd], Key::Char('q')),
            StandardAction::MissionControl => (&[Ctrl], Key::Up),
            StandardAction::AppWindows => (&[Ctrl], Key::Down),
            StandardAction::SpaceLeft => (&[Ctrl], Key::Left),
            StandardAction::SpaceRight => (&[Ctrl], Key::Right),
            StandardAction::SwitchToSpace(n) => (&[Ctrl], Key::Char(n.digit())),
        };
        mods.iter().copied().chain([key]).collect()
    }

    /// The standard action that owns `keys`, in any order and any letter case.
    pub fn owning(keys: &[Key]) -> Option<StandardAction> {
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
    pub fn custom(keys: impl IntoIterator<Item = Key>) -> Result<Shortcut, Reserved> {
        let keys = normalized(keys);
        match StandardAction::owning(&keys) {
            Some(action) => Err(Reserved(action)),
            None => Ok(Shortcut(keys)),
        }
    }
}

/// `keys` with its modifiers first, deduplicated, in the Mac's order (⌃⌥⇧⌘), then the rest in
/// the order given, letters lower-cased.
pub(crate) fn normalized(keys: impl IntoIterator<Item = Key>) -> Vec<Key> {
    let keys: Vec<Key> = keys
        .into_iter()
        .map(|key| match key {
            Key::Char(c) => Key::Char(c.to_ascii_lowercase()),
            other => other,
        })
        .collect();
    let mut mods: Vec<(u8, Key)> = keys
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
