use std::{
    fs::File,
    io::Write
};
use transaction::Transaction;

use crate::{
    hash_utils::{double_hash256, hash256},
    merkle::reorder_txs,
    utils::{get_txs, prepare_blockheader, prepare_coinbase_tx, remove_double_spending_tx, pick_best_transactions},
};

mod hash_utils;
mod interpreter;
mod macro_utils;
mod merkle;
mod opcodes;
mod stack;
mod str_utils;
mod transaction;
mod utils;

const MAX_WEIGHT_ALLOWED : u32 = 4_000_000;

fn main() {
    let mut txs: Vec<Transaction> = get_txs();

    // cosnider mutable iterator because, transactino id is computed and set accordingly
    txs.iter_mut().for_each(|tx| {
        let raw_tx: Vec<u8> = tx.get_raw_bytes(false);

        let mut result = double_hash256(&raw_tx);
        result.reverse();

        // after this step, result will have the actual txid
        let txid: String = result.iter().fold(String::new(), |acc, val| format!("{}{:02x}", acc, val));

        tx.txid = Some(txid.clone());

        // this is just a sanity check whether, the serialzed data is correct or not
        let result = hash256(&result);

        let hash_txid: String = result.iter().fold(String::new(), |acc, val| format!("{}{:02x}", acc, val));
        assert_eq!(hash_txid, *tx.sanity_hash.as_ref().unwrap());

        if tx.is_segwit.unwrap() {
            let raw_tx_witness = tx.get_raw_bytes(true);
            let wtxid: String = double_hash256(&raw_tx_witness)
                .iter()
                .rev()
                .fold(String::new(), |acc, val| format!("{}{:02x}", acc, *val));
            tx.wtxid = Some(wtxid);
        } else {
            tx.wtxid = Some(txid.clone());
        }
    });

    // this is the filtered txs of double spending
    let txs = remove_double_spending_tx(&mut txs);

    println!("Number of txs after removing double spending {}", txs.len());

    let mut verified_txs: Vec<&Transaction> = Vec::new();
    // verify each trannscations vin
    for tx in txs.iter() {
        if tx.validate_transacation() {
            verified_txs.push(tx);
        }
    }
    println!("Verified {}", verified_txs.len());

    let transactions_to_consider: Vec<&Transaction> = pick_best_transactions(verified_txs.as_slice(), 8000_000);

    // order the transactions topologically
    let mut ordered_txs: Vec<&Transaction> = reorder_txs(&transactions_to_consider);


    println!(
        "Number of transactions after reordering: {}",
        ordered_txs.len()
    );

    let coinbase_tx = prepare_coinbase_tx(&ordered_txs);
    ordered_txs.insert(0, &coinbase_tx);

    let mut out_file = File::create("output.txt").expect("Cannot create file");

    let blockheader = prepare_blockheader(&ordered_txs);
    let serialized_coinbase = ordered_txs[0].get_raw_bytes(true);

    let blockheader_str = hex_str!(blockheader);
    let serialized_tx_str = hex_str!(serialized_coinbase);

    let _ = out_file.write(blockheader_str.as_bytes());

    let _ = out_file.write("\n".as_bytes());

    let _ = out_file.write(serialized_tx_str.as_bytes());

    let _ = out_file.write("\n".as_bytes());

    ordered_txs.iter().for_each(|tx| {
        let _ = out_file.write(tx.txid.as_ref().unwrap().as_bytes());
        let _ = out_file.write("\n".as_bytes());
    });
}
