//! RLP encoding macros for Ethereum data structures.
//!
//! This module provides macros for encoding Ethereum data structures using
//! the Recursive Length Prefix (RLP) encoding scheme. These macros simplify
//! the process of encoding multiple values in sequence while ensuring proper
//! RLP encoding rules are followed.

#[macro_export]
/// A macro for RLP encoding multiple values in sequence.
///
/// This macro provides a convenient way to encode multiple values using RLP encoding.
/// It handles both single values and multiple values, and ensures proper encoding
/// of the values in the specified order. The macro automatically handles the
/// encoding of each value and combines them into a single RLP-encoded output.
///
/// # Arguments
/// * `$out` - The output buffer to write the encoded data to
/// * `$e` - The first expression to encode
/// * `$($others:expr),+` - Additional expressions to encode (optional)
///
/// # Note
/// Each expression must implement the `Encodable` trait from `alloy_rlp`.
macro_rules! encode {
    ($out:ident, $e:expr) => {
        $e.encode($out);
        {
            let mut vec = vec![];
            $e.encode(&mut vec);
        }
    };
    ($out:ident, $e:expr, $($others:expr),+) => {
        {
            encode!($out, $e);
            encode!($out, $($others),+);
        }
    };
}
