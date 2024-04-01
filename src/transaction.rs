use serde::Deserialize;
use serde_json;

use crate::str_utils::{get_compact_size_bytes, get_hex_bytes};


pub enum PubkeyType {
    P2PKH,    // Pay to pubkey hash
    P2WPKH,   // SegWit transaction unlock script type, witness field is present
    P2WSH,    // SegWit transaction unlock script type, witness field is present
    P2TR,     // Pay to taproot locks bitcoin
    P2SH,     // Pay to hash
    OPRETURN, // This is itself a opcode as well as a script itself, used to prevent burn money ?
}

#[derive(Deserialize)]
pub struct Pubkey {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64
}

#[derive(Deserialize)]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    pub prevout: Pubkey,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[derive(Deserialize)]
pub struct Transaction {
    pub txid: Option<String>,
    // this is the sha256 hash of the txid (reverse order, again just a bitcoin thing)
    pub sanity_hash: Option<String>,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Pubkey>,
}

impl Transaction {
    // Constructor for Transaction from raw json string
    pub fn new(raw_json_tx: &str) -> Result<Transaction, serde_json::Error> {
        let tx: Transaction = serde_json::from_str(raw_json_tx)?;
        Ok(tx)
    }

    // Raw transaction in bytes which can be considered for computing txid
    pub fn get_raw_bytes(&self) -> Vec<u8> {
        let mut raw_bytes: Vec<u8> = Vec::new();

        // First we push the version bytes
        let version_bytes: Vec<u8> = self.version.to_le_bytes().to_vec();
        version_bytes.iter().for_each(|val| raw_bytes.push(*val));

        // The number of input bytes
        let vin_len_bytes = get_compact_size_bytes(&(self.vin.len() as u64));
        vin_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

        // will push the input ids
        self.vin.iter().for_each(|vin| {
            // will first push in txid in reverse order you know just bitcoin things
            let txid_bytes: Vec<u8> = get_hex_bytes(vin.txid.as_ref()).unwrap();
            txid_bytes.iter().rev().for_each(|val| raw_bytes.push(*val));

            // vout bytes in little endian format
            let vout_bytes: Vec<u8> = vin.vout.to_le_bytes().to_vec();
            vout_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // get length of scriptsig size in compact_size foramt
            let scriptsig_len_bytes: Vec<u8> = get_compact_size_bytes(&((vin.scriptsig.len() / 2) as u64));
            scriptsig_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // lets push the actual signature now
            let scriptsig_bytes: Vec<u8> = get_hex_bytes(&vin.scriptsig).unwrap();
            scriptsig_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // push the sequence bytes in little endian format
            let sequence_bytes: Vec<u8> = vin.sequence.to_le_bytes().to_vec();
            sequence_bytes.iter().for_each(|val| raw_bytes.push(*val));
        });

        // push vout bytes in compact size format
        let vout_len_bytes: Vec<u8> = get_compact_size_bytes(&(self.vout.len() as u64));
        vout_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

        self.vout.iter().for_each(|vout| {
            // push amout in little endian format
            let amount_bytes: Vec<u8> = vout.value.to_le_bytes().to_vec();
            amount_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // push scriptsig size in compact size format
            let scriptsigsize_len_bytes: Vec<u8> = get_compact_size_bytes(&((vout.scriptpubkey.len() / 2) as u64));
            scriptsigsize_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // push the actual scriptsig
            let scripitsig_bytes: Vec<u8> = get_hex_bytes(&vout.scriptpubkey).unwrap();
            scripitsig_bytes.iter().for_each(|val| raw_bytes.push(*val));
        });

        self.locktime.to_le_bytes().iter().for_each(|val| raw_bytes.push(*val));

        raw_bytes
    }
}
