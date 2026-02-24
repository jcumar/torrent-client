#[derive(Debug, PartialEq)]
pub struct Bitfield(pub Vec<u8>);

impl Bitfield {
    pub fn hash_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;

        if byte_index >= self.0.len() {
            return false;
        }

        self.0[byte_index] >> (7 - offset) & 1 != 0
    }
    
    pub fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;

        if byte_index >= self.0.len() {
            return;
        }

        self.0[byte_index] |= 1 << (7 - offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn test_hash_piece() {
        let bitfield = Bitfield(vec![0b01010100, 0b01010100]);

    	let outputs = [false, true, false, true, false, true, false, false,
            false, true, false, true, false, true, false, false, false, false,
            false, false];

        for (i, &output) in outputs.iter().enumerate() {
            assert_eq!(output, bitfield.hash_piece(i));
        }
    }
    
    #[test]
    fn test_set_piece() {
        struct TestCase {
            input: Bitfield,
            index: usize,
            expected: Bitfield,
        }

        let tests = vec![
            TestCase {
                input: Bitfield(vec![0b01010100, 0b01010100]),
                index: 4,
                expected: Bitfield(vec![0b01011100, 0b01010100]),
            },
            TestCase {
                input: Bitfield(vec![0b01010100, 0b01010100]),
                index: 9, // No-op because bit 9 (index 1 of byte 1) is already set
                expected: Bitfield(vec![0b01010100, 0b01010100]),
            },
            TestCase {
                input: Bitfield(vec![0b01010100, 0b01010100]),
                index: 15, // Last bit of second byte
                expected: Bitfield(vec![0b01010100, 0b01010101]),
            },
            TestCase {
                input: Bitfield(vec![0b01010100, 0b01010100]),
                index: 19, // Out of bounds (2 bytes = 16 bits), should no-op
                expected: Bitfield(vec![0b01010100, 0b01010100]),
            },
        ];

        for mut test in tests {
            test.input.set_piece(test.index);
            assert_eq!(
                test.input, test.expected, 
                "Failed at index {}", test.index
            );
        }
    }
} 
