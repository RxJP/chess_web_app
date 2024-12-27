use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn generate_hash(inp: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(inp);
    return STANDARD_NO_PAD.encode(hasher.finalize());
}