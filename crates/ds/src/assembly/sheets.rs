//! Every component stylesheet, registered once in the cascade's fixed order: the components,
//! then the details, then the widgets. The stylesheet appends them in this order, and a test
//! reads the list to find one sheet by name.

/// `(sheet name, css)`, in cascade order.
pub(crate) const SHEETS: [(&str, &str); 86] = [
    (
        "account_tile",
        include_str!("../components/app/account_tile.css"),
    ),
    ("alert", include_str!("../components/overlays/alert.css")),
    (
        "animated_list",
        include_str!("../components/lists/animated_list.css"),
    ),
    (
        "appearance_picker",
        include_str!("../components/lists/appearance_picker.css"),
    ),
    (
        "app_switcher",
        include_str!("../shell/switcher/app_switcher.css"),
    ),
    ("avatar", include_str!("../components/content/avatar.css")),
    (
        "banner_stack",
        include_str!("../shell/notifications/banner_stack.css"),
    ),
    ("battery_level", include_str!("../shell/battery/level.css")),
    ("button", include_str!("../components/controls/button.css")),
    ("chip", include_str!("../components/controls/chip.css")),
    ("clock_face", include_str!("../shell/clock/face.css")),
    (
        "command_palette",
        include_str!("../components/menus/palette/command_palette.css"),
    ),
    (
        "command_pill",
        include_str!("../components/app/command_pill.css"),
    ),
    ("count", include_str!("../components/controls/count.css")),
    (
        "device_glyph",
        include_str!("../shell/battery/device_glyph.css"),
    ),
    ("dock_parts", include_str!("../shell/dock_parts.css")),
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
    ("emoji", include_str!("../shell/emoji.css")),
    (
        "emoji_grid",
        include_str!("../components/lists/emoji_grid/grid.css"),
    ),
    (
        "group_header",
        include_str!("../shell/notifications/group_header.css"),
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
        "icon_button",
        include_str!("../components/controls/icon_button.css"),
    ),
    (
        "icon_view",
        include_str!("../components/content/icon_view.css"),
    ),
    ("idle_dim", include_str!("../shell/idle_dim.css")),
    ("kbd", include_str!("../components/controls/kbd.css")),
    ("chord", include_str!("../components/controls/chord.css")),
    ("level", include_str!("../components/controls/level.css")),
    ("link_pill", include_str!("../components/app/link_pill.css")),
    (
        "leaving_list",
        include_str!("../components/lists/leaving_list.css"),
    ),
    ("list_row", include_str!("../components/lists/list_row.css")),
    ("lock_screen", include_str!("../shell/lock/screen.css")),
    ("lock_clock", include_str!("../shell/lock/clock.css")),
    ("lock_prompt", include_str!("../shell/lock/prompt.css")),
    ("menu", include_str!("../components/menus/menu.css")),
    (
        "menu_bar_item",
        include_str!("../shell/bar/menu_bar_item.css"),
    ),
    (
        "menu_entry",
        include_str!("../components/menus/menu_entry.css"),
    ),
    (
        "notification_card",
        include_str!("../shell/notifications/card.css"),
    ),
    ("osd", include_str!("../shell/osd.css")),
    (
        "module_tile",
        include_str!("../shell/control_center/module_tile.css"),
    ),
    (
        "module_panel",
        include_str!("../shell/control_center/module_panel.css"),
    ),
    ("month_grid", include_str!("../shell/month_grid.css")),
    (
        "pane_switcher",
        include_str!("../components/lists/preview/switcher.css"),
    ),
    ("panel", include_str!("../components/overlays/panel.css")),
    ("peek", include_str!("../components/app/peek.css")),
    (
        "pdf_thumb",
        include_str!("../components/content/pdf_thumb.css"),
    ),
    ("user_picture", include_str!("../shell/user_picture.css")),
    (
        "popover",
        include_str!("../components/overlays/popover.css"),
    ),
    (
        "polkit_prompt",
        include_str!("../shell/lock/polkit_prompt.css"),
    ),
    (
        "preview_pane",
        include_str!("../components/lists/preview/pane.css"),
    ),
    (
        "provider_mark",
        include_str!("../components/content/provider_mark.css"),
    ),
    ("scrim", include_str!("../components/overlays/scrim.css")),
    (
        "search_field",
        include_str!("../components/fields/search_field.css"),
    ),
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
    (
        "settings_row",
        include_str!("../components/lists/settings_row.css"),
    ),
    ("now_playing", include_str!("../shell/now_playing.css")),
    ("sheet", include_str!("../components/overlays/sheet.css")),
    (
        "shot_thumbnail",
        include_str!("../shell/thumbs/shot_thumbnail.css"),
    ),
    (
        "sidebar_item",
        include_str!("../components/app/sidebar_item.css"),
    ),
    ("slider", include_str!("../components/controls/slider.css")),
    ("space_editor", include_str!("../shell/space_editor.css")),
    (
        "spinner",
        include_str!("../components/controls/spinner.css"),
    ),
    (
        "status_glyph",
        include_str!("../components/content/status_glyph.css"),
    ),
    ("sync_halo", include_str!("../components/app/sync_halo.css")),
    ("tabs", include_str!("../components/controls/tabs.css")),
    (
        "text_input",
        include_str!("../components/fields/text_input.css"),
    ),
    (
        "text_runs",
        include_str!("../components/content/text_runs.css"),
    ),
    ("toast", include_str!("../components/overlays/toast.css")),
    ("toggle", include_str!("../components/controls/toggle.css")),
    (
        "tooltip",
        include_str!("../components/overlays/tooltip.css"),
    ),
    ("tree_item", include_str!("../components/app/tree_item.css")),
    ("widget_frame", include_str!("../shell/widget/frame.css")),
    (
        "window_frame",
        include_str!("../components/chrome/window_frame.css"),
    ),
    (
        "workspace_pills",
        include_str!("../shell/bar/workspace_pills.css"),
    ),
    // Last: the drop states SidebarItem and TreeItem share must win over either
    // item's hover and current rules, which have the same specificity.
    (
        "drop_place",
        include_str!("../components/app/drop_place.css"),
    ),
    ("detail_reveal", include_str!("../motion/detail/reveal.css")),
    ("detail_layer", include_str!("../motion/detail/layer.css")),
    ("detail_morph", include_str!("../motion/detail/morph.css")),
    ("detail_roll", include_str!("../motion/detail/roll.css")),
    ("widget_views", include_str!("../shell/widget/views.css")),
    (
        "widget_gallery",
        include_str!("../shell/widget/gallery.css"),
    ),
];
