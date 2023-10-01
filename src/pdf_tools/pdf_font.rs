use std::cmp::Ordering;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PdfFont {
    pub size: (f32, f32),
    pub base_font: String,
}

impl PdfFont {
    #[allow(dead_code)]
    pub fn new(base_font: String, size: (f32, f32)) -> Self {
        PdfFont { size, base_font }
    }
    pub fn set_size(&mut self, size: (f32, f32)) {
        self.size = size
    }
    pub fn set_base_font(&mut self, base_font: String) {
        self.base_font = base_font
    }
}

impl Eq for PdfFont {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for PdfFont {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.size.partial_cmp(&other.size) {
            None | Some(Ordering::Equal) => Some(self.base_font.cmp(&other.base_font)),
            Some(x) => Some(x),
        }
    }
}

impl Ord for PdfFont {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        //arguments reversed for descending order
        other.partial_cmp(self).unwrap()
    }
}

impl std::fmt::Display for PdfFont {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ", self.base_font)?;
        if self.size.0 == self.size.1 {
            write!(f, "{}", self.size.0)
        } else {
            write!(f, "({}, {})", self.size.0, self.size.1)
        }
    }
}
