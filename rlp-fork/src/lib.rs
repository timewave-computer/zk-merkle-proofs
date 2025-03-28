extern crate alloc;

mod decode;
pub use decode::{decode_exact, Decodable, Rlp};

mod error;
pub use error::{Error, Result};

pub mod alloy_bytes;

mod encode;
#[cfg(feature = "arrayvec")]
pub use encode::encode_fixed_size;
pub use encode::{
    encode, encode_iter, encode_list, length_of_length, list_length, Encodable, MaxEncodedLen,
    MaxEncodedLenAssoc,
};

mod header;
pub use header::{Header, PayloadView};

#[doc(no_inline)]
pub use bytes::{self, Buf, BufMut, Bytes, BytesMut};

/// RLP prefix byte for 0-length string.
pub const EMPTY_STRING_CODE: u8 = 0x80;

/// RLP prefix byte for a 0-length array.
pub const EMPTY_LIST_CODE: u8 = 0xC0;

// Not public API.
#[doc(hidden)]
#[deprecated(since = "0.3.0", note = "use `Error` instead")]
pub type DecodeError = Error;

#[doc(hidden)]
pub mod private {
    pub use core::{
        default::Default,
        option::Option::{self, None, Some},
        result::Result::{self, Err, Ok},
    };
}