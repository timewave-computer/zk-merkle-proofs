//! RLP encoding macros for Ethereum data structures.
//!
//! This module provides macros for encoding Ethereum data structures using
//! the Recursive Length Prefix (RLP) encoding scheme.

#[macro_export]
/// A macro for RLP encoding multiple values in sequence.
///
/// This macro provides a convenient way to encode multiple values using RLP encoding.
/// It handles both single values and multiple values, and ensures proper encoding
/// of the values in the specified order.
///
/// # Arguments
/// * `$out` - The output buffer to write the encoded data to
/// * `$e` - The first expression to encode
/// * `$($others:expr),+` - Additional expressions to encode (optional)
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
