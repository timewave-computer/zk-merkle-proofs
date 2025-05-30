extern crate alloc;
use crate::{timewave_rlp::Error, timewave_rlp::Header, timewave_rlp::Result};
use bytes::{Bytes, BytesMut};
use core::marker::{PhantomData, PhantomPinned};

/// A type that can be decoded from an RLP blob.
pub trait Decodable: Sized {
    /// Decodes the blob into the appropriate type.
    ///
    /// # Arguments
    /// * `buf` - A mutable reference to the buffer containing RLP-encoded data
    ///
    /// # Returns
    /// * `Ok(Self)` if decoding was successful
    /// * `Err(Error)` if decoding failed
    ///
    /// # Note
    /// The buffer must be advanced past the decoded object.
    fn decode(buf: &mut &[u8]) -> Result<Self>;
}

/// An active RLP decoder, with a specific slice of a payload.
#[derive(Debug)]
pub struct Rlp<'a> {
    payload_view: &'a [u8],
}

impl<'a> Rlp<'a> {
    /// Instantiate an RLP decoder with a payload slice.
    ///
    /// # Arguments
    /// * `payload` - The RLP-encoded payload to decode
    ///
    /// # Returns
    /// * `Ok(Self)` if the payload is valid RLP
    /// * `Err(Error)` if the payload is invalid
    pub fn new(mut payload: &'a [u8]) -> Result<Self> {
        let payload_view = Header::decode_bytes(&mut payload, true)?;
        Ok(Self { payload_view })
    }

    /// Decode the next item from the buffer.
    ///
    /// # Arguments
    /// * `T` - The type to decode into
    ///
    /// # Returns
    /// * `Ok(Some(T))` if an item was successfully decoded
    /// * `Ok(None)` if there are no more items to decode
    /// * `Err(Error)` if decoding failed
    #[inline]
    pub fn get_next<T: Decodable>(&mut self) -> Result<Option<T>> {
        if self.payload_view.is_empty() {
            Ok(None)
        } else {
            T::decode(&mut self.payload_view).map(Some)
        }
    }
}

impl<T: ?Sized> Decodable for PhantomData<T> {
    fn decode(_buf: &mut &[u8]) -> Result<Self> {
        Ok(Self)
    }
}

impl Decodable for PhantomPinned {
    fn decode(_buf: &mut &[u8]) -> Result<Self> {
        Ok(Self)
    }
}

impl Decodable for bool {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(match u8::decode(buf)? {
            0 => false,
            1 => true,
            _ => return Err(Error::Custom("invalid bool value, must be 0 or 1")),
        })
    }
}

impl<const N: usize> Decodable for [u8; N] {
    #[inline]
    fn decode(from: &mut &[u8]) -> Result<Self> {
        let bytes = Header::decode_bytes(from, false)?;
        Self::try_from(bytes).map_err(|_| Error::UnexpectedLength)
    }
}

macro_rules! decode_integer {
    ($($t:ty),+ $(,)?) => {$(
        impl Decodable for $t {
            #[inline]
            fn decode(buf: &mut &[u8]) -> Result<Self> {
                let bytes = Header::decode_bytes(buf, false)?;
                static_left_pad(bytes).map(<$t>::from_be_bytes)
            }
        }
    )+};
}

decode_integer!(u8, u16, u32, u64, usize, u128);

impl Decodable for Bytes {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Header::decode_bytes(buf, false).map(|x| Self::from(x.to_vec()))
    }
}

impl Decodable for BytesMut {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Header::decode_bytes(buf, false).map(Self::from)
    }
}

impl Decodable for alloc::string::String {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Header::decode_str(buf).map(Into::into)
    }
}

impl<T: Decodable> Decodable for alloc::vec::Vec<T> {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        let mut bytes = Header::decode_bytes(buf, true)?;
        let mut vec = Self::new();
        let payload_view = &mut bytes;
        while !payload_view.is_empty() {
            vec.push(T::decode(payload_view)?);
        }
        Ok(vec)
    }
}

macro_rules! wrap_impl {
    ($($(#[$attr:meta])* [$($gen:tt)*] <$t:ty>::$new:ident($t2:ty)),+ $(,)?) => {$(
        $(#[$attr])*
        impl<$($gen)*> Decodable for $t {
            #[inline]
            fn decode(buf: &mut &[u8]) -> Result<Self> {
                <$t2 as Decodable>::decode(buf).map(<$t>::$new)
            }
        }
    )+};
}

wrap_impl! {
    [const N: usize] <arrayvec::ArrayVec<u8, N>>::from([u8; N]),
    [T: Decodable] <alloc::boxed::Box<T>>::new(T),
    [T: Decodable] <alloc::rc::Rc<T>>::new(T),
    [T: Decodable] <alloc::sync::Arc<T>>::new(T),
}

impl<T: ?Sized + alloc::borrow::ToOwned> Decodable for alloc::borrow::Cow<'_, T>
where
    T::Owned: Decodable,
{
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        T::Owned::decode(buf).map(Self::Owned)
    }
}

/// Decodes the entire input, ensuring no trailing bytes remain.
///
/// # Errors
///
/// Returns an error if the encoding is invalid or if data remains after decoding the RLP item.
#[inline]
pub fn decode_exact<T: Decodable>(bytes: impl AsRef<[u8]>) -> Result<T> {
    let mut buf = bytes.as_ref();
    let out = T::decode(&mut buf)?;

    // check if there are any remaining bytes after decoding
    if !buf.is_empty() {
        // TODO: introduce a new variant TrailingBytes to better distinguish this error
        return Err(Error::UnexpectedLength);
    }

    Ok(out)
}

/// Left-pads a slice to a statically known size array.
///
/// # Errors
///
/// Returns an error if the slice is too long or if the first byte is 0.
#[inline]
pub(crate) fn static_left_pad<const N: usize>(data: &[u8]) -> Result<[u8; N]> {
    if data.len() > N {
        return Err(Error::Overflow);
    }

    let mut v = [0; N];

    // yes, data may empty, e.g. we decode a bool false value
    if data.is_empty() {
        return Ok(v);
    }

    if data[0] == 0 {
        return Err(Error::LeadingZero);
    }

    // SAFETY: length checked above
    unsafe { v.get_unchecked_mut(N - data.len()..) }.copy_from_slice(data);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timewave_rlp::encode, timewave_rlp::Encodable};
    use core::fmt::Debug;
    use hex_literal::hex;

    #[allow(unused_imports)]
    use alloc::{format, string::String, vec, vec::Vec};

    fn check_decode<'a, T, IT>(fixtures: IT)
    where
        T: Encodable + Decodable + PartialEq + Debug,
        IT: IntoIterator<Item = (Result<T>, &'a [u8])>,
    {
        for (expected, mut input) in fixtures {
            if let Ok(expected) = &expected {
                assert_eq!(crate::timewave_rlp::encode(expected), input, "{expected:?}");
            }

            let orig = input;
            assert_eq!(
                T::decode(&mut input),
                expected,
                "input: {}{}",
                hex::encode(orig),
                expected.as_ref().map_or_else(
                    |_| String::new(),
                    |expected| format!(
                        "; expected: {}",
                        hex::encode(crate::timewave_rlp::encode(expected))
                    )
                )
            );

            if expected.is_ok() {
                assert_eq!(input, &[] as &[u8]);
            }
        }
    }

    #[test]
    fn rlp_bool() {
        let out = [0x80];
        let val = bool::decode(&mut &out[..]);
        assert_eq!(Ok(false), val);

        let out = [0x01];
        let val = bool::decode(&mut &out[..]);
        assert_eq!(Ok(true), val);
    }

    #[test]
    fn rlp_strings() {
        check_decode::<Bytes, _>([
            (Ok(hex!("00")[..].to_vec().into()), &hex!("00")[..]),
            (
                Ok(hex!("6f62636465666768696a6b6c6d")[..].to_vec().into()),
                &hex!("8D6F62636465666768696A6B6C6D")[..],
            ),
            (Err(Error::UnexpectedList), &hex!("C0")[..]),
        ])
    }

    #[test]
    fn rlp_fixed_length() {
        check_decode([
            (
                Ok(hex!("6f62636465666768696a6b6c6d")),
                &hex!("8D6F62636465666768696A6B6C6D")[..],
            ),
            (
                Err(Error::UnexpectedLength),
                &hex!("8C6F62636465666768696A6B6C")[..],
            ),
            (
                Err(Error::UnexpectedLength),
                &hex!("8E6F62636465666768696A6B6C6D6E")[..],
            ),
        ])
    }

    #[test]
    fn rlp_u64() {
        check_decode([
            (Ok(9_u64), &hex!("09")[..]),
            (Ok(0_u64), &hex!("80")[..]),
            (Ok(0x0505_u64), &hex!("820505")[..]),
            (Ok(0xCE05050505_u64), &hex!("85CE05050505")[..]),
            (Err(Error::Overflow), &hex!("8AFFFFFFFFFFFFFFFFFF7C")[..]),
            (
                Err(Error::InputTooShort),
                &hex!("8BFFFFFFFFFFFFFFFFFF7C")[..],
            ),
            (Err(Error::UnexpectedList), &hex!("C0")[..]),
            (Err(Error::LeadingZero), &hex!("00")[..]),
            (Err(Error::NonCanonicalSingleByte), &hex!("8105")[..]),
            (Err(Error::LeadingZero), &hex!("8200F4")[..]),
            (Err(Error::NonCanonicalSize), &hex!("B8020004")[..]),
            (
                Err(Error::Overflow),
                &hex!("A101000000000000000000000000000000000000008B000000000000000000000000")[..],
            ),
        ])
    }

    #[test]
    fn rlp_vectors() {
        check_decode::<Vec<u64>, _>([
            (Ok(vec![]), &hex!("C0")[..]),
            (
                Ok(vec![0xBBCCB5_u64, 0xFFC0B5_u64]),
                &hex!("C883BBCCB583FFC0B5")[..],
            ),
        ])
    }

    #[test]
    fn malformed_rlp() {
        check_decode::<Bytes, _>([
            (Err(Error::InputTooShort), &hex!("C1")[..]),
            (Err(Error::InputTooShort), &hex!("D7")[..]),
        ]);
        check_decode::<[u8; 5], _>([
            (Err(Error::InputTooShort), &hex!("C1")[..]),
            (Err(Error::InputTooShort), &hex!("D7")[..]),
        ]);
        check_decode::<Vec<u8>, _>([
            (Err(Error::InputTooShort), &hex!("C1")[..]),
            (Err(Error::InputTooShort), &hex!("D7")[..]),
        ]);
        check_decode::<String, _>([
            (Err(Error::InputTooShort), &hex!("C1")[..]),
            (Err(Error::InputTooShort), &hex!("D7")[..]),
        ]);
        check_decode::<String, _>([
            (Err(Error::InputTooShort), &hex!("C1")[..]),
            (Err(Error::InputTooShort), &hex!("D7")[..]),
        ]);
        check_decode::<u8, _>([(Err(Error::InputTooShort), &hex!("82")[..])]);
        check_decode::<u64, _>([(Err(Error::InputTooShort), &hex!("82")[..])]);
    }

    #[test]
    fn rlp_full() {
        fn check_decode_exact<T: Decodable + Encodable + PartialEq + Debug>(input: T) {
            let encoded = encode(&input);
            assert_eq!(decode_exact::<T>(&encoded), Ok(input));
            assert_eq!(
                decode_exact::<T>([encoded, vec![0x00]].concat()),
                Err(Error::UnexpectedLength)
            );
        }

        check_decode_exact::<String>("".into());
        check_decode_exact::<String>("test1234".into());
        check_decode_exact::<Vec<u64>>(vec![]);
        check_decode_exact::<Vec<u64>>(vec![0; 4]);
    }
}
