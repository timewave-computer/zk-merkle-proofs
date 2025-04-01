use std::ops::Range;
pub const EMPTY_STRING_CODE: u8 = 0x80;
pub const MAX: usize = 33;
pub const CHILD_INDEX_RANGE: Range<u8> = 0..16;

pub const EVEN_FLAG: u8 = 0x20;
pub const ODD_FLAG: u8 = 0x30;

pub const EMPTY_ROOT_HASH_BYTES: [u8; 32] = [
    86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153,
    108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33,
];
