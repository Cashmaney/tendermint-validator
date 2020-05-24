#![crate_name = "tee_validator_enclave"]
#![crate_type = "staticlib"]

#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate ring;
extern crate sgx_types;
extern crate sgx_trts;
extern crate sp_core;
extern crate sgx_rand;
extern crate log;

mod password;
mod fs;
mod ed25519;
mod consts;
mod utils;
mod aes;


use sp_core::{crypto::Pair, hashing::blake2_256};

use sgx_trts::trts::{rsgx_raw_is_outside_enclave, rsgx_lfence, rsgx_sfence};
use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};
use consts::SIGNATURE_SIZE;

use crate::aes::AesGcm256Key;
use crate::utils::{validate_const_ptr, validate_mut_ptr};

pub type Hash = sp_core::H256;

use log::*;
use utils::UnwrapOrSgxErrorUnexpected;

/// Signs the data in the input using a sealed ed25519 key
///
/// # Parameters
/// **data**
///
/// A pointer to the data to be signed
///
/// **data_len**
///
/// An unsigned int indicates the length of the data
///
/// **sig**
///
/// Signature on the input data
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn ecall_sign(data: *const u8,
                             data_len: u32,
                             sig: &mut [u8; SIGNATURE_SIZE]) -> sgx_status_t {

    validate_const_ptr(data, data_len as usize);
    validate_mut_ptr(sig.as_mut_ptr(), sig.len());


    let data_slice = unsafe { slice::from_raw_parts(data, data_len as usize) };

    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };

    let ed_sig = signer.sign(data_slice);

    sig.copy_from_slice(&ed_sig.as_ref());

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ecall_get_ecc_signing_pubkey(pubkey: *mut u8, pubkey_size: usize) -> sgx_status_t {

    validate_const_ptr(pubkey, pubkey_size as usize);

    if let Err(status) = ed25519::create_sealed_if_absent() {
        return status;
    }

    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };
    info!("Restored ECC pubkey: {:?}", signer.public());

    let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
    pubkey_slice.clone_from_slice(&signer.public());

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ecall_import_key(key: *const u8, key_size: usize,
                                          password: *const u8, pass_size: usize) -> sgx_status_t {

    validate_const_ptr(key, key_size as usize);
    validate_const_ptr(password, pass_size as usize);

    let key_slice = slice::from_raw_parts(key, key_size);
    let pass_slice = slice::from_raw_parts(password, pass_size);

    let aes_key = AesGcm256Key::from_password(pass_slice);

    if let Err(status) = password::seal_password(aes_key) {
        return status;
    }

    if let Err(status) = ed25519::seal_seed_with_key(key_slice, aes_key) {
        return status;
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ecall_export_key(password: *const u8, pass_size: usize, key: &mut [u8; 32]) -> sgx_status_t {

    if rsgx_raw_is_outside_enclave(password as * const u8, pass_size) {
        error!("ecall_get_encrypted_seed tried to access memory from outside the enclave");
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }
    rsgx_lfence();

    let key_slice = slice::from_raw_parts(password, pass_size );
    match password::validate_password(key_slice) {
        Ok(res) => if !res {
            error!("Invalid password");
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER
        }
        Err(status) => return status
    }
    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };

    key.copy_from_slice(signer.seed().as_ref());

    sgx_status_t::SGX_SUCCESS
}