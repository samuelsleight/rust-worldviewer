#[derive(Debug, Copy, Clone)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: u8::MAX,
        }
    }

    pub fn as_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
