//! Window chrome: a client-decorated window's frame, its traffic lights and its resize edges, and
//! what fills a window's content (`Toolbar`, `SplitView`, `Sidebar`, `TabView`).

pub mod capsule;
pub(crate) mod light_mark;
pub(crate) mod resize_edges;
pub mod sidebar;
pub mod split_view;
pub mod tab_view;
pub mod titlebar_parts;
pub mod toolbar;
pub mod traffic_lights;
pub mod window_frame;
