use sha2::{Digest, Sha256};

use super::fingerprint::IdentityMaterial;

pub type DeviceHash = [u8; 32];

pub fn compute_device_identity(material: &IdentityMaterial) -> DeviceHash {
    let mut hasher = Sha256::new();
    hasher.update(material.to_bytes());
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}
