//! Resting opacities a keyframe ends on, named so a keyframe and the element that rests there
//! agree (mailo gaps 3).

use super::name::VarName;

/// One resting opacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpacityToken {
    /// `--veil` .16: an ink layer laid over content to push it back (C's scrim, `C:1055`: `ink`
    /// at .16), where `fade-in` ends. Not `--scrim`, which is a colour (black at .22) painted at
    /// full opacity and faded by `fade`.
    Veil,
}

impl OpacityToken {
    /// Every opacity token, in stylesheet order.
    pub const ALL: [OpacityToken; 1] = [OpacityToken::Veil];

    /// The custom property: `--veil`.
    pub fn var(self) -> VarName {
        VarName(match self {
            OpacityToken::Veil => "--veil",
        })
    }

    /// The opacity in thousandths: 160 is .16.
    pub fn thousandths(self) -> u16 {
        match self {
            OpacityToken::Veil => 160,
        }
    }

    /// The CSS text: `.16`.
    pub fn css(self) -> String {
        super::hex::thousandths(i64::from(self.thousandths()))
    }
}

#[cfg(test)]
mod tests {
    use super::OpacityToken;

    #[test]
    fn the_veil_is_c_s_scrim_opacity() {
        assert_eq!(OpacityToken::Veil.var().as_str(), "--veil");
        assert_eq!(OpacityToken::Veil.css(), ".16");
    }
}
