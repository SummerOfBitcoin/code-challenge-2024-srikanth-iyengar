use std::fs;

use sha2::{Digest, Sha256};
use transaction::Transaction;

mod assembler;
mod opcodes;
mod stack;
mod transaction;
mod str_utils;

fn get_txs() -> Vec<Transaction> {
    let directory = "mempool/";
    let mut txs: Vec<Transaction> = Vec::new();

    let paths = fs::read_dir(directory).unwrap();

    for path in paths {
        let raw_json_tx: String = fs::read_to_string(path.as_ref().unwrap().path().display().to_string()).expect("");
        let result = Transaction::new(&raw_json_tx);
        match result {
            Ok(mut val) => {
                let json_path: String = path.as_ref().unwrap().path().display().to_string();

                let start_index = json_path.find(directory).unwrap_or(0) + directory.len();
                let end_index = json_path.find(".json").unwrap_or(json_path.len());

                val.sanity_hash = Some(String::from(&json_path[start_index..end_index]));
                txs.push(val);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    txs
}

fn main() {
    let mut txs: Vec<Transaction> = get_txs();
    // cosnider mutable iterator because, transactino id is computed and set accordingly
    txs.iter_mut().for_each(|tx| {
        let raw_tx: Vec<u8> = tx.get_raw_bytes();

        let mut hasher = Sha256::new();
        hasher.update(raw_tx);
        let result = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(result);
        let mut result = hasher.finalize();
        result.reverse();

        // after this step, result will have the actual txid
        let txid: String = result.iter().map(|val| format!("{:02x}", val)).collect();
        tx.txid = Some(txid);


        // this is just a sanity check whether, the serialzed data is correct or not
        let mut hasher = Sha256::new();
        hasher.update(result);
        let result = hasher.finalize();

        let hash_txid: String = result.iter().map(|val| format!("{:02x}", val)).collect();
        assert_eq!(hash_txid, *tx.sanity_hash.as_ref().unwrap());
    });
}

