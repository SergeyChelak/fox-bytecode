use crate::data::consume_byte;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
        let local = if self.is_local { 1u8 } else { 0 };
        vec![local, self.index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upvalue_write_test() {
        let src_upvalue = UpvalueData {
            is_local: true,
            index: 59,
        };
        let buffer = src_upvalue.as_vec();
        assert_eq!(buffer, vec![1, 59]);
    }

    #[test]
    fn upvalue_read_test() {
        let buffers = [vec![1, 59], vec![0, 37]];

        let expected = [
            UpvalueData {
                is_local: true,
                index: 59,
            },
            UpvalueData {
                is_local: false,
                index: 37,
            },
        ];

        for (buf, exp) in buffers.iter().zip(expected.iter()) {
            let mut offset = 0;
            let upvalue = UpvalueData::fetch(buf, &mut offset).expect("Failed to parse upvalue");
            assert_eq!(upvalue, *exp);
        }
    }
}
