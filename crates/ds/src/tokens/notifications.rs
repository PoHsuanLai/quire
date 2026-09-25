//! The notification parts' geometry tokens (sill Q120-Q123; design/22-SETTINGS.md section 3.12,
//! design/13 section 13.3.6): the banner's width, floor, padding and icon, the close button, the
//! gap between stacked banners, the offset of a group's layers and the notification center's
//! width, each a [`Tuned`] token its `notifications.*` key moves through one inline write
//! ([`NotificationMetrics::style_attr`]) on any element around the banners or the center.

use super::name::VarName;
use super::tuned::{Tuned, px};
use crate::geometry::Px;

const fn tuned(token: &'static str, input: &'static str, default: &'static str) -> Tuned {
    Tuned {
        token: VarName(token),
        input: VarName(input),
        default,
    }
}

/// `--notifications-banner-width` (`notifications.banner_width_px`, 360).
pub const BANNER_WIDTH: Tuned = tuned(
    "--notifications-banner-width",
    "--notifications-banner-width-px",
    "360px",
);
/// `--notifications-banner-min-height` (`notifications.banner_min_height_px`, 64).
pub const BANNER_MIN_HEIGHT: Tuned = tuned(
    "--notifications-banner-min-height",
    "--notifications-banner-min-height-px",
    "64px",
);
/// `--notifications-banner-padding` (`notifications.banner_padding_px`, 12).
pub const BANNER_PADDING: Tuned = tuned(
    "--notifications-banner-padding",
    "--notifications-banner-padding-px",
    "12px",
);
/// `--notifications-icon` (`notifications.icon_px`, 32): the app icon's side.
pub const ICON: Tuned = tuned("--notifications-icon", "--notifications-icon-px", "32px");
/// `--notifications-close` (`notifications.close_button_px`, 18): the close button's diameter.
pub const CLOSE: Tuned = tuned("--notifications-close", "--notifications-close-px", "18px");
/// `--notifications-stack-gap` (`notifications.stack_gap_px`, 8): between two banners.
pub const STACK_GAP: Tuned = tuned(
    "--notifications-stack-gap",
    "--notifications-stack-gap-px",
    "8px",
);
/// `--notifications-group-offset` (`notifications.group_offset_px`, 4): how far each of a
/// group's layers shows below the one above it.
pub const GROUP_OFFSET: Tuned = tuned(
    "--notifications-group-offset",
    "--notifications-group-offset-px",
    "4px",
);
/// `--notifications-center-width` (`notifications.center_width_px`, 384).
pub const CENTER_WIDTH: Tuned = tuned(
    "--notifications-center-width",
    "--notifications-center-width-px",
    "384px",
);

/// Every notification token, in stylesheet order.
pub const NOTIFICATION_TOKENS: [Tuned; 8] = [
    BANNER_WIDTH,
    BANNER_MIN_HEIGHT,
    BANNER_PADDING,
    ICON,
    CLOSE,
    STACK_GAP,
    GROUP_OFFSET,
    CENTER_WIDTH,
];

/// The notification geometry from the settings, written as the tokens' inputs on any element
/// around the banners or the center.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotificationMetrics {
    /// A banner's width, 360.
    pub banner_width: Px,
    /// A banner's least height, 64.
    pub banner_min_height: Px,
    /// A banner's padding, 12.
    pub banner_padding: Px,
    /// The app icon's side, 32.
    pub icon: Px,
    /// The close button's diameter, 18.
    pub close: Px,
    /// Between two stacked banners, 8.
    pub stack_gap: Px,
    /// Each group layer's offset below the one above it, 4.
    pub group_offset: Px,
    /// The notification center's width, 384 (the key's range is 280 to 600).
    pub center_width: Px,
}

impl Default for NotificationMetrics {
    /// The keys' defaults (design/22 section 3.12).
    fn default() -> Self {
        NotificationMetrics {
            banner_width: Px(360.0),
            banner_min_height: Px(64.0),
            banner_padding: Px(12.0),
            icon: Px(32.0),
            close: Px(18.0),
            stack_gap: Px(8.0),
            group_offset: Px(4.0),
            center_width: Px(384.0),
        }
    }
}

impl NotificationMetrics {
    /// Every input, inline: `--notifications-banner-width-px:360px;…`. The center's width is
    /// held to its key's 280 to 600; every other length to 0 to 999.
    pub fn style_attr(&self) -> String {
        let length = |value: Px| px(value.0.round().clamp(0.0, 999.0) as u16);
        [
            BANNER_WIDTH.write(&length(self.banner_width)),
            BANNER_MIN_HEIGHT.write(&length(self.banner_min_height)),
            BANNER_PADDING.write(&length(self.banner_padding)),
            ICON.write(&length(self.icon)),
            CLOSE.write(&length(self.close)),
            STACK_GAP.write(&length(self.stack_gap)),
            GROUP_OFFSET.write(&length(self.group_offset)),
            CENTER_WIDTH.write(&length(Px(self.center_width.0.clamp(280.0, 600.0)))),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::{NOTIFICATION_TOKENS, NotificationMetrics};
    use crate::geometry::Px;

    #[test]
    fn the_defaults_write_what_the_stylesheet_falls_back_to() {
        let written = NotificationMetrics::default().style_attr();
        let want: String = NOTIFICATION_TOKENS
            .iter()
            .map(|token| token.write(token.default))
            .collect();
        assert_eq!(written, want);
    }

    #[test]
    fn the_center_is_held_to_its_keys_range() {
        let wide = NotificationMetrics {
            center_width: Px(900.0),
            ..NotificationMetrics::default()
        };
        assert!(
            wide.style_attr()
                .contains("--notifications-center-width-px:600px;")
        );
        let narrow = NotificationMetrics {
            center_width: Px(100.0),
            ..NotificationMetrics::default()
        };
        assert!(
            narrow
                .style_attr()
                .contains("--notifications-center-width-px:280px;")
        );
    }
}
