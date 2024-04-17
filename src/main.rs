use std::{
    fs::File,
    io::Write, sync::atomic::Ordering,
};
use rand::thread_rng;
use transaction::Transaction;

use crate::{
    hash_utils::{double_hash256, hash256},
    merkle::reorder_txs,
    utils::{get_txs, prepare_blockheader, prepare_coinbase_tx, remove_double_spending_tx},
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
        let txid: String = result.iter().map(|val| format!("{:02x}", val)).collect();
        tx.txid = Some(txid.clone());

        // this is just a sanity check whether, the serialzed data is correct or not
        let result = hash256(&result);

        let hash_txid: String = result.iter().map(|val| format!("{:02x}", val)).collect();
        assert_eq!(hash_txid, *tx.sanity_hash.as_ref().unwrap());

        if tx.is_segwit.unwrap() {
            let raw_tx_witness = tx.get_raw_bytes(true);
            let wtxid: String = double_hash256(&raw_tx_witness)
                .iter()
                .rev()
                .map(|val| format!("{:02x}", *val))
                .collect();
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
            verified_txs.push(&tx);
        }
    }
    println!("Verified {}", verified_txs.len());

    // verified_txs.sort_by(|a, b| {
    //     // compare a.fee / a.weight with b.fee / b.weight
    //     let ratio_a = (a.weight.unwrap() as u64) / a.tx_fee.unwrap() * 2;
    //     let ratio_b = (b.weight.unwrap() as u64) / b.tx_fee.unwrap() / 2;

    //     ratio_a.cmp(&ratio_b)
    // });

    let mut transactions_to_consider: Vec<&Transaction> = Vec::new();

    let mut weights_filled : u32 = 0;

    let mut idx = 0;

    while weights_filled + 1000 <= MAX_WEIGHT_ALLOWED {
        if idx >= verified_txs.len() {
            break;
        }

        if weights_filled + verified_txs[idx].weight.unwrap() as u32 + 1000 <= MAX_WEIGHT_ALLOWED {
            transactions_to_consider.push(verified_txs[idx]);
            weights_filled += verified_txs[idx].weight.unwrap() as u32;
        } else {
            // do nothing consider the next tx
        }

        idx += 1;
    }

    debug!(weights_filled);

    // now we have the verified transactions

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

    let _ = ordered_txs.iter().for_each(|tx| {
        let _ = out_file.write(tx.txid.as_ref().unwrap().as_bytes());
        let _ = out_file.write("\n".as_bytes());
    });
}
