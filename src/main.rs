use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs};
use transaction::Transaction;

mod hash_utils;
mod interpreter;
mod opcodes;
mod stack;
mod str_utils;
mod transaction;

fn get_txs() -> Vec<Transaction> {
    // let directory = "serialize_test/";
    let directory = "mempool/";

    let mut txs: Vec<Transaction> = Vec::new();

    let paths = fs::read_dir(directory).unwrap();

    for path in paths {
        let raw_json_tx: String =
            fs::read_to_string(path.as_ref().unwrap().path().display().to_string()).expect("");
        let result = Transaction::new(&raw_json_tx);
        match result {
            Ok(mut val) => {
                let json_path: String = path.as_ref().unwrap().path().display().to_string();

                let start_index = json_path.find(directory).unwrap_or(0) + directory.len();
                let end_index = json_path.find(".json").unwrap_or(json_path.len());

                val.sanity_hash = Some(String::from(&json_path[start_index..end_index]));


                // check if the current transaction is segwit
                let pubkey_types: HashSet<String> = val.vin.iter().map(|vin| vin.prevout.scriptpubkey_type.clone()).collect();
                val.is_segwit = Some(pubkey_types.contains("v0_p2wpkh") || pubkey_types.contains("v0_p2wsh"));

                txs.push(val);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    txs
}

fn remove_double_spending_tx<'a>(txs: &'a mut Vec<Transaction>) -> Vec<&'a Transaction> {
    let mut used_tx: HashSet<String> = HashSet::new();
    let filtered_txs: Vec<&Transaction> = txs
        .iter()
        .map(|tx| {
            let mut should_accept: bool = true;


            tx.vin.iter().for_each(|vin| {
                let vout_str = vin.vout.to_string();

                // check if txid#vout is already used in previously selected
                // transaction
                let key = vin.txid.clone() + "#" + vout_str.as_str();

                should_accept &= used_tx.get(&key) == None;

                // push the txid#vout in the map
                used_tx.insert(vin.txid.clone() + "#" + vout_str.as_str());
            });

            tx.vout.iter().enumerate().for_each(|(idx, _)| {
                let key = tx.txid.as_ref().unwrap().clone() + "#" + idx.to_string().as_str();
                used_tx.insert(key);
            });

            if should_accept {
                Some(tx)
            } else {
                None
            }
        })
        .filter(|tx| match tx {
            Some(_) => true,
            None => false,
        })
        .map(|tx| tx.unwrap())
        .collect();
    filtered_txs
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

    // this is the filtered txs of double spending
    let txs = remove_double_spending_tx(&mut txs);

    println!("Number of txs after removing double spending {}", txs.len());

    let mut verified_tx: Vec<&Transaction> = Vec::new();
    // verify each trannscations vin
    for tx in txs.iter() {
        if tx.validate_transacation() {
            verified_tx.push(&tx);
        }
    }
    println!("Verified {}", verified_tx.len());

    // now we have the verified transactions
}
