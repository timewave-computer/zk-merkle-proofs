#[cfg(not(feature = "sp1"))]
use tiny_keccak::{Hasher, Keccak};
#[cfg(feature = "sp1")]
use tiny_keccak_sp1::{Hasher, Keccak};

pub fn digest_keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}
