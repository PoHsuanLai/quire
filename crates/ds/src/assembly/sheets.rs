//! Every component stylesheet the components own, registered once in the cascade's fixed order: the components,
//! then the details. The stylesheet appends them in this order, and a test
//! reads the list to find one sheet by name.

/// `(sheet name, css)`, in cascade order.
pub(crate) const SHEETS: [(&str, &str); 57] = [
    (
        "account_tile",
        include_str!("../components/app/account_tile.css"),
    ),
    ("alert", include_str!("../components/overlays/alert.css")),
    (
        "appearance_picker",
        include_str!("../components/lists/appearance_picker.css"),
    ),
    ("avatar", include_str!("../components/content/avatar.css")),
    ("badge", include_str!("../components/controls/badge.css")),
    ("button", include_str!("../components/controls/button.css")),
    (
        "checkbox",
        include_str!("../components/controls/checkbox.css"),
    ),
    ("chip", include_str!("../components/controls/chip.css")),
    (
        "command_palette",
        include_str!("../components/menus/palette/command_palette.css"),
    ),
    (
        "command_pill",
        include_str!("../components/app/command_pill.css"),
    ),
    (
        "disclosure",
        include_str!("../components/controls/disclosure.css"),
    ),
    (
        "drag_ghost",
        include_str!("../components/overlays/drag_ghost.css"),
    ),
    (
        "edge_strip",
        include_str!("../components/app/edge_strip.css"),
    ),
    (
        "edit_surface",
        include_str!("../components/editor/surface.css"),
    ),
    (
        "emoji_grid",
        include_str!("../components/lists/emoji_grid/grid.css"),
    ),
    (
        "empty_state",
        include_str!("../components/overlays/empty_state.css"),
    ),
    (
        "hover_card",
        include_str!("../components/overlays/hover_card.css"),
    ),
    (
        "hover_strip",
        include_str!("../components/app/hover_strip.css"),
    ),
    (
        "icon_view",
        include_str!("../components/content/icon_view.css"),
    ),
    (
        "key_equivalent",
        include_str!("../components/controls/key_equivalent.css"),
    ),
    ("label", include_str!("../components/content/label.css")),
    (
        "level_glyph",
        include_str!("../components/content/level_glyph/glyph.css"),
    ),
    (
        "level_indicator",
        include_str!("../components/controls/level_indicator.css"),
    ),
    ("link_pill", include_str!("../components/app/link_pill.css")),
    ("list", include_str!("../components/lists/list/list.css")),
    ("menu", include_str!("../components/menus/menu/menu.css")),
    (
        "menu_item",
        include_str!("../components/menus/item/item.css"),
    ),
    (
        "pane_switcher",
        include_str!("../components/lists/preview/switcher.css"),
    ),
    (
        "radio_group",
        include_str!("../components/controls/radio_group.css"),
    ),
    ("peek", include_str!("../components/app/peek.css")),
    (
        "pdf_thumb",
        include_str!("../components/content/pdf_thumb.css"),
    ),
    (
        "popover",
        include_str!("../components/overlays/popover.css"),
    ),
    (
        "pop_up_button",
        include_str!("../components/menus/pop_up_button.css"),
    ),
    (
        "preview_pane",
        include_str!("../components/lists/preview/pane.css"),
    ),
    (
        "provider_mark",
        include_str!("../components/content/provider_mark.css"),
    ),
    ("row", include_str!("../components/lists/row/row.css")),
    ("scrim", include_str!("../components/overlays/scrim.css")),
    (
        "section_header",
        include_str!("../components/lists/section_header.css"),
    ),
    (
        "segmented",
        include_str!("../components/controls/segmented.css"),
    ),
    (
        "selection_bubble",
        include_str!("../components/fields/selection_bubble.css"),
    ),
    ("send_pill", include_str!("../components/app/send_pill.css")),
    ("sheet", include_str!("../components/overlays/sheet.css")),
    (
        "side_panel",
        include_str!("../components/overlays/side_panel.css"),
    ),
    (
        "skeleton",
        include_str!("../components/overlays/skeleton.css"),
    ),
    ("slider", include_str!("../components/controls/slider.css")),
    (
        "progress",
        include_str!("../components/controls/progress/progress.css"),
    ),
    (
        "spinner",
        include_str!("../components/controls/spinner.css"),
    ),
    (
        "status_glyph",
        include_str!("../components/content/status_glyph.css"),
    ),
    (
        "text_field",
        include_str!("../components/fields/text_field.css"),
    ),
    (
        "text_runs",
        include_str!("../components/content/text_runs.css"),
    ),
    (
        "thread_row",
        include_str!("../components/app/thread_row.css"),
    ),
    ("toast", include_str!("../components/overlays/toast.css")),
    ("toggle", include_str!("../components/controls/toggle.css")),
    (
        "tooltip",
        include_str!("../components/overlays/tooltip.css"),
    ),
    (
        "voice_orb",
        include_str!("../components/content/voice_orb/style.css"),
    ),
    (
        "window_frame",
        include_str!("../components/chrome/window_frame.css"),
    ),
    ("detail_morph", ds_motion::detail::morph::CSS),
];
