/// A single-channel float image (alpha masks, coverage), row-major, values usually in 0..=1.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

impl Plane {
    /// A plane of one value.
    pub fn filled(width: u32, height: u32, value: f32) -> Self {
        Self {
            width,
            height,
            data: vec![value; (width * height) as usize],
        }
    }

    /// A plane computed per pixel from its coordinates.
    pub fn from_fn(width: u32, height: u32, f: impl Fn(u32, u32) -> f32) -> Self {
        let data = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| f(x, y))
            .collect();
        Self {
            width,
            height,
            data,
        }
    }

    /// The value at (x, y); outside the plane it is 0.
    pub fn at(&self, x: i64, y: i64) -> f32 {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return 0.0;
        }
        self.data[(y as usize) * (self.width as usize) + x as usize]
    }

    /// The plane moved by (dx, dy) pixels; uncovered pixels are 0.
    pub fn shifted(&self, dx: i64, dy: i64) -> Self {
        Self::from_fn(self.width, self.height, |x, y| {
            self.at(i64::from(x) - dx, i64::from(y) - dy)
        })
    }

    /// Mean value: the covered fraction for a mask.
    pub fn mean(&self) -> f32 {
        self.data.iter().sum::<f32>() / self.data.len().max(1) as f32
    }

    /// An approximate Gaussian blur: three box passes per axis (the usual 3-box approximation).
    pub fn blurred(&self, sigma: f32) -> Self {
        if sigma < 0.5 {
            return self.clone();
        }
        let radius = box_radius(sigma);
        (0..3).fold(self.clone(), |p, _| {
            p.box_pass(radius, Axis::X).box_pass(radius, Axis::Y)
        })
    }

    fn box_pass(&self, r: usize, axis: Axis) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let (lines, len) = match axis {
            Axis::X => (h, w),
            Axis::Y => (w, h),
        };
        let index = |line: usize, i: usize| match axis {
            Axis::X => line * w + i,
            Axis::Y => i * w + line,
        };
        let mut out = vec![0.0; w * h];
        let norm = 1.0 / (2 * r + 1) as f32;
        for line in 0..lines {
            let get = |i: isize| {
                if i < 0 || i as usize >= len {
                    0.0
                } else {
                    self.data[index(line, i as usize)]
                }
            };
            let mut sum: f32 = (-(r as isize)..=r as isize).map(get).sum();
            for i in 0..len {
                out[index(line, i)] = sum * norm;
                sum += get(i as isize + r as isize + 1) - get(i as isize - r as isize);
            }
        }
        Self {
            width: self.width,
            height: self.height,
            data: out,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

/// Box radius whose three passes match a Gaussian of `sigma` (variance of a box of width
/// 2r+1 is ((2r+1)^2 - 1) / 12; three of them add up).
fn box_radius(sigma: f32) -> usize {
    let width = (4.0 * sigma * sigma + 1.0).sqrt();
    ((width - 1.0) / 2.0).round().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_keeps_mass_away_from_edges() {
        let p = Plane::from_fn(64, 64, |x, y| {
            if (28..36).contains(&x) && (28..36).contains(&y) {
                1.0
            } else {
                0.0
            }
        });
        let b = p.blurred(3.0);
        assert!(
            (b.data.iter().sum::<f32>() - 64.0).abs() < 0.5,
            "sum {}",
            b.data.iter().sum::<f32>()
        );
        assert!(b.at(32, 32) < 1.0 && b.at(32, 32) > 0.3);
    }

    #[test]
    fn shift_moves_and_zero_fills() {
        let p = Plane::from_fn(4, 4, |x, _| x as f32);
        let s = p.shifted(1, 0);
        assert_eq!(s.at(0, 0), 0.0);
        assert_eq!(s.at(3, 2), 2.0);
    }
}
