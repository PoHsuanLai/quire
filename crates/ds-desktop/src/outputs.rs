//! The shell's per-output work areas, as an app reads them (`org.quire.Outputs1`, capability
//! [`Capability::Outputs`](crate::Capability::Outputs)): each output's frame, its work area (the
//! frame less the bar's and dock's exclusive zones) and its scale. The data and its lookups are
//! plain and always built; reading the bus is [`Outputs::read`] (feature `dbus`).
//!
//! The portable fallback is the empty list, which is what everything yields without the shell:
//! the caller then keeps the window system's own output size (`ds_blitz::ScreenArea`).

/// A rectangle in the compositor's logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OutputRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// One output: what `NSScreen` calls `frame`, `visibleFrame` and `backingScaleFactor`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputArea {
    /// The shell's id for the output, stable while it is connected.
    pub id: u32,
    /// The connector name (`DP-1`); empty when the compositor sent none.
    pub name: String,
    /// The whole output, in logical pixels.
    pub frame: OutputRect,
    /// The part a window may use.
    pub work: OutputRect,
    /// Device pixels per logical pixel, in 120ths (120 is 1x, 180 is 1.5x).
    pub scale: u32,
}

/// The wire row, `(us(iiuu)(iiuu)u)`.
pub(crate) type Wire = (u32, String, (i32, i32, u32, u32), (i32, i32, u32, u32), u32);

fn rect((x, y, width, height): (i32, i32, u32, u32)) -> OutputRect {
    OutputRect {
        x,
        y,
        width,
        height,
    }
}

impl OutputArea {
    /// The output's size in device pixels: the frame times the scale, rounded.
    pub fn physical(&self) -> (u32, u32) {
        let device = |logical: u32| {
            let pixels = (u64::from(logical) * u64::from(self.scale) + 60) / 120;
            u32::try_from(pixels).unwrap_or(u32::MAX)
        };
        (device(self.frame.width), device(self.frame.height))
    }
}

/// Every output the shell reports, in its order. Empty means no shell answered.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outputs {
    areas: Vec<OutputArea>,
}

impl Outputs {
    /// Outputs from already-known areas.
    pub fn new(areas: Vec<OutputArea>) -> Outputs {
        Outputs { areas }
    }

    /// Outputs from the property's wire rows.
    #[cfg_attr(not(feature = "dbus"), allow(dead_code))]
    pub(crate) fn of_wire(rows: Vec<Wire>) -> Outputs {
        Outputs {
            areas: rows
                .into_iter()
                .map(|(id, name, frame, work, scale)| OutputArea {
                    id,
                    name,
                    frame: rect(frame),
                    work: rect(work),
                    scale,
                })
                .collect(),
        }
    }

    /// The outputs, in the shell's order.
    pub fn areas(&self) -> &[OutputArea] {
        &self.areas
    }

    /// The output called `name`.
    pub fn named(&self, name: &str) -> Option<&OutputArea> {
        self.areas.iter().find(|area| area.name == name)
    }

    /// The output whose device-pixel size is `physical` (within a pixel on each axis, since the
    /// logical frame is rounded). Outputs of the same size are one answer when they agree on the
    /// work area's size and the scale (two identical monitors with the same bar), and no answer
    /// when they do not.
    pub fn of_physical(&self, physical: (u32, u32)) -> Option<&OutputArea> {
        let near = |a: u32, b: u32| a.abs_diff(b) <= 1;
        let mut matching = self.areas.iter().filter(|area| {
            let (width, height) = area.physical();
            near(width, physical.0) && near(height, physical.1)
        });
        let first = matching.next()?;
        let same = |other: &OutputArea| {
            (other.work.width, other.work.height, other.scale)
                == (first.work.width, first.work.height, first.scale)
        };
        matching.all(same).then_some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(name: &str, frame: (u32, u32), top: u32, scale: u32) -> OutputArea {
        OutputArea {
            id: 1,
            name: name.to_owned(),
            frame: OutputRect {
                x: 0,
                y: 0,
                width: frame.0,
                height: frame.1,
            },
            work: OutputRect {
                x: 0,
                y: i32::try_from(top).unwrap_or(0),
                width: frame.0,
                height: frame.1 - top,
            },
            scale,
        }
    }

    #[test]
    fn the_physical_size_is_the_frame_times_the_scale() {
        // name, frame, scale, physical
        let cases = [
            ("1x", (1920, 1080), 120, (1920, 1080)),
            ("2x", (1920, 1080), 240, (3840, 2160)),
            ("1.5x", (2560, 1440), 180, (3840, 2160)),
            ("1.25x rounds", (2048, 1152), 150, (2560, 1440)),
        ];
        for (name, frame, scale, want) in cases {
            assert_eq!(area("", frame, 0, scale).physical(), want, "{name}");
        }
    }

    #[test]
    fn an_output_is_found_by_its_pixels() {
        let outputs = Outputs::new(vec![
            area("DP-1", (2560, 1440), 32, 180),
            area("HDMI-1", (1920, 1080), 32, 120),
        ]);
        // name, physical, found
        let cases = [
            ("4k", (3840, 2160), Some("DP-1")),
            ("one pixel off", (3841, 2160), Some("DP-1")),
            ("1080p", (1920, 1080), Some("HDMI-1")),
            ("unknown", (800, 600), None),
        ];
        for (name, physical, want) in cases {
            let got = outputs.of_physical(physical).map(|a| a.name.as_str());
            assert_eq!(got, want, "{name}");
        }
        assert!(outputs.named("DP-1").is_some());
        assert!(outputs.named("DP-9").is_none());
    }

    #[test]
    fn identical_outputs_are_one_answer_only_when_they_agree() {
        let same = Outputs::new(vec![
            area("A", (1920, 1080), 32, 120),
            area("B", (1920, 1080), 32, 120),
        ]);
        assert_eq!(
            same.of_physical((1920, 1080)).map(|a| a.work.height),
            Some(1048)
        );
        let differ = Outputs::new(vec![
            area("A", (1920, 1080), 32, 120),
            area("B", (1920, 1080), 100, 120),
        ]);
        assert_eq!(differ.of_physical((1920, 1080)), None);
    }

    #[test]
    fn no_shell_is_an_empty_list() {
        assert_eq!(Outputs::default().of_physical((1920, 1080)), None);
        assert!(Outputs::default().areas().is_empty());
    }

    #[test]
    fn wire_rows_become_areas() {
        let rows = vec![(
            3,
            "DP-2".to_owned(),
            (0, 0, 2560, 1440),
            (0, 32, 2560, 1340),
            180,
        )];
        let outputs = Outputs::of_wire(rows);
        assert_eq!(outputs.areas(), &[area_with(3, "DP-2", 32, 1340)]);
    }

    fn area_with(id: u32, name: &str, top: i32, height: u32) -> OutputArea {
        OutputArea {
            id,
            name: name.to_owned(),
            frame: OutputRect {
                x: 0,
                y: 0,
                width: 2560,
                height: 1440,
            },
            work: OutputRect {
                x: 0,
                y: top,
                width: 2560,
                height,
            },
            scale: 180,
        }
    }
}
