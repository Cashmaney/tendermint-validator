use crate::aes::AesGcm256Key;
use crate::fs;
use crate::consts::PASSWORD_FILE;

use sgx_types::{sgx_status_t, SgxResult};

pub fn seal_password(key: AesGcm256Key) -> SgxResult<sgx_status_t> {
    fs::seal(&key.to_slice(), PASSWORD_FILE)
}

pub fn unseal_password() -> SgxResult<AesGcm256Key> {
    let key_slice = fs::unseal(PASSWORD_FILE)?;
    Ok(AesGcm256Key::from_slice(&key_slice))
}

pub fn validate_password(password: &[u8]) -> SgxResult<bool> {
    let k1 = AesGcm256Key::from_password(password);

    let k2 = unseal_password()?;

    Ok(k1.key == k2.key)
}