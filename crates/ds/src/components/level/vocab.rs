//! The level control's vocabulary: what it is for, how it looks, whether it ticks, and which
//! glyph it carries. Every choice is a named variant (no `bool`, CONVENTIONS section 11).

/// Whether the control takes input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LevelMode {
    /// A control: pointer and keys move it (`role="slider"`, focusable).
    #[default]
    Interactive,
    /// A read-only level (`role="progressbar"`, not focusable, no handlers): the OSD's.
    ReadOnly,
}

impl LevelMode {
    /// The `data-mode` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            LevelMode::Interactive => "interactive",
            LevelMode::ReadOnly => "read-only",
        }
    }
}

/// How the level is drawn: three looks for the user to choose between (FINDINGS "Level control
/// (2026-09-25)" has the references each follows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LevelLook {
    /// A thick capsule whose fill is the material's bright ink, with the glyph inside at its left
    /// end, two-toned where the fill covers it (recommended).
    #[default]
    Capsule,
    /// The capsule with a separate round knob riding the fill's end, the glyph before it.
    CapsuleKnob,
    /// Sixteen rounded squares that fill in one by one, the glyph before them.
    Segments,
}

impl LevelLook {
    /// Every look, in the gallery's order.
    pub const ALL: [LevelLook; 3] = [
        LevelLook::Capsule,
        LevelLook::CapsuleKnob,
        LevelLook::Segments,
    ];

    /// The `data-look` value.
    pub fn slug(self) -> &'static str {
        match self {
            LevelLook::Capsule => "capsule",
            LevelLook::CapsuleKnob => "capsule-knob",
            LevelLook::Segments => "segments",
        }
    }
}

/// Whether the fill's edge marks each step it crosses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tick {
    /// No mark.
    #[default]
    Off,
    /// A quiet mark at the fill's edge (`Anim::LevelTick`) each time the level crosses one of
    /// the sixteen steps. The sound, where there is one, is the shell's.
    Quiet,
}

/// Whether a volume is heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Muting {
    /// Heard: the speaker's waves follow the level.
    #[default]
    Audible,
    /// Muted: the waves go and a slash comes in.
    Muted,
}

/// The glyph a level carries, which follows the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelGlyph {
    /// A speaker: no wave at 0, then one, two and three waves by thirds; muted, a slash.
    Volume(Muting),
    /// A sun whose rays grow with the brightness.
    Brightness,
}
