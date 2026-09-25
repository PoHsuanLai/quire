//! What the app calls a frame. A `FrameId` names a Blitz document, which only ds-native sees
//! being born; the app knows its `iframe` element (which message it shows). The app writes
//! `data-frame-tag="<text>"` on the element, and ds-native reads it as the frame's document is
//! attached, so every request and link click from that document carries the app's own name.

use std::fmt;

/// The attribute an app puts on its `iframe` to name the frame.
pub(crate) const TAG_ATTRIBUTE: &str = "data-frame-tag";

/// The app's name for one frame, from its `iframe`'s `data-frame-tag`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameTag(String);

impl FrameTag {
    /// The tag `text` names: what an app looks a frame up by with
    /// [`crate::frames::frame_by_tag`].
    pub fn new(text: impl Into<String>) -> Self {
        FrameTag(text.into())
    }

    /// The tag an attribute's `value` gives, trimmed; a blank or missing one is no tag, so an
    /// untagged frame never matches a lookup by the empty string.
    pub(crate) fn read(value: Option<&str>) -> Option<Self> {
        value
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(FrameTag::new)
    }

    /// The tag's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FrameTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::FrameTag;

    const CASES: &[(Option<&str>, Option<&str>)] = &[
        (None, None),
        (Some(""), None),
        (Some("  "), None),
        (Some("msg-42"), Some("msg-42")),
        (Some(" msg-42\n"), Some("msg-42")),
    ];

    #[test]
    fn an_attribute_reads_as_its_trimmed_tag() {
        for &(value, want) in CASES {
            assert_eq!(
                FrameTag::read(value).as_ref().map(FrameTag::as_str),
                want,
                "{value:?}"
            );
        }
    }
}
