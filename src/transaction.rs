use std::str::FromStr;

use serde::Deserialize;
use serde_json;

use crate::{
    hash_utils::double_hash256,
    interpreter::Interpreter,
    opcodes::all_opcodes::{OP_CHECKSIG, OP_DUP, OP_EQUALVERIFY, OP_HASH160, OP_PUSHBYTES},
    str_utils::{get_compact_size_bytes, get_hex_bytes},
};

#[path = "./test/transaction_tests.rs"]
#[cfg(test)]
mod transaction_test;

#[derive(Debug)]
pub enum PubkeyType {
    P2PKH,    // Pay to pubkey hash
    P2WPKH,   // SegWit transaction unlock script type, witness field is present
    P2WSH,    // SegWit transaction unlock script type, witness field is present
    P2TR,     // Pay to taproot locks bitcoin
    P2SH,     // Pay to hash
    OPRETURN, // This is itself a opcode as well as a script itself, used to prevent burn money ?
}

impl FromStr for PubkeyType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p2pkh" => Ok(PubkeyType::P2PKH),
            "p2sh" => Ok(PubkeyType::P2SH),
            "v0_p2wpkh" => Ok(PubkeyType::P2WPKH),
            "v0_p2wsh" => Ok(PubkeyType::P2WSH),
            "v1_p2tr" => Ok(PubkeyType::P2TR),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize)]
pub struct Pubkey {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
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
    pub inner_redeemscript_asm: Option<String>,
}

#[derive(Deserialize)]
pub struct Transaction {
    pub txid: Option<String>,
    pub wtxid: Option<String>,
    // this is the sha256 hash of the txid (reverse order, again just a bitcoin thing)
    pub sanity_hash: Option<String>,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Pubkey>,
    pub is_segwit: Option<bool>,
    pub weight: Option<usize>,
    pub tx_fee: Option<u64>,
}

impl Transaction {
    // Constructor for Transaction from raw json string
    pub fn new(raw_json_tx: &str) -> Result<Transaction, serde_json::Error> {
        let mut tx: Transaction = serde_json::from_str(raw_json_tx)?;
        tx.assign_weight();
        tx.assign_tx_fee();
        Ok(tx)
    }

    // Raw transaction in bytes which can be considered for computing txid
    pub fn get_raw_bytes(&self, include_witness: bool) -> Vec<u8> {
        let mut raw_bytes: Vec<u8> = Vec::new();

        // First we push the version bytes
        let version_bytes: Vec<u8> = self.version.to_le_bytes().to_vec();
        version_bytes.iter().for_each(|val| raw_bytes.push(*val));

        if include_witness {
            raw_bytes.push(0x00);
            raw_bytes.push(0x01);
        }

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
            let scriptsig_len_bytes: Vec<u8> =
                get_compact_size_bytes(&((vin.scriptsig.len() / 2) as u64));
            scriptsig_len_bytes
                .iter()
                .for_each(|val| raw_bytes.push(*val));

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
            let scriptsigsize_len_bytes: Vec<u8> =
                get_compact_size_bytes(&((vout.scriptpubkey.len() / 2) as u64));
            scriptsigsize_len_bytes
                .iter()
                .for_each(|val| raw_bytes.push(*val));

            // push the actual scriptsig
            let scripitsig_bytes: Vec<u8> = get_hex_bytes(&vout.scriptpubkey).unwrap();
            scripitsig_bytes.iter().for_each(|val| raw_bytes.push(*val));
        });

        if include_witness {
            // push number of txs

            for vin in self.vin.iter() {
                // iterate over all the witness field if any
                if let Some(witness) = &vin.witness {
                    get_compact_size_bytes(&(witness.len() as u64))
                        .iter()
                        .for_each(|x| raw_bytes.push(*x));
                    witness.iter().for_each(|val| {
                        get_compact_size_bytes(&((val.len() / 2) as u64))
                            .iter()
                            .for_each(|x| raw_bytes.push(*x)); //  I know that this wil be a even number
                        get_hex_bytes(val)
                            .unwrap()
                            .iter()
                            .for_each(|x| raw_bytes.push(*x));
                    });
                } else {
                    raw_bytes.push(0x00);
                }
            }
        }

        self.locktime
            .to_le_bytes()
            .iter()
            .for_each(|val| raw_bytes.push(*val));

        raw_bytes
    }

    pub fn get_raw_tx_for_legacy_tx(&self, idx: u32) -> Vec<u8> {
        let mut raw_bytes: Vec<u8> = Vec::new();

        // push version bytes
        let version_bytes: Vec<u8> = self.version.to_le_bytes().to_vec();
        version_bytes.iter().for_each(|val| raw_bytes.push(*val));

        // number of vins in compact size format
        let vin_len_bytes = get_compact_size_bytes(&(self.vin.len() as u64));
        vin_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

        // operation specific to each vin
        for (i, vin) in self.vin.iter().enumerate() {
            // push txid bytes
            let txid_bytes = get_hex_bytes(&vin.txid);
            if let Ok(vin) = txid_bytes {
                vin.iter().rev().for_each(|val| raw_bytes.push(*val));
            }

            // vout bytes in little endian format
            let vout_bytes: Vec<u8> = vin.vout.to_le_bytes().to_vec();
            vout_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // include the vin which we are verifying, else script will be empty
            if idx == i as u32 {
                // get length of scriptsig size in compact_size foramt
                let scriptsig_len_bytes: Vec<u8> =
                    get_compact_size_bytes(&((vin.prevout.scriptpubkey.len() / 2) as u64));
                scriptsig_len_bytes
                    .iter()
                    .for_each(|val| raw_bytes.push(*val));
                // lets push the actual script sig now
                let scriptsig_bytes: Vec<u8> = get_hex_bytes(&vin.prevout.scriptpubkey).unwrap();
                scriptsig_bytes.iter().for_each(|val| raw_bytes.push(*val));
            } else {
                let x = 0x00;
                let scriptsig_len_bytes: Vec<u8> = get_compact_size_bytes(&(x as u64));
                scriptsig_len_bytes
                    .iter()
                    .for_each(|val| raw_bytes.push(*val));
            }

            // push the sequence bytes in little endian format
            let sequence_bytes: Vec<u8> = vin.sequence.to_le_bytes().to_vec();
            sequence_bytes.iter().for_each(|val| raw_bytes.push(*val));
        }

        // push vout bytes in compact size format
        let vout_len_bytes: Vec<u8> = get_compact_size_bytes(&(self.vout.len() as u64));
        vout_len_bytes.iter().for_each(|val| raw_bytes.push(*val));

        self.vout.iter().for_each(|vout| {
            // push amout in little endian format
            let amount_bytes: Vec<u8> = vout.value.to_le_bytes().to_vec();
            amount_bytes.iter().for_each(|val| raw_bytes.push(*val));

            // push scriptsig size in compact size format
            let scriptsigsize_len_bytes: Vec<u8> =
                get_compact_size_bytes(&((vout.scriptpubkey.len() / 2) as u64));
            scriptsigsize_len_bytes
                .iter()
                .for_each(|val| raw_bytes.push(*val));

            // push the actual scriptsig
            let scripitsig_bytes: Vec<u8> = get_hex_bytes(&vout.scriptpubkey).unwrap();
            scripitsig_bytes.iter().for_each(|val| raw_bytes.push(*val));
        });

        self.locktime
            .to_le_bytes()
            .iter()
            .for_each(|val| raw_bytes.push(*val));

        raw_bytes
    }

    pub fn serialize_witness_transaction(&self, idx: u32) -> Vec<u8> {
        let version_bytes: Vec<u8> = self.version.to_le_bytes().to_vec();

        let mut txid_vout_bytes: Vec<u8> = Vec::new();
        let mut sequence_bytes: Vec<u8> = Vec::new();

        for vin in self.vin.iter() {
            // prepare txid+vout for all tx
            let txid_bytes = get_hex_bytes(&vin.txid);
            if let Ok(val) = txid_bytes {
                val.iter().rev().for_each(|x| txid_vout_bytes.push(*x));
            }
            let vout_bytes = vin.vout.to_le_bytes().to_vec();
            vout_bytes.iter().for_each(|x| txid_vout_bytes.push(*x));

            // prepare sequence byte array
            vin.sequence
                .to_le_bytes()
                .iter()
                .for_each(|val| sequence_bytes.push(*val));
        }

        let txid_vout_hash = double_hash256(&txid_vout_bytes);
        let sequence_hash = double_hash256(&sequence_bytes);

        let vin = self.vin.get(idx as usize);

        let (cur_txid_vout_bytes, scriptcode_bytes, amount_bytes, sequence_bytes) =
            if let Some(val) = vin {
                let mut cur_txid_vout_bytes: Vec<u8> = Vec::new();
                let mut scriptcode_bytes: Vec<u8> = vec![
                    OP_DUP.code,
                    OP_HASH160.code,
                    OP_PUSHBYTES.code + 0x014 - 0x01,
                ];
                if let Ok(txid_bytes) = get_hex_bytes(&val.txid) {
                    txid_bytes
                        .iter()
                        .rev()
                        .for_each(|x| cur_txid_vout_bytes.push(*x));
                }

                val.vout
                    .to_le_bytes()
                    .to_vec()
                    .iter()
                    .for_each(|x| cur_txid_vout_bytes.push(*x));

                // crerate the scriptcode
                if let Ok(pubkeyhash) = get_hex_bytes(val.prevout.scriptpubkey.as_str()) {
                    pubkeyhash[2..]
                        .iter()
                        .for_each(|val| scriptcode_bytes.push(*val));

                    scriptcode_bytes.push(OP_EQUALVERIFY.code);
                    scriptcode_bytes.push(OP_CHECKSIG.code)
                }

                // amount
                let amount_bytes = val.prevout.value.to_le_bytes();

                // sequence
                let sequence_bytes = val.sequence.to_le_bytes();

                (
                    cur_txid_vout_bytes,
                    scriptcode_bytes,
                    amount_bytes,
                    sequence_bytes,
                )
            } else {
                // can we do better coz this is deadcode
                let zero_u32: u32 = 0x00;
                let zero_u64: u64 = 0x00;
                (
                    vec![],
                    vec![],
                    zero_u64.to_le_bytes(),
                    zero_u32.to_le_bytes(),
                )
            };

        let mut vout_bytes: Vec<u8> = Vec::new();

        self.vout.iter().for_each(|vout| {
            // push amout in little endian format
            let amount_bytes: Vec<u8> = vout.value.to_le_bytes().to_vec();
            amount_bytes.iter().for_each(|val| vout_bytes.push(*val));

            // push scriptsig size in compact size format
            let scriptsigsize_len_bytes: Vec<u8> =
                get_compact_size_bytes(&((vout.scriptpubkey.len() / 2) as u64));
            scriptsigsize_len_bytes
                .iter()
                .for_each(|val| vout_bytes.push(*val));

            // push the actual scriptsig
            let scripitsig_bytes: Vec<u8> = get_hex_bytes(&vout.scriptpubkey).unwrap();
            scripitsig_bytes
                .iter()
                .for_each(|val| vout_bytes.push(*val));
        });

        let vout_hash = double_hash256(&vout_bytes);

        let locktime_bytes = self.locktime.to_le_bytes();

        let mut raw_bytes: Vec<u8> = Vec::new();

        // let debug: String = scriptcode_bytes.iter().map(|x| format!("{:02x}", x)).collect();
        // println!("{} {}", debug, debug.len());

        // preimage = version + hash256(inputs) + hash256(sequences) + input +
        // scriptcode + amount + sequence + hash256(outputs) + locktime
        version_bytes.iter().for_each(|x| raw_bytes.push(*x));
        txid_vout_hash.iter().for_each(|x| raw_bytes.push(*x));
        sequence_hash.iter().for_each(|x| raw_bytes.push(*x));
        cur_txid_vout_bytes.iter().for_each(|x| raw_bytes.push(*x));
        get_compact_size_bytes(&(scriptcode_bytes.len() as u64))
            .iter()
            .for_each(|x| raw_bytes.push(*x));
        scriptcode_bytes.iter().for_each(|x| raw_bytes.push(*x));
        amount_bytes.iter().for_each(|x| raw_bytes.push(*x));
        sequence_bytes.iter().for_each(|x| raw_bytes.push(*x));
        vout_hash.iter().for_each(|x| raw_bytes.push(*x));
        locktime_bytes.iter().for_each(|x| raw_bytes.push(*x));

        raw_bytes
    }

    pub fn get_raw_tx_for_vin(&self, idx: u32) -> Vec<u8> {
        let parsed_type = self
            .vin
            .get(idx as usize)
            .unwrap()
            .prevout
            .scriptpubkey_type
            .parse::<PubkeyType>();
        match parsed_type.unwrap() {
            PubkeyType::P2WSH | PubkeyType::P2WPKH => {
                return self.serialize_witness_transaction(idx);
            }
            _ => self.get_raw_tx_for_legacy_tx(idx),
        }
    }

    pub fn validate_transacation(&self) -> bool {
        let mut success = true;

        for (idx, vin) in self.vin.iter().enumerate() {
            if let Ok(parsed_enum) = vin.prevout.scriptpubkey_type.parse::<PubkeyType>() {
                match parsed_enum {
                    PubkeyType::P2PKH => {
                        // prepare the instruction
                        let mut instructions: Vec<u8> = Vec::new();
                        if let Ok(scripitsig_bytes) = get_hex_bytes(vin.scriptsig.as_ref()) {
                            scripitsig_bytes
                                .iter()
                                .for_each(|val| instructions.push(*val));
                        }

                        // push the scriptpubkey
                        if let Ok(scriptpubkey_bytes) = get_hex_bytes(&vin.prevout.scriptpubkey) {
                            scriptpubkey_bytes
                                .iter()
                                .for_each(|val| instructions.push(*val));
                        }

                        let mut interpreter = Interpreter::new(&instructions, idx as u32, self);
                        if let Some(result) = interpreter.exec_all() {
                            success &= result.len() == 1 && result[0] == 0x01;
                        }
                    }
                    PubkeyType::P2WPKH => {
                        // prepare the instruction
                        let pubkeyhash =
                            if let Ok(scriptpubkey) = get_hex_bytes(&vin.prevout.scriptpubkey) {
                                scriptpubkey[2..].to_vec()
                            } else {
                                vec![]
                            };

                        // basic sanity check if the pubkeyhash is of 20 bytes
                        assert_eq!(pubkeyhash.len(), 20);

                        let mut instruction = vec![
                            OP_DUP.code,
                            OP_HASH160.code,
                            OP_PUSHBYTES.code + 0x014 - 0x01,
                        ];

                        pubkeyhash.iter().for_each(|val| instruction.push(*val));

                        instruction.push(OP_EQUALVERIFY.code);
                        instruction.push(OP_CHECKSIG.code);

                        let mut interpreter = Interpreter::new(&instruction, idx as u32, self);

                        vin.witness.as_ref().unwrap().iter().for_each(|val| {
                            if let Ok(bytes) = get_hex_bytes(val) {
                                interpreter.stack.push(bytes);
                            }
                        });

                        if let Some(result) = interpreter.exec_all() {
                            success &= result.len() == 1 && result[0] == 0x01;
                        }
                    },
                    _ => {
                        success = false;
                    }
                }
            }
        }

        success
    }

    pub fn assign_weight(&mut self) {
        // these are the fields that will directly go with x4 multiplier
        let raw_tx = self.get_raw_bytes(false);
        let raw_tx_with_witness = self.get_raw_bytes(true);

        let weight = 3 * raw_tx.len() +  raw_tx_with_witness.len();

        self.weight = Some(weight);
    }

    pub fn assign_tx_fee(&mut self) {
        let vin_amount: u64 = self.vin.iter().map(|vin| vin.prevout.value).sum();
        let vout_amount: u64 = self.vout.iter().map(|vout| vout.value).sum();

        if vin_amount > vout_amount {
            self.tx_fee = Some(vin_amount - vout_amount);
        }
    }
}
