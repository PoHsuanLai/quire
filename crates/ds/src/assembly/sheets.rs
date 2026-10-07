//! Every component stylesheet the components own, registered once in the cascade's fixed order: the components,
//! then the details. The stylesheet appends them in this order, and a test
//! reads the list to find one sheet by name.

/// `(sheet name, css)`, in cascade order.
pub(crate) const SHEETS: [(&str, &str); 88] = [
    ("alert", include_str!("../components/overlays/alert.css")),
    ("avatar", include_str!("../components/content/avatar.css")),
    ("badge", include_str!("../components/controls/badge.css")),
    ("button", include_str!("../components/controls/button.css")),
    (
        "capsule",
        include_str!("../components/chrome/capsule/capsule.css"),
    ),
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
    ("edge_peek", include_str!("../components/app/edge_peek.css")),
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
        "fact_list",
        include_str!("../components/fields/fact_list.css"),
    ),
    (
        "field_row",
        include_str!("../components/fields/field_row.css"),
    ),
    ("form", include_str!("../components/forms/form.css")),
    (
        "form_section",
        include_str!("../components/forms/form_section.css"),
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
        "icon_tile",
        include_str!("../components/forms/icon_tile.css"),
    ),
    (
        "icon_view",
        include_str!("../components/content/icon_view.css"),
    ),
    (
        "inline_banner",
        include_str!("../components/overlays/inline_banner.css"),
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
    (
        "loadable",
        include_str!("../components/overlays/loadable.css"),
    ),
    ("menu", include_str!("../components/menus/menu/menu.css")),
    (
        "menu_item",
        include_str!("../components/menus/item/item.css"),
    ),
    (
        "search_card",
        include_str!("../components/menus/search/search.css"),
    ),
    (
        "pane_stack",
        include_str!("../components/forms/pane_stack/pane_stack.css"),
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
    ("pin_tile", include_str!("../components/app/pin_tile.css")),
    (
        "pick_list",
        include_str!("../components/menus/pick_list.css"),
    ),
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
    ("row_more", include_str!("../components/app/row_more.css")),
    ("scrim", include_str!("../components/overlays/scrim.css")),
    (
        "scroller",
        include_str!("../components/controls/scroller/scroller.css"),
    ),
    (
        "section_header",
        include_str!("../components/lists/section_header.css"),
    ),
    (
        "segmented",
        include_str!("../components/controls/segmented.css"),
    ),
    ("send_pill", include_str!("../components/app/send_pill.css")),
    ("sheet", include_str!("../components/overlays/sheet.css")),
    (
        "side_panel",
        include_str!("../components/overlays/side_panel.css"),
    ),
    ("sidebar", include_str!("../components/chrome/sidebar.css")),
    (
        "skeleton",
        include_str!("../components/overlays/skeleton.css"),
    ),
    (
        "skeleton_row",
        include_str!("../components/overlays/skeleton_row.css"),
    ),
    ("slider", include_str!("../components/controls/slider.css")),
    (
        "scrubber",
        include_str!("../components/controls/scrubber.css"),
    ),
    (
        "space_editor",
        include_str!("../components/app/space_editor.css"),
    ),
    (
        "spaces",
        include_str!("../components/app/spaces/spaces.css"),
    ),
    (
        "progress",
        include_str!("../components/controls/progress/progress.css"),
    ),
    (
        "split_view",
        include_str!("../components/chrome/split_view/split_view.css"),
    ),
    (
        "stepper",
        include_str!("../components/fields/stepper/stepper.css"),
    ),
    (
        "status_glyph",
        include_str!("../components/content/status_glyph.css"),
    ),
    (
        "tab_view",
        include_str!("../components/chrome/tab_view.css"),
    ),
    ("table", include_str!("../components/lists/table/table.css")),
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
        "toolbar",
        include_str!("../components/chrome/toolbar/toolbar.css"),
    ),
    (
        "tooltip",
        include_str!("../components/overlays/tooltip.css"),
    ),
    (
        "virtual_list",
        include_str!("../components/lists/virtual_list/virtual_list.css"),
    ),
    (
        "voice_orb",
        include_str!("../components/content/voice_orb/style.css"),
    ),
    (
        "window_frame",
        include_str!("../components/chrome/window_frame.css"),
    ),
    (
        "companion_orb",
        include_str!("../components/companion/orb/style.css"),
    ),
    (
        "context_chips",
        include_str!("../components/companion/chips/style.css"),
    ),
    (
        "answer_card",
        include_str!("../components/companion/answer/style.css"),
    ),
    (
        "plan_list",
        include_str!("../components/companion/plan/style.css"),
    ),
    (
        "replace_bar",
        include_str!("../components/companion/replace/style.css"),
    ),
    (
        "run_row",
        include_str!("../components/companion/run_row/style.css"),
    ),
    (
        "activity_strip",
        include_str!("../components/companion/activity/style.css"),
    ),
    (
        "memory_timeline",
        include_str!("../components/companion/memory/style.css"),
    ),
    (
        "served_by_chip",
        include_str!("../components/companion/served_by/style.css"),
    ),
    ("detail_morph", ds_motion::detail::morph::CSS),
    ("symbol", ds_motion::symbol::CSS),
];
