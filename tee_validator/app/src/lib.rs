mod enclave_api;
mod consts;

use enclave_api::{e_api_get_pubkey, init_enclave, e_api_sign};
use sgx_types::SgxResult;
use crate::enclave_api::{e_api_export_key, e_api_import_key};


pub fn e_if_get_pubkey() -> [u8; 32] {
    let enclave = init_enclave().unwrap();
    let pubkey = e_api_get_pubkey(enclave.geteid()).unwrap();
    pubkey
}

pub fn e_if_sign(data: &[u8]) -> [u8; 64] {
    let enclave = init_enclave().unwrap();
    let sig = e_api_sign(enclave.geteid(), data).unwrap();
    sig
}

pub fn e_if_import_key(key: &[u8], password: &[u8]) -> SgxResult<()> {
    let enclave = init_enclave().unwrap();
    let res = e_api_import_key(enclave.geteid(), key, password);
    res
}

pub fn e_if_export_key(key: &[u8]) -> SgxResult<[u8; 32]> {
    let enclave = init_enclave().unwrap();
    let res = e_api_export_key(enclave.geteid(), key);
    res
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
