use crate::timewave_rlp::Decodable;
#[derive(Clone, Copy)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedBytes<N> {
    /// Returns a slice containing the entire array.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns a new `FixedBytes` from a slice.
    pub fn from_slice(src: &[u8]) -> Self {
        match Self::try_from(src) {
            Ok(x) => x,
            Err(_) => panic!(
                "cannot convert a slice of length {} to FixedBytes<{N}>",
                src.len()
            ),
        }
    }

    /// Converts the fixed byte array into a `Vec<u8>`.
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

/// Tries to create a `FixedBytes<N>` by copying from a slice `&[u8]`. Succeeds
/// if `slice.len() == N`.
impl<const N: usize> TryFrom<&[u8]> for FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        <&Self>::try_from(slice).copied()
    }
}

/// Tries to create a `FixedBytes<N>` by copying from a mutable slice `&mut
/// [u8]`. Succeeds if `slice.len() == N`.
impl<const N: usize> TryFrom<&mut [u8]> for FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &mut [u8]) -> Result<Self, Self::Error> {
        Self::try_from(&*slice)
    }
}

/// Tries to create a ref `FixedBytes<N>` by copying from a slice `&[u8]`.
/// Succeeds if `slice.len() == N`.
impl<'a, const N: usize> TryFrom<&'a [u8]> for &'a FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [u8]) -> Result<&'a FixedBytes<N>, Self::Error> {
        // SAFETY: `FixedBytes<N>` is `repr(transparent)` for `[u8; N]`
        <&[u8; N]>::try_from(slice).map(|array_ref| unsafe { core::mem::transmute(array_ref) })
    }
}

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use core::{
    borrow::Borrow,
    fmt,
    ops::{Deref, DerefMut, RangeBounds},
};

/// Wrapper type around [`bytes::Bytes`] to support "0x" prefixed hex strings.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Bytes(pub bytes::Bytes);

impl Default for &Bytes {
    #[inline]
    fn default() -> Self {
        static EMPTY: Bytes = Bytes::new();
        &EMPTY
    }
}

impl Decodable for Bytes {
    #[inline]
    fn decode(buf: &mut &[u8]) -> crate::timewave_rlp::Result<Self, crate::timewave_rlp::Error> {
        bytes::Bytes::decode(buf).map(Self)
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl fmt::LowerHex for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&hex::encode_prefixed(self.as_ref()))
    }
}

impl fmt::UpperHex for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&hex::encode_upper_prefixed(self.as_ref()))
    }
}

impl Deref for Bytes {
    type Target = bytes::Bytes;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bytes {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Borrow<[u8]> for Bytes {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl FromIterator<u8> for Bytes {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(bytes::Bytes::from_iter(iter))
    }
}

impl<'a> FromIterator<&'a u8> for Bytes {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        Self(iter.into_iter().copied().collect::<bytes::Bytes>())
    }
}

impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = bytes::buf::IntoIter<bytes::Bytes>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<bytes::Bytes> for Bytes {
    #[inline]
    fn from(value: bytes::Bytes) -> Self {
        Self(value)
    }
}

impl From<Bytes> for bytes::Bytes {
    #[inline]
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl From<Vec<u8>> for Bytes {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<FixedBytes<N>> for Bytes {
    #[inline]
    fn from(value: FixedBytes<N>) -> Self {
        value.to_vec().into()
    }
}

impl<const N: usize> From<&'static FixedBytes<N>> for Bytes {
    #[inline]
    fn from(value: &'static FixedBytes<N>) -> Self {
        Self::from_static(value.as_slice())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    #[inline]
    fn from(value: [u8; N]) -> Self {
        value.to_vec().into()
    }
}

impl<const N: usize> From<&'static [u8; N]> for Bytes {
    #[inline]
    fn from(value: &'static [u8; N]) -> Self {
        Self::from_static(value)
    }
}

impl From<&'static [u8]> for Bytes {
    #[inline]
    fn from(value: &'static [u8]) -> Self {
        Self::from_static(value)
    }
}

impl From<&'static str> for Bytes {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::from_static(value.as_bytes())
    }
}

impl From<Box<[u8]>> for Bytes {
    #[inline]
    fn from(value: Box<[u8]>) -> Self {
        Self(value.into())
    }
}

impl From<Bytes> for Vec<u8> {
    #[inline]
    fn from(value: Bytes) -> Self {
        value.0.into()
    }
}

impl PartialEq<[u8]> for Bytes {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self[..] == *other
    }
}

impl PartialEq<Bytes> for [u8] {
    #[inline]
    fn eq(&self, other: &Bytes) -> bool {
        *self == other[..]
    }
}

impl PartialEq<Vec<u8>> for Bytes {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self[..] == other[..]
    }
}

impl PartialEq<Bytes> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialEq<bytes::Bytes> for Bytes {
    #[inline]
    fn eq(&self, other: &bytes::Bytes) -> bool {
        other == self.as_ref()
    }
}

impl core::str::FromStr for Bytes {
    type Err = hex::FromHexError;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        hex::decode(value).map(Into::into)
    }
}

impl hex::FromHex for Bytes {
    type Error = hex::FromHexError;

    #[inline]
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        hex::decode(hex).map(Self::from)
    }
}

impl bytes::Buf for Bytes {
    #[inline]
    fn remaining(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.0.chunk()
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        self.0.advance(cnt)
    }

    #[inline]
    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        self.0.copy_to_bytes(len)
    }
}

impl Bytes {
    /// Creates a new empty `Bytes`.
    ///
    /// This will not allocate and the returned `Bytes` handle will be empty.
    ///
    #[inline]
    pub const fn new() -> Self {
        Self(bytes::Bytes::new())
    }

    /// Creates a new `Bytes` from a static slice.
    ///
    /// The returned `Bytes` will point directly to the static slice. There is
    /// no allocating or copying.
    ///
    #[inline]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self(bytes::Bytes::from_static(bytes))
    }

    /// Creates a new `Bytes` instance from a slice by copying it.
    #[inline]
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self(bytes::Bytes::copy_from_slice(data))
    }

    /// Returns a slice of self for the provided range.
    ///
    /// # Panics
    ///
    /// Requires that `begin <= end` and `end <= self.len()`, otherwise slicing
    /// will panic.
    #[inline]
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        Self(self.0.slice(range))
    }

    /// Returns a slice of self that is equivalent to the given `subset`.
    ///
    /// # Panics
    ///
    /// Requires that the given `subset` slice is in fact contained within the
    /// `Bytes` buffer; otherwise this function will panic.
    #[inline]
    pub fn slice_ref(&self, subset: &[u8]) -> Self {
        Self(self.0.slice_ref(subset))
    }

    /// Splits the bytes into two at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[must_use = "consider Bytes::truncate if you don't need the other half"]
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }

    /// Splits the bytes into two at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[must_use = "consider Bytes::advance if you don't need the other half"]
    #[inline]
    pub fn split_to(&mut self, at: usize) -> Self {
        Self(self.0.split_to(at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let expected = Bytes::from_static(&[0x12, 0x13, 0xab, 0xcd]);
        assert_eq!("1213abcd".parse::<Bytes>().unwrap(), expected);
        assert_eq!("0x1213abcd".parse::<Bytes>().unwrap(), expected);
        assert_eq!("1213ABCD".parse::<Bytes>().unwrap(), expected);
        assert_eq!("0x1213ABCD".parse::<Bytes>().unwrap(), expected);
    }

    #[test]
    fn format() {
        let b = Bytes::from_static(&[1, 35, 69, 103, 137, 171, 205, 239]);
        assert_eq!(format!("{b}"), "0x0123456789abcdef");
        assert_eq!(format!("{b:x}"), "0x0123456789abcdef");
        assert_eq!(format!("{b:?}"), "0x0123456789abcdef");
        assert_eq!(format!("{b:#?}"), "0x0123456789abcdef");
        assert_eq!(format!("{b:#x}"), "0x0123456789abcdef");
        assert_eq!(format!("{b:X}"), "0x0123456789ABCDEF");
        assert_eq!(format!("{b:#X}"), "0x0123456789ABCDEF");
    }
}
