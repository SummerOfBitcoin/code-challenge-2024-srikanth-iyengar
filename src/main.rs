use std::{collections::HashSet, fs, io::Write};

use sha2::{Digest, Sha256};

use crate::utils::tx_type::{ScriptPubkey, ScriptPubkeyType, Transaction};

pub mod utils {
    pub mod assembler;
    pub mod str;
    pub mod opcodes;
    pub mod stack;
    pub mod tx_type;
}

fn main() {
    let mut transactions: Vec<Transaction> = Vec::new();

    let paths = fs::read_dir("test_tx").unwrap();

    for path in paths {
        let raw_json_tx =
            fs::read_to_string(path.as_ref().unwrap().path().display().to_string()).expect("");
        let result = Transaction::new(&raw_json_tx);
        match result {
            Ok(mut result) => {
                result.file_name = Some(path.as_ref().unwrap().path().display().to_string());
                transactions.push(result);
            }
            Err(e) => {
                println!(
                    "error while parsing tx {:?} {}",
                    e,
                    path.unwrap().path().display().to_string()
                );
            }
        }
    }

    let mut instruction_set: HashSet<u32> = HashSet::new();
    for tx in transactions {
        instruction_set.insert(tx.version);
        if tx.version != 1 {
            continue
        }
        // check if all vin are p2pkh
        // let mut check = true;
        // for pubkey in tx.vin.iter() {
        //     check &= pubkey.prevout.scriptpubkey_type == "p2pkh";
        // }
        // for pubkey in tx.vout.iter() {
        //     check &= pubkey.scriptpubkey_type == "p2pkh";
        // }
        // if !check {
        //     continue;
        // }
        let x = tx.version_1_get_raw_bytes();
        // println!("{:?}", x);
        // first hash
        let mut hasher = Sha256::new();
        hasher.update(x);
        let result = hasher.finalize();

        // // second hash
        let mut hasher = Sha256::new();
        hasher.update(result);
        let mut result = hasher.finalize();

        // result.reverse();

        let mut hasher = Sha256::new();
        hasher.update(result);
        let result = hasher.finalize();

        let hex_string: String = result.iter().map(|val| {
            format!("{:02x?}", val)
        }).collect();
        // if tx.file_name.unwrap().contains(&hex_string) {
        println!("{} {}", hex_string, tx.file_name.unwrap());
        println!();
        // }
    }

    println!("{:?}", instruction_set);
}
