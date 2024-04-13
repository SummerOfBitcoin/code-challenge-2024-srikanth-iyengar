use ripemd::Ripemd160;
use sha2::{Digest, Sha256};


pub fn hash256<'a> (data: &'a Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn double_hash256<'a> (data: &'a Vec<u8>) -> Vec<u8> {
    hash256(&hash256(data)).to_vec()
}

pub fn hash_ripemd<'a> (data: &'a Vec<u8>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn pk_hash<'a> (data: &'a Vec<u8>) -> Vec<u8> {
    hash_ripemd(&hash256(data)).to_vec()
}
