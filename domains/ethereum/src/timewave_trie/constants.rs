use std::ops::Range;
pub const EMPTY_STRING_CODE: u8 = 0x80;
pub const MAX: usize = 33;
pub const CHILD_INDEX_RANGE: Range<u8> = 0..16;

pub const EVEN_FLAG: u8 = 0x20;
pub const ODD_FLAG: u8 = 0x30;

pub const EMPTY_ROOT_HASH: &str =
    "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421";
