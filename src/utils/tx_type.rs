use std::str::FromStr;

use serde::Deserialize;

use crate::utils::assembler::Assembler;
use crate::utils::str::{decode_hex, get_compact_size_bytes};

use super::str;

#[derive(Debug)]
pub enum ScriptPubkeyType {
    P2PKH,    // Pay to pubkey hash
    P2WPKH,   // SegWit transaction unlock script type, witness field is present
    P2WSH,    // SegWit transaction unlock script type, witness field is present
    P2TR,     // Pay to taproot locks bitcoin
    P2SH,     // Pay to hash
    OPRETURN, // This is itself a opcode as well as a script itself, used to prevent burn money ?
}

#[derive(Deserialize, Debug)]
pub struct ScriptPubkey {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    #[serde(default = "default_str")]
    pub scriptpubkey_address: String,
    pub value: u64,
}

#[derive(Deserialize, Debug)]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    pub prevout: ScriptPubkey,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    #[serde(default = "empty_vec")]
    pub witness: Vec<String>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub file_name: Option<String>,
    pub version: u32,
    pub locktime: u32,
    pub vout: Vec<ScriptPubkey>,
    pub vin: Vec<Vin>,
}

fn empty_vec() -> Vec<String> {
    vec![]
}

fn default_str() -> String {
    String::from("")
}

impl FromStr for ScriptPubkeyType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p2sh" => Ok(ScriptPubkeyType::P2SH),
            "v0_p2wpkh" => Ok(ScriptPubkeyType::P2WPKH),
            "v0_p2wsh" => Ok(ScriptPubkeyType::P2WSH),
            "v1_p2tr" => Ok(ScriptPubkeyType::P2TR),
            "p2pkh" => Ok(ScriptPubkeyType::P2PKH),
            "op_return" => Ok(ScriptPubkeyType::OPRETURN),
            _ => Err(()),
        }
    }
}

impl Transaction {
    // Function to deserialize JSON string into Transaction struct
    pub fn new(raw_str: &String) -> Result<Transaction, serde_json::Error> {
        let tx: Transaction = serde_json::from_str(raw_str)?;
        Ok(tx)
    }

    pub fn version_1_get_raw_bytes(&self) -> Vec<u8> {
        let mut raw_result: Vec<u8> = Vec::new();

        let version_bytes = self.version.to_le_bytes();
        version_bytes.iter().for_each(|val| raw_result.push(*val));

        let vin_len = self.vin.len();
        get_compact_size_bytes(vin_len as u64)
            .iter()
            .for_each(|val| raw_result.push(*val));

        self.vin.iter().for_each(|input| {
            decode_hex(input.txid.as_str()).unwrap()
                .iter()
                .rev()
                .for_each(|val| raw_result.push(*val));
            input
                .vout
                .to_le_bytes()
                .iter()
                .for_each(|val| raw_result.push(*val));

            get_compact_size_bytes((input.scriptsig.len() / 2) as u64)
                .iter()
                .for_each(|val| raw_result.push(*val));

            decode_hex(input
                .scriptsig.as_str()).unwrap()
                .iter()
                .for_each(|val| raw_result.push(*val));
            input
                .sequence
                .to_le_bytes()
                .iter()
                .for_each(|val| raw_result.push(*val));
        });

        let vout_len = self.vout.len();
        get_compact_size_bytes(vout_len as u64)
            .iter()
            .for_each(|val| raw_result.push(*val));

        self.vout.iter().for_each(|out| {
            out.value
                .to_le_bytes()
                .iter()
                .for_each(|val| raw_result.push(*val));

            get_compact_size_bytes((out.scriptpubkey.len() / 2) as u64)
                .iter()
                .for_each(|val| raw_result.push(*val));

            decode_hex(out.scriptpubkey.as_str()).unwrap()
                .iter()
                .for_each(|val| raw_result.push(*val));
        });

        self.locktime.to_le_bytes().iter().for_each(|val| raw_result.push(*val));

        raw_result
    }

    pub fn verify_tx(&self) {
        for i in 0..self.vin.len() {
            let mut asm_program: Vec<u8> = Vec::new();
            decode_hex(&self.vin[i].scriptsig)
                .unwrap()
                .iter()
                .for_each(|val| asm_program.push(*val));
            decode_hex(&self.vin[i].prevout.scriptpubkey)
                .unwrap()
                .iter()
                .for_each(|val| asm_program.push(*val));

            let mut assembler: Assembler = Assembler::new(&asm_program);
            if let Some(exit_val) = assembler.exec_all() {
                // println!("{:?}", exit_val);
            } else {
                println!("Program should crash");
            }
        }
    }
}

impl ScriptPubkey {
    pub fn validate_script(self) {
        let script_type: Result<ScriptPubkeyType, ()> = self.scriptpubkey_type.parse();
        match script_type {
            Ok(pubscript_type) => match pubscript_type {
                ScriptPubkeyType::P2PKH => {}
                ScriptPubkeyType::P2WSH => {}
                ScriptPubkeyType::P2TR => {}
                ScriptPubkeyType::P2SH => {}
                ScriptPubkeyType::P2WPKH => {}
                ScriptPubkeyType::OPRETURN => {}
            },
            Err(_) => {
                println!(
                    "Type doesn't exist in the enum, tx: {}",
                    self.scriptpubkey_type
                );
            }
        }
    }
}
