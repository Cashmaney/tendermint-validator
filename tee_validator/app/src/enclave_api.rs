extern crate sgx_types;
extern crate sgx_urts;

use std::path::Path;
use std::env;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::{fs::File, path::PathBuf};
use std::io::{Read, Write};

use crate::consts::{ENCLAVE_FILE};

use log::*;

extern "C" {
    // fn init(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;

    fn ecall_get_ecc_signing_pubkey(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        pubkey: *mut u8,
        pubkey_size: usize,
    ) -> sgx_status_t;

    fn ecall_sign(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        data: *const u8,
        data_len: usize,
        sig: &mut [u8; 64],
    ) -> sgx_status_t;

    fn ecall_import_key(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        key: *const u8,
        key_len: usize,
    ) -> sgx_status_t;

    fn ecall_export_key(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        password: *const u8,
        pass_len: usize,
        key: &mut [u8; 32],
    ) -> sgx_status_t;
}

pub fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };

    // Step : try to create a .enigma folder for storing all the files
    // Create a directory, returns `io::Result<()>`
    let enclave_directory = env::var("ENCLAVE_DIR").unwrap_or('.'.to_string());

    let path = Path::new(&enclave_directory);

    let enclave_file_path: std::path::PathBuf = path.join(ENCLAVE_FILE);

    SgxEnclave::create(
        enclave_file_path,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

pub fn e_api_get_pubkey(eid: sgx_enclave_id_t) -> SgxResult<[u8; 32]> {
    let pubkey_size = 32;
    let mut pubkey = [0u8; 32];
    let mut status = sgx_status_t::SGX_SUCCESS;
    let result =
        unsafe { ecall_get_ecc_signing_pubkey(eid, &mut status, pubkey.as_mut_ptr(), pubkey_size) };
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    Ok(pubkey)
}

pub fn e_api_sign(eid: sgx_enclave_id_t, data: &[u8]) -> SgxResult<[u8; 64]> {
    let mut sig = [0u8; 64];
    let mut status = sgx_status_t::SGX_SUCCESS;
    let result =
        unsafe { ecall_sign(eid, &mut status, data.as_ptr(), data.len(), &mut sig) };
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    // check that sig is not 0
    Ok(sig)
}

pub fn e_api_import_key(eid: sgx_enclave_id_t, key: &[u8]) -> SgxResult<()> {

    let mut status = sgx_status_t::SGX_SUCCESS;
    let result =
        unsafe { ecall_import_key(eid, &mut status, key.as_ptr(), key.len()) };
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    // check that sig is not 0
    Ok(())
}

pub fn e_api_export_key(eid: sgx_enclave_id_t, password: &[u8]) -> SgxResult<[u8; 32]> {
    let mut key = [0u8; 32];
    let mut status = sgx_status_t::SGX_SUCCESS;
    let result =
        unsafe { ecall_export_key(eid, &mut status, password.as_ptr(), password.len(), &mut key) };
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    // check that key is not 0
    Ok(key)
}