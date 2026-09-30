//! quire's renderer-free design system: the host seams, the overlay stack, the root, the generic
//! components, and the assembly of the stylesheet and `Ds`. It sits over three crates and re-exports
//! their names, one path each: `ds-core` (vocabulary, geometry, time, the `Spawner`, colour),
//! `ds-style` (appearance, tokens, materials, palettes, fonts, icons, the stylesheet's sections)
//! and `ds-motion` (animation data, timers, machines and the details); its tests lint with
//! `ds-lint`. `ds-shell` builds the shell's own parts on top of this crate. Inside the crate a layer names
//! only the layers below it (`scripts/check-boundary.sh`). Every public item has one path: a root
//! name, or a name in one of the modules below. DESIGN.md maps each module to the design doc
//! section it implements.

mod assembly;
// `ds-shell` names the layers it builds on by path (`ds::components::controls::button::Button`); a
// consumer takes the root names below.
pub mod components;
pub mod detail;
mod edit;
mod file_drop;
pub mod focus;
pub mod host;
pub mod icon;
pub mod motion;
pub mod root;
mod spell;
mod stack;
pub mod time;
mod window;

pub use crate::assembly::selectors;
pub use crate::assembly::{
    ds::{Ds, Inject},
    kit::{KIT, kits},
    stylesheet::{component_sheets, stylesheet},
};
pub use crate::components::{
    app::{
        command_pill::CommandPill,
        edge_peek::{EdgePeek, SideState},
        hover_strip::{ActionId, HoverStrip, StripAction, Titles},
        link_pill::{LinkPill, LinkTarget},
        peek::Peek,
        pin_tile::{PinFace, PinTile},
        pin_tiles::{PinAdd, PinItem, PinTiles},
        send_mood::SendMood,
        send_pill::{PillAction, SEND_COUNTDOWN, SEND_TICK, SendPill},
        space_switch::{space_pressed, space_shortcut},
        thread_row::ThreadRow,
        thread_row_hooks::PartHooks,
        today_tabs::{TodayTab, TodayTabs, expiry_of, left_text},
    },
    chrome::{
        sidebar::Sidebar,
        split_view::{
            model::{Collapsing, DividerDrag, PaneSpec, SplitPane},
            view::SplitView,
        },
        tab_view::{TAB_LIMIT, TabView},
        titlebar_parts::{DocumentState, TitleParts},
        toolbar::{
            model::{ITEM_PITCH, Kept, TITLE_ROOM, ToolbarItem, ToolbarRoom},
            view::Toolbar,
        },
        traffic_lights::TilePose,
        window_frame::{TrafficLights, WindowFrame, WindowTitlebar},
    },
    content::{
        avatar::{
            Avatar, AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue, muting_colour,
            person_hue,
        },
        icon_source::{ExternalIcon, IconSource},
        icon_view::IconView,
        image_source::{ImageSize, ImageSource},
        label::{Label, LabelRole, LabelStyle},
        level_glyph::vocab::{LevelGlyph, LevelSource},
        pdf_thumb::{PdfPage, PdfThumb, PdfTrouble},
        provider_mark::{MarkProvider, MarkStyle, ProviderMark},
        rich_text::{Rich, RichRun, RichText},
        status::{
            battery::BatteryGlyph,
            battery_state::{BatteryPower, BatteryState, LowAt},
            bluetooth::BluetoothGlyph,
            bluetooth_state::BluetoothState,
            family::StatusState,
            volume::{VolumeGlyph, VolumeState, VolumeWaves},
            wifi::WifiGlyph,
            wifi_state::{WifiBars, WifiReach, WifiState},
        },
        text_runs::{RunTone, TextLine, TextRun},
        voice_orb::{
            model::{OrbColour, OrbColours},
            view::{ORB_PERIOD, ORB_SIZE, VoiceOrb},
        },
    },
    controls::{
        badge::{Badge, BadgeContent, BadgeTone},
        button::Button,
        button_face::{ButtonFace, Leading, Trailing},
        button_model::{Answers, Bezel, ButtonRole, IconSwap, ImagePosition},
        checkbox::Checkbox,
        chip::{Chip, ChipVariant},
        choice::Choice,
        disclosure::Disclosure,
        key_equivalent::{KeyEquivalent, KeyStyle},
        level_indicator::{Bands, LevelIndicator, LevelStyle},
        press::Propagation,
        progress::{
            model::{Progress, ProgressStyle, RingGap},
            view::ProgressIndicator,
        },
        radio_group::{Arrangement, RadioGroup},
        segmented::{SegmentedControl, Tracking},
        slider::Slider,
        slider_model::{SliderLook, Ticks},
        toggle::Toggle,
    },
    editor::{spell_menu::SpellMarks, surface::EditSurface},
    fields::{
        field_row::{FieldGroup, FieldRow, RowLayout},
        stepper::{
            model::{Readout, StepDirection, StepRange},
            view::Stepper,
        },
        text_field::TextField,
        text_field_focus::FieldFocus,
        text_field_model::{FieldBezel, FieldKind, Invalid, Validity},
    },
    lists::{
        appearance_picker::{AppearancePicker, PickerLayout},
        emoji_grid::grid::{EMOJI_CELL, EMOJI_COLUMNS, EmojiCell, EmojiCells, EmojiGrid},
        list::{
            list::List,
            model::{ListItem, ListRole, ListStyle},
        },
        preview::{
            content::{PANE_MEDIA, PaneContent, PaneMono},
            pane::{PaneAction, PreviewPane},
            switcher::PaneSwitcher,
        },
        row::{
            accessory::Accessory,
            action::RowAction,
            chord::{ChordShown, RowChord},
            leading::RowLeading,
            motion::RowMotion,
            row::{Outline, Row},
            shape::{ClipBody, Expiry, RowShape},
            size::RowSize,
        },
        section_header::SectionHeader,
        table::{
            model::{CellAlign, Sort, SortDirection, Sorting, TableColumn, TableRow},
            view::Table,
        },
    },
    menus::{
        item::item::{MenuImage, MenuItem},
        menu::{cursor::MenuCursor, menu::Menu, placement::MenuPlacement},
        menu_bar::{BarCommand, BarMenu, BarSection, MenuBarModel, opens},
        palette::{
            command_palette::CommandPalette,
            palette_claim::{Claim, FieldKey},
            palette_group::{GroupEntries, GroupOrder, PaletteGroup, PaletteGroups, PaletteRow},
            palette_host::CommandPaletteHost,
            palette_motion::{PaletteHandle, use_palette_handle},
        },
        pop_up_button::{PopUpButton, PopUpKind},
    },
    overlays::{
        alert::Alert,
        alert_model::{AlertButton, AlertRole, AlertStyle, Suppression},
        drag_ghost::{DragCount, DragGhost, DragReturnFrame, DropLine, Grip},
        empty_state::{EmptyForm, EmptyState},
        flow::Flow,
        hover_card::{
            HoverCard,
            intent::{HoverAnchor, use_hover_intent},
            parts::{FlagTone, HoverCardPart, HoverMessage, HoverStat},
            target::{HoverTarget, TargetElement},
        },
        popover::{Arrow, Popover},
        sheet::Sheet,
        sheet_attach::Attach,
        side_panel::SidePanel,
        skeleton::{Skeleton, SkeletonShape},
        swipe_glue::{SwipeGlue, SwipeOn, use_swipe_glue},
        toast::use_toasts,
        tooltip::Tooltip,
    },
};
pub use crate::edit::{
    handle::{EditHandle, use_edit_handle},
    input::{Composition, EditInput, KeyInput, PreeditCursor},
    pointer::{EditFocus, EditPointer, Extend},
};
pub use crate::file_drop::{
    board::FileDropBoard,
    drag::{DropAcceptance, FileDrag, FileDragInput, FileDrop, Offer},
    hook::use_file_drop,
};
pub use crate::focus::{
    field::{FieldHandle, use_field_handle},
    request::{FocusRequest, use_focus_request},
    select::Select,
    selector::{FocusError, focus_by_selector},
    soon::focus_soon,
};
pub use crate::host::{
    captured::{CapturedPointer, PointerPhase},
    caret::{Caret, Collapsed, FieldSelection, InitialCaret, caret_at},
    document::{DocumentHost, use_document_host},
    drop_hit::DropHit,
    fallback::Fallback,
    focused::Focused,
    found::{Found, SameNode},
    hand_back::{HandBack, Record},
    ime::{ImeEvent, ImeListener, ImeSwitch},
    measure::{Anchor, Measured, MountedRef, RectProbe, use_rect},
    no_host::NoHost,
    parts::{CaretHost, ClickFocusHost, EditHost, FileDropHost, FocusHost, GeometryHost, ImeHost},
    pasted::Pasted,
    position::{EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditNode, TextPosition, TextRange},
    probe::Probe,
    reveal::{ScrollSpan, Scrolled, nearest_scroll},
    signals::HostSignals,
};
pub use crate::root::{
    chrome::{FrameTint, Ground, RootChrome},
    common::Common,
    extent::RootExtent,
    pass_through::{DataAttr, DataName, ExtraClass, PassThroughError},
    surface::Surface,
    typeface::use_typeface,
};
pub use crate::spell::{
    lang::{Lang, Spell},
    marks::SpellReplace,
    service::{Learned, Paragraph, SpellFuture, SpellService},
    words::WordSpan,
};
pub use crate::stack::{
    host::{OverlayId, use_overlays},
    hover_hub::{HoverKey, HoverKind, use_hover_hub},
    layer_stack::{Dismissal, LayerId, LayerStack},
    menu_track::types::{
        Branch, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming,
        MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, ShownBy,
    },
    roving::{Rove, Roving, Wrap},
    toast_hub::{ToastState, UndoToken, use_toast_hub},
    typeahead::Typeahead,
};
pub use crate::window::{
    host::{HostWindow, WindowHost, use_window_host_provider},
    vocab::{
        Activation, Fullscreen, Maximized, ResizeEdge, Support, TileError, WindowState, WindowTile,
        Zoom,
    },
};
pub use ds_core::word::Word;
pub use ds_core::{
    colour::contrast::{Verdict, ratio},
    geometry::{
        placement::{Align, Flip, Placed, Placement, Side, place},
        scale::Scale,
        units::{Point, Px, Rect, Size},
    },
    press::{PointerButton, Press},
    spawner::Spawner,
    standard_action::{Reserved, SpaceNumber, StandardAction},
    text::clip::clip_chars,
    time::{
        FRAME_SLACK, FRAME_TICK,
        clock::{ClockGuard, VirtualClock, sleep},
    },
    vocab::{
        Activity, Availability, Check, Dismiss, DropState, Emphasis, FocusStyle, Fraction,
        InputModality, Muting, Percent, PressPhase, RowState, Selection, Shortcut, ShortcutKey,
        Shown,
    },
};
pub use ds_motion::{
    anim::Anim,
    drag::{DRAG_THRESHOLD, Drag, DragPhase, DragTracker, WINDOW_DRAG_THRESHOLD, use_drag},
    hover_intent::{HoverEvent, HoverIntent, HoverProfile, HoverWarmth, IntentEffect, IntentPhase},
    long_press::{LONG_PRESS_SLOP, LongPress, LongPressEffect, LongPressEvent},
    pane_slide::Pane,
    presence::{Exit, Presence},
    pulse_key::{PulseKey, PulsePhase},
    recipe::{Fill, Iteration},
    roster::{Heal, RosterState, RowPitch, StayError, Stayed},
    rubber::{RUBBER_SHARE, resist},
    settle::settle,
    swipe::{Speed, SwipeMetrics},
    timer::{TimerPhase, use_motion_timer},
    use_collapse::{Collapse, use_collapse},
    use_roster::{LeaveBy, Pitches, Roster, RosterSpec, use_roster},
    wake::WakeStamp,
};
pub use ds_style::{
    appearance::{
        accent::Accent,
        appearance::Appearance,
        blur::BlurState,
        material::Material,
        motion::{Motion, MotionLevel},
        peek::PeekMode,
        resolve::{Resolved, resolve},
        system::{Contrast, ReducedMotion, SystemPrefs},
        theme::{Scheme, Theme},
        typeface::Typeface,
    },
    fonts::{FACES, Face, FaceStyle, Subset},
    icon::{
        Icon,
        classify::ChromaLimit,
        family::PlateFamily,
        plate_tint::PlateTint,
        render::{Glyph, GlyphProps, IconPx, IconSize},
        shape::Shape,
        url::IconUrl,
    },
    kit::{Kit, KitRank, Kits, KnownNames, Section, Vocabulary},
    look::Look,
    material::{recipe::recipe, stack::MaterialStack},
    scale::use_scale,
    scope::{Scope, use_scope},
    space::{
        frame_vars::FrameVars,
        look::{CardAccent, Grain, SpaceLook},
        palette::{Capping, Dot, derive, readout::readout, swatch},
        presets::{PRESETS, default_look},
        store::{SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex},
    },
    tokens::{
        accent_table::accent_of,
        colour::ColourToken,
        control_size::{ControlSize, SidebarSize},
        delay::DelayToken,
        easing::{Easing, EasingToken},
        elevation::Shadow,
        hex::{Alpha, Colour, Hex},
        label_hue::{HueColour, HueMember, LabelHue},
        layer::ZLayer,
        name::VarName,
        person::PersonSwatch,
        pixel::PixelToken,
        set::{Only, Place, TokenSet},
        shape::{Corner, Radius},
        shell_scale::{SHELL_SCALE, ShellSize},
        shell_type::{FontWeight, ShellMetrics, ShellType},
        size_scale::WholePx,
        spacing::SpacingToken,
        status::StatusMetrics,
        timing::DurationToken,
        token::{CssValue, Token, TokenKind, TokenScope},
        type_scale::{Family, FontSize},
        type_voice::VoiceToken,
        widget_paint::WidgetPaint,
    },
};
