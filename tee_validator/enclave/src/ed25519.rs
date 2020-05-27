use sgx_rand::{Rng, StdRng};
use sgx_types::*;
use std::path::Path;
use std::sgxfs::SgxFile;
use std::vec::Vec;

use crate::aes::AesGcm256Key;
use crate::consts::{PASSWORD_FILE, SEALED_SIGNER_SEED_FILE};
use crate::fs;
use crate::password;
use log::*;
use sp_core::{crypto::Pair, ed25519};

pub fn unseal_pair() -> SgxResult<ed25519::Pair> {
    let key = password::unseal_password()?;

    let seedvec = unseal_seed_with_key(&key)?;

    let mut seed = [0u8; 32];
    // panics if not enough data
    seed.copy_from_slice(&seedvec);
    Ok(ed25519::Pair::from_seed(&seed))
}

pub fn does_key_exist() -> bool {
    if SgxFile::open(SEALED_SIGNER_SEED_FILE).is_err() {
        if Path::new(SEALED_SIGNER_SEED_FILE).exists() {
            panic!("[Enclave] Keyfile {} exists but can't be opened. has it been written by the same enclave?", SEALED_SIGNER_SEED_FILE);
        }
        return false;
    }
    return true;
}

fn unseal_seed() -> SgxResult<Vec<u8>> {
    fs::unseal(SEALED_SIGNER_SEED_FILE)
}

fn unseal_seed_with_key(key: &AesGcm256Key) -> SgxResult<Vec<u8>> {
    let mut enc_seed = fs::unseal(SEALED_SIGNER_SEED_FILE).unwrap();

    key.decrypt(&mut enc_seed);

    Ok(enc_seed)
}

pub fn seal_seed_with_key(seed: &[u8], key: &AesGcm256Key) -> SgxResult<sgx_status_t> {
    let mut enc_seed = seed.to_vec();

    key.encrypt(&mut enc_seed);

    fs::seal(&enc_seed, SEALED_SIGNER_SEED_FILE)
}

pub fn seal_seed(seed: &[u8]) -> SgxResult<sgx_status_t> {
    fs::seal(seed, SEALED_SIGNER_SEED_FILE)
}

pub fn create_sealed_seed(key: &AesGcm256Key) -> SgxResult<sgx_status_t> {
    let mut seed = [0u8; 32];
    let mut rand = match StdRng::new() {
        Ok(rng) => rng,
        Err(_) => {
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    };
    rand.fill_bytes(&mut seed);

    seal_seed_with_key(&seed, key)
}

// pub fn sign(bytes: &[u8]) -> SgxResult<[u8; 32]> {
//
// }
