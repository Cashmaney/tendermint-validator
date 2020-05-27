mod consts;
mod enclave_api;

use crate::enclave_api::{e_api_export_key, e_api_generate_key, e_api_import_key};
use enclave_api::{e_api_get_pubkey, e_api_sign, init_enclave};
use sgx_types::SgxResult;

pub fn e_if_get_pubkey() -> SgxResult<[u8; 32]> {
    let enclave = init_enclave().unwrap();
    let pubkey = e_api_get_pubkey(enclave.geteid());
    pubkey
}

pub fn e_if_sign(data: &[u8]) -> SgxResult<[u8; 64]> {
    let enclave = init_enclave().unwrap();
    let sig = e_api_sign(enclave.geteid(), data);
    sig
}

pub fn e_if_import_key(key: &[u8], password: &[u8]) -> SgxResult<()> {
    let enclave = init_enclave().unwrap();
    let res = e_api_import_key(enclave.geteid(), key, password);
    res
}

pub fn e_if_generate_key(password: &[u8]) -> SgxResult<()> {
    let enclave = init_enclave().unwrap();
    let res = e_api_generate_key(enclave.geteid(), password);
    res
}

pub fn e_if_export_key(key: &[u8]) -> SgxResult<[u8; 32]> {
    let enclave = init_enclave().unwrap();
    let res = e_api_export_key(enclave.geteid(), key);
    res
}

pub fn health_check_enclave() -> SgxResult<()> {
    if let Err(status) = init_enclave() {
        return Err(status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
