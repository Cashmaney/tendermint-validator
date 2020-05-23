// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#![crate_name = "tee_validator_enclave"]
#![crate_type = "staticlib"]

// #![no_std]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate sgx_types;
extern crate sgx_trts;
extern crate sp_core;
extern crate sgx_rand;
extern crate log;



use sp_core::{crypto::Pair, hashing::blake2_256};

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};
use consts::SIGNATURE_SIZE;
use sgx_trts::trts::{rsgx_raw_is_outside_enclave, rsgx_lfence, rsgx_sfence};

mod fs;
mod ed25519;
mod consts;
mod utils;

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

    if rsgx_raw_is_outside_enclave(data as * const u8, data_len as usize) {
        error!("ecall_get_encrypted_seed tried to access memory from outside the enclave");
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }
    rsgx_lfence();

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

    if rsgx_raw_is_outside_enclave(pubkey as * const u8, pubkey_size) {
        error!("ecall_get_encrypted_seed tried to access memory from outside the enclave");
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }
    rsgx_sfence();

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
pub unsafe extern "C" fn ecall_import_key(key: *const u8, key_size: usize) -> sgx_status_t {

    if rsgx_raw_is_outside_enclave(key as * const u8, key_size) {
        error!("ecall_get_encrypted_seed tried to access memory from outside the enclave");
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }
    rsgx_lfence();

    let key_slice = slice::from_raw_parts(key, key_size);

    if let Err(status) = ed25519::seal_seed(key_slice) {
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

    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };

    key.copy_from_slice(signer.seed().as_ref());

    sgx_status_t::SGX_SUCCESS
}