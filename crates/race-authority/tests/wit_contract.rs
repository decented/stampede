//! `wit/authority.wit` is a byte copy of fray-session's frozen contract, from the fray
//! revision pinned in Cargo.toml. fray's own test pins the same digest, so a copy that
//! drifts from the contract the session layer speaks fails here.
use sha2::{Digest, Sha256};

#[test]
fn the_wit_copy_is_fray_sessions_frozen_contract() {
    let digest = Sha256::digest(include_bytes!("../wit/authority.wit"));
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, "bf769f074945476a4da4c66fa0be4be39a9cc05269d6e3d3e0442b2d58a95900");
}
