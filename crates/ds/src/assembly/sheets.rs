//! Every component stylesheet, registered once in the cascade's fixed order: the components,
//! then the details, then the widgets. The stylesheet appends them in this order, and a test
//! reads the list to find one sheet by name.

/// `(sheet name, css)`, in cascade order.
pub(crate) const SHEETS: [(&str, &str); 86] = [
    (
        "account_tile",
        include_str!("../components/account_tile.css"),
    ),
    ("alert", include_str!("../components/alert.css")),
    (
        "animated_list",
        include_str!("../components/animated_list.css"),
    ),
    (
        "appearance_picker",
        include_str!("../components/appearance_picker.css"),
    ),
    (
        "app_switcher",
        include_str!("../components/app_switcher.css"),
    ),
    ("avatar", include_str!("../components/avatar.css")),
    (
        "banner_stack",
        include_str!("../components/banner_stack.css"),
    ),
    (
        "battery_level",
        include_str!("../components/battery_level.css"),
    ),
    ("button", include_str!("../components/button.css")),
    ("chip", include_str!("../components/chip.css")),
    ("clock_face", include_str!("../components/clock_face.css")),
    (
        "command_palette",
        include_str!("../components/command_palette.css"),
    ),
    (
        "command_pill",
        include_str!("../components/command_pill.css"),
    ),
    ("count", include_str!("../components/count.css")),
    (
        "device_glyph",
        include_str!("../components/device_glyph.css"),
    ),
    ("dock_parts", include_str!("../components/dock_parts.css")),
    ("drag_ghost", include_str!("../components/drag_ghost.css")),
    ("edge_strip", include_str!("../components/edge_strip.css")),
    (
        "edit_surface",
        include_str!("../components/edit_surface.css"),
    ),
    ("emoji", include_str!("../components/emoji.css")),
    ("emoji_grid", include_str!("../components/emoji_grid.css")),
    (
        "group_header",
        include_str!("../components/group_header.css"),
    ),
    ("hover_card", include_str!("../components/hover_card.css")),
    ("hover_strip", include_str!("../components/hover_strip.css")),
    ("icon_button", include_str!("../components/icon_button.css")),
    ("icon_view", include_str!("../components/icon_view.css")),
    ("idle_dim", include_str!("../components/idle_dim.css")),
    ("kbd", include_str!("../components/kbd.css")),
    ("chord", include_str!("../components/chord.css")),
    ("level", include_str!("../components/level.css")),
    ("link_pill", include_str!("../components/link_pill.css")),
    (
        "leaving_list",
        include_str!("../components/leaving_list.css"),
    ),
    ("list_row", include_str!("../components/list_row.css")),
    ("lock_screen", include_str!("../components/lock_screen.css")),
    ("lock_clock", include_str!("../components/lock_clock.css")),
    ("lock_prompt", include_str!("../components/lock_prompt.css")),
    ("menu", include_str!("../components/menu.css")),
    (
        "menu_bar_item",
        include_str!("../components/menu_bar_item.css"),
    ),
    ("menu_entry", include_str!("../components/menu_entry.css")),
    (
        "notification_card",
        include_str!("../components/notification_card.css"),
    ),
    ("osd", include_str!("../components/osd.css")),
    ("module_tile", include_str!("../components/module_tile.css")),
    (
        "module_panel",
        include_str!("../components/module_panel.css"),
    ),
    ("month_grid", include_str!("../components/month_grid.css")),
    (
        "pane_switcher",
        include_str!("../components/pane_switcher.css"),
    ),
    ("panel", include_str!("../components/panel.css")),
    ("peek", include_str!("../components/peek.css")),
    ("pdf_thumb", include_str!("../components/pdf_thumb.css")),
    (
        "user_picture",
        include_str!("../components/user_picture.css"),
    ),
    ("popover", include_str!("../components/popover.css")),
    (
        "polkit_prompt",
        include_str!("../components/polkit_prompt.css"),
    ),
    (
        "preview_pane",
        include_str!("../components/preview_pane.css"),
    ),
    (
        "provider_mark",
        include_str!("../components/provider_mark.css"),
    ),
    ("scrim", include_str!("../components/scrim.css")),
    (
        "search_field",
        include_str!("../components/search_field.css"),
    ),
    (
        "section_header",
        include_str!("../components/section_header.css"),
    ),
    ("segmented", include_str!("../components/segmented.css")),
    (
        "selection_bubble",
        include_str!("../components/selection_bubble.css"),
    ),
    ("send_pill", include_str!("../components/send_pill.css")),
    (
        "settings_row",
        include_str!("../components/settings_row.css"),
    ),
    ("now_playing", include_str!("../components/now_playing.css")),
    ("sheet", include_str!("../components/sheet.css")),
    (
        "shot_thumbnail",
        include_str!("../components/shot_thumbnail.css"),
    ),
    (
        "sidebar_item",
        include_str!("../components/sidebar_item.css"),
    ),
    ("slider", include_str!("../components/slider.css")),
    (
        "space_editor",
        include_str!("../components/space_editor.css"),
    ),
    ("spinner", include_str!("../components/spinner.css")),
    (
        "status_glyph",
        include_str!("../components/status_glyph.css"),
    ),
    ("sync_halo", include_str!("../components/sync_halo.css")),
    ("tabs", include_str!("../components/tabs.css")),
    ("text_input", include_str!("../components/text_input.css")),
    ("text_runs", include_str!("../components/text_runs.css")),
    ("toast", include_str!("../components/toast.css")),
    ("toggle", include_str!("../components/toggle.css")),
    ("tooltip", include_str!("../components/tooltip.css")),
    ("tree_item", include_str!("../components/tree_item.css")),
    (
        "widget_frame",
        include_str!("../components/widget_frame.css"),
    ),
    (
        "window_frame",
        include_str!("../components/window_frame.css"),
    ),
    (
        "workspace_pills",
        include_str!("../components/workspace_pills.css"),
    ),
    // Last: the drop states SidebarItem and TreeItem share must win over either
    // item's hover and current rules, which have the same specificity.
    ("drop_place", include_str!("../components/drop_place.css")),
    ("detail_reveal", include_str!("../detail/reveal.css")),
    ("detail_layer", include_str!("../detail/layer.css")),
    ("detail_morph", include_str!("../detail/morph.css")),
    ("detail_roll", include_str!("../detail/roll.css")),
    ("widget_views", include_str!("../widget/views.css")),
    ("widget_gallery", include_str!("../widget/gallery.css")),
];
