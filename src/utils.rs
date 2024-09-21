//! Utility functions

use alloy::primitives::{aliases::U192, Address};

/// Convert address to key
pub(crate) fn address_to_key(address: &Address) -> U192 {
    let mut key_bytes = [0u8; 24];
    key_bytes[..20].copy_from_slice(&address.as_slice());
    U192::from_be_bytes(key_bytes)
}
