use crate::data::consume_byte;

#[derive(Default, Clone, Copy)]
pub struct UpvalueData {
    pub index: u8,
    pub is_local: bool,
}

impl UpvalueData {
    pub fn fetch(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        let local = consume_byte(buffer, offset)?;
        let index = consume_byte(buffer, offset)?;
        Some(Self {
            is_local: local != 0,
            index,
        })
    }

    pub fn as_vec(&self) -> Vec<u8> {
        let byte = if self.is_local { 1u8 } else { 0 };
        vec![byte, self.index]
    }
}
