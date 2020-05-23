extern crate tee_validator;
extern crate errno;

mod memory;
mod error;

use std::vec::Vec;

use error::{set_error};
use tee_validator::{e_if_get_pubkey, e_if_sign, e_if_import_key, e_if_export_key};
pub use memory::{free_rust, Buffer};

#[no_mangle]
pub extern "C" fn import_key(key: Buffer, err: Option<&mut Buffer>) -> bool {
    let data = match key.read() {
        None => {
            set_error("Imported key is empty".to_string(), err);
            return false;
        }
        Some(r) => r,
    };

    return match e_if_import_key(data) {
        Err(e) => {
            error::set_error(e.to_string(), err);
            false
        }
        Ok(_) => {
            error::clear_error();
            true
        }
    };
}

#[no_mangle]
pub extern "C" fn export_key(password: Buffer, err: Option<&mut Buffer>) -> Buffer {
    // let data = match password.read() {
    //     None => {
    //         set_error("Password is empty".to_string(), err);
    //         return Buffer::default();
    //     }
    //     Some(r) => r,
    // };

    let data: Vec<u8> = vec![1, 2, 3, 4];

    return match e_if_export_key(data.as_slice()) {
        Err(e) => {
            error::set_error(e.to_string(), err);
            Buffer::default()
        }
        Ok(key) => {
            error::clear_error();
            Buffer::from_vec(key.to_vec())
        }
    };
}

#[no_mangle]
pub extern "C" fn get_pubkey(err: Option<&mut Buffer>) -> Buffer {
    let result = e_if_get_pubkey();
    error::clear_error();
    Buffer::from_vec(result.to_vec())
}

#[no_mangle]
pub extern "C" fn sign(bytes: Buffer, err: Option<&mut Buffer>) -> Buffer {
    let data = match bytes.read() {
        None => {
            set_error("Encrypted seed is empty".to_string(), err);
            return Buffer::default();
        }
        Some(r) => r,
    };
    let result = e_if_sign(data);
    error::clear_error();
    Buffer::from_vec(result.to_vec())
}

// #[no_mangle]
// pub extern "C" fn import_key(keyfile: i32, password: i32) -> i32 {
//     // input * 2
// }
//
// #[no_mangle]
// pub extern "C" fn export_key(password: i32) -> i32 {
//     // input * 2
// }