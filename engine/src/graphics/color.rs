#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    fn is_valid(c: char) -> Option<u8> {
        match c {
            '0' => Some(0),
            '1' => Some(1),
            '2' => Some(2),
            '3' => Some(3),
            '4' => Some(4),
            '5' => Some(5),
            '6' => Some(6),
            '7' => Some(7),
            '8' => Some(8),
            '9' => Some(9),
            'A' => Some(10),
            'B' => Some(11),
            'C' => Some(12),
            'D' => Some(13),
            'E' => Some(14),
            'F' => Some(15),
            _ => None,
        }
    }

    fn next_two(chars: &mut dyn Iterator<Item = char>) -> Result<u8, Box<dyn std::error::Error>> {
        Ok(
            Self::is_valid(chars.next().ok_or("invalid")?).ok_or("invalid character")? * 16
                + Self::is_valid(chars.next().ok_or("invalid")?).ok_or("invalid character")?,
        )
    }

    pub fn from_hex(hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let upper = &hex[1..].to_uppercase();
        let mut chars = upper.chars();

        let r = Self::next_two(&mut chars)?;
        let g = Self::next_two(&mut chars)?;
        let b = Self::next_two(&mut chars)?;
        let a = Self::next_two(&mut chars).unwrap_or(255);

        Ok(Color { r, g, b, a })
    }

    pub fn fade(mut self, alpha: f32) -> Self {
        self.a = (alpha * 256.0).floor() as u8;
        self
    }
}

#[inline]
fn cv(n: f64) -> f64 {
    (n / 256.0).powf(2.2)
}

/// Converts color from srgb to wgpu color, but corrects for gamma.
/// sRGB is stored in relative color, while our eyes perceive the brightness differently, so we have to
/// modify the sRGB according to the gamma curve, with an exponent of ~ 2.2
/// See [learnopengl/gamma-correction](https://learnopengl.com/Advanced-Lighting/Gamma-Correction) & [learnwgpu/colorcorrection](https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/#color-correction)
/// for more information.
impl From<Color> for wgpu::Color {
    fn from(val: Color) -> Self {
        wgpu::Color {
            r: cv(val.r as f64),
            g: cv(val.g as f64),
            b: cv(val.b as f64),
            a: cv(val.a as f64),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Color;
    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("292828").unwrap();
        assert_eq!(
            color,
            Color {
                r: 41,
                g: 40,
                b: 40,
                a: 255
            }
        );
    }
    #[test]
    fn test_color_to_wgpu_color() {
        let color = Color::from_hex("292828").unwrap();
        assert_eq!(
            wgpu::Color::from(color),
            wgpu::Color {
                r: 41.0 / 256.0,
                g: 40.0 / 256.0,
                b: 40.0 / 256.0,
                a: 1.0
            }
        );
    }
}
