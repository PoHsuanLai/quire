//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).
//! Separate targets, each with its reason: `pdf_thumb` and `pdf_thumb_queue` share the process-wide
//! PDF worker queue and its cache with each other and would count each other's rasters;
//! `spell_worker` and `spell_edit` start the process-wide spell service on their own dictionaries.
//! All four also have `required-features` in Cargo.toml.

mod cancelled_animation;
mod caret_same_frame;
mod cascade_layer;
mod click_focus_restore;
mod clipboard;
mod colour_emoji;
mod css_filter;
mod edit_format_bar_focus;
mod edit_stacking;
mod edit_surface;
mod edit_surface_pointer;
mod field_handle_focus;
mod file_drop;
mod focus_select;
mod frame_link_menu;
mod frame_link_text;
mod frame_links;
mod frame_phase;
mod frame_tags;
mod frames;
mod hit_absolute_in_padded_parent;
mod hit_outside_empty_parent;
mod keep_focus;
mod kept_click_focus;
mod net_policy;
mod open_window_harness;
mod pdf_app;
mod pdf_output;
mod pdf_raster;
mod pixel_snap;
#[cfg(feature = "debug-probe")]
mod probe_snapshot;
mod provide_host;
mod registered_property_animation;
mod removed_focus;
mod root_contexts;
mod selection_same_frame;
mod strip_hit_area;
mod support;
mod text_stack_fonts;
mod user_style_reload;
mod window_scroll;
mod window_scroll_fingers;
mod window_scroll_frames;
mod window_scroll_same_frame;
