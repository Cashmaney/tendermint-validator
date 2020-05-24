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
// under the License.

use std::prelude::v1::*;

use ring::aead;
use std::format;
use std::path::Path;
use sgx_rand::{Rng, StdRng};
use sgx_types::*;

use crate::blake2_256;

const AES_GCM_256_KEY_LENGTH: usize = 32;
const AES_GCM_256_IV_LENGTH: usize = 12;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AesGcm256Key {
    pub key: [u8; AES_GCM_256_KEY_LENGTH],
    pub iv: [u8; AES_GCM_256_IV_LENGTH],
}

impl AesGcm256Key {
    pub const SCHEMA: &'static str = "aes-gcm-256";

    pub fn new(in_key: &[u8], in_iv: &[u8]) -> SgxResult<Self> {
        if in_key.len() != AES_GCM_256_KEY_LENGTH {
            panic!("Invalid key length for AesGcm256: {}")

        }
        if in_iv.len() == AES_GCM_256_IV_LENGTH {
            panic!("Invalid iv length for AesGcm256: {}")
        }

        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        key.copy_from_slice(in_key);
        iv.copy_from_slice(in_iv);

        Ok(AesGcm256Key { key, iv })
    }

    pub fn to_slice(&self) -> Vec<u8> {
        let mut result: Vec<u8> = self.iv.to_vec();
        result.extend_from_slice(&self.key);

        result
    }

    pub fn from_slice(slice: &[u8]) -> Self {

        if slice.len() != AES_GCM_256_IV_LENGTH + AES_GCM_256_KEY_LENGTH {
            panic!("Encryption key corrupted")
        }

        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];

        iv.copy_from_slice(&slice[..AES_GCM_256_IV_LENGTH]);
        key.copy_from_slice(&slice[AES_GCM_256_IV_LENGTH..AES_GCM_256_IV_LENGTH + AES_GCM_256_KEY_LENGTH]);
        return Self {
            iv,
            key
        }
    }

    pub fn from_password(in_pass: &[u8]) -> Self {
        let key = blake2_256(in_pass);

        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        let mut rand = match StdRng::new() {
            Ok(rng) => rng,
            Err(_) => {
                return Self {
                    key: [0u8; AES_GCM_256_KEY_LENGTH],
                    iv,
                };
            }
        };
        rand.fill_bytes(&mut iv);

        return Self {
            iv,
            key
        }
    }

    pub fn random() -> Self {
        Self::default()
    }

    pub fn decrypt(&self, in_out: &mut Vec<u8>) -> SgxResult<()> {
        let plaintext_len = aead_decrypt(&aead::AES_256_GCM, in_out, &self.key, &self.iv)?.len();
        in_out.truncate(plaintext_len);

        Ok(())
    }

    pub fn encrypt(&self, in_out: &mut Vec<u8>) -> SgxResult<()> {
        aead_encrypt(&aead::AES_256_GCM, in_out, &self.key, &self.iv)
    }
}

pub fn aead_decrypt<'a>(
    alg: &'static aead::Algorithm,
    in_out: &'a mut [u8],
    key: &[u8],
    iv: &[u8],
) -> SgxResult<&'a mut [u8]> {
    let key =
        aead::UnboundKey::new(alg, key).map_err(|_| panic!("Aead unbound key init error"))?;
    let nonce =
        aead::Nonce::try_assume_unique_for_key(iv).map_err(|_| panic!("Aead iv init error"))?;
    let aad = aead::Aad::from([0u8; 8]);

    let dec_key = aead::LessSafeKey::new(key);
    let slice = dec_key
        .open_in_place(nonce, aad, in_out)
        .map_err(|_| panic!("Aead open_in_place error"))?;
    Ok(slice)
}

pub fn aead_encrypt(
    alg: &'static aead::Algorithm,
    in_out: &mut Vec<u8>,
    key: &[u8],
    iv: &[u8],
) -> SgxResult<()> {
    let key =
        aead::UnboundKey::new(alg, key).map_err(|_| panic!("Aead unbound key init error"))?;
    let nonce =
        aead::Nonce::try_assume_unique_for_key(iv).map_err(|_| panic!("Aead iv init error"))?;
    let aad = aead::Aad::from([0u8; 8]);

    let enc_key = aead::LessSafeKey::new(key);
    enc_key
        .seal_in_place_append_tag(nonce, aad, in_out)
        .map_err(|_| panic!("Aead seal_in_place_append_tag error"))?;
    Ok(())
}


impl Default for AesGcm256Key {
    fn default() -> Self {
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        let mut rand = match StdRng::new() {
            Ok(rng) => rng,
            Err(_) => {
                return Self {
                    key,
                    iv,
                };
            }
        };
        rand.fill_bytes(&mut key);
        rand.fill_bytes(&mut iv);

        Self { key, iv }
    }
}
