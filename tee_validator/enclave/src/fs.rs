use std::fs::File;
use std::io::{Read, Write};
use std::sgxfs::SgxFile;
use std::string::String;
use std::vec::Vec;

use sgx_types::*;

use crate::utils::UnwrapOrSgxErrorUnexpected;

pub fn unseal(filepath: &str) -> SgxResult<Vec<u8>> {
    SgxFile::open(filepath)
        .map(_read)
        .sgx_error_with_log(&format!("[Enclave] File '{}' not found!", filepath))?
}

pub fn read(filepath: &str) -> SgxResult<Vec<u8>> {
    File::open(filepath)
        .map(_read)
        .sgx_error_with_log(&format!("[Enclave] File '{}' not found!", filepath))?
}

fn _read<F: Read>(mut file: F) -> SgxResult<Vec<u8>> {
    let mut read_data: Vec<u8> = Vec::new();
    file.read_to_end(&mut read_data)
        .sgx_error_with_log("[Enclave] Reading File failed!")?;

    Ok(read_data)
}

pub fn read_to_string(filepath: &str) -> SgxResult<String> {
    let mut contents = String::new();
    File::open(filepath)
        .map(|mut f| f.read_to_string(&mut contents))
        .sgx_error_with_log(&format!("[Enclave] Could not read '{}'", filepath))?
        .sgx_error_with_log(&format!("[Enclave] File '{}' not found!", filepath))?;

    Ok(contents)
}

pub fn seal(bytes: &[u8], filepath: &str) -> SgxResult<sgx_status_t> {
    SgxFile::create(filepath)
        .map(|f| _write(bytes, f))
        .sgx_error_with_log(&format!("[Enclave] Creating '{}' failed", filepath))?
}

pub fn write(bytes: &[u8], filepath: &str) -> SgxResult<sgx_status_t> {
    File::create(filepath)
        .map(|f| _write(bytes, f))
        .sgx_error_with_log(&format!("[Enclave] Creating '{}' failed", filepath))?
}

fn _write<F: Write>(bytes: &[u8], mut file: F) -> SgxResult<sgx_status_t> {
    file.write_all(bytes)
        .sgx_error_with_log("[Enclave] Writing File failed!")?;

    Ok(sgx_status_t::SGX_SUCCESS)
}