pub struct UTF8Iter<'a> {
    pub i: usize,
    pub data: &'a [u8],
    //iter: std::slice::Iter<'a, u8>
}

pub trait Utf8ToBytes {
    fn utf8_to_bytes(self) -> Option<(u8, [u8; 4])>;
}

pub trait UTF8IntoIter<'a> {
    fn utf8_iter(&'a self) -> UTF8Iter<'a>;
}

pub trait BitFlag {
    fn has_flag(&self, flag: Self) -> bool;
    fn has_flag_with_mask(&self, mask: Self, flag: Self) -> bool;
}

pub trait UTF8Flag {
    fn utf8_glyth_size(&self) -> Option<u8>;
    fn utf8_is_next_glyth(&self) -> bool;
}

impl Utf8ToBytes for char {
    fn utf8_to_bytes(self) -> Option<(u8, [u8; 4])> {
        let mut buffer: [u8; 4] = [0; 4];
        self.encode_utf8(&mut buffer);
        let size = buffer[0].utf8_glyth_size();
        size.map(|size| (size, buffer))
    }
}

impl<'a> UTF8IntoIter<'a> for &[u8] {
    fn utf8_iter(&'a self) -> UTF8Iter<'a> {
        UTF8Iter { i: 0, data: self }
    }
}

impl<'a> UTF8IntoIter<'a> for &str {
    fn utf8_iter(&'a self) -> UTF8Iter<'a> {
        UTF8Iter {
            i: 0,
            data: self.as_bytes(),
        }
    }
}

impl<'a> UTF8IntoIter<'a> for Vec<u8> {
    fn utf8_iter(&'a self) -> UTF8Iter<'a> {
        UTF8Iter { i: 0, data: &self }
    }
}

impl<'a> Iterator for UTF8Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let glytph_size = self.data.get(self.i)?.utf8_glyth_size()?;

        let slice = &self.data[self.i..self.i + glytph_size as usize];

        self.i += glytph_size as usize;

        Some(slice)
    }
}

impl BitFlag for u8 {
    fn has_flag(&self, flag: Self) -> bool {
        *self & flag == flag
    }

    fn has_flag_with_mask(&self, mask: Self, flag: Self) -> bool {
        *self & mask == flag
    }
}

impl UTF8Flag for u8 {
    fn utf8_glyth_size(&self) -> Option<u8> {
        let size = if self.has_flag_with_mask(0b11111_000_u8, 0b11110_000_u8) {
            4
        } else if self.has_flag_with_mask(0b1111_0000_u8, 0b1110_0000_u8) {
            3
        } else if self.has_flag_with_mask(0b111_00000_u8, 0b110_00000_u8) {
            2
        } else if self.has_flag_with_mask(0b1_0000000_u8, 0b0_0000000_u8) {
            1
        } else {
            return None;
        };

        Some(size)
    }

    fn utf8_is_next_glyth(&self) -> bool {
        self.has_flag_with_mask(0b11_000000_u8, 0b10_000000_u8)
    }
}