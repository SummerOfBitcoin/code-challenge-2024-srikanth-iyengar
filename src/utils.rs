use std::{collections::HashSet, fs, time::{SystemTime, UNIX_EPOCH}};

use hex_literal::hex;
use num_bigint::BigUint;

use crate::{debug, debug_hex, hash_utils::double_hash256, merkle::{merkleroot, prepare_merkle_root}, opcodes::all_opcodes::{OP_PUSHBYTES, OP_RETURN}, str_utils::get_hex_bytes, transaction::{Pubkey, Transaction, Vin}};


pub fn get_txs() -> Vec<Transaction> {
    let directory = "mempool/";
    // let directory = "mempool/";

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
                let pubkey_types: HashSet<String> = val
                    .vin
                    .iter()
                    .map(|vin| vin.prevout.scriptpubkey_type.clone())
                    .collect();
                val.is_segwit =
                    Some(pubkey_types.contains("v0_p2wpkh") || pubkey_types.contains("v0_p2wsh"));

                txs.push(val);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    txs
}


pub fn remove_double_spending_tx<'a>(txs: &'a mut Vec<Transaction>) -> Vec<&'a Transaction> {
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

            // tx.vout.iter().enumerate().for_each(|(idx, _)| {
            //     let key = tx.txid.as_ref().unwrap().clone() + "#" + idx.to_string().as_str();
            //     used_tx.insert(key);
            // });

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

pub fn prepare_coinbase_tx(txs: &Vec<&Transaction>) -> Transaction {
    let mut fees: u64 = 0;
    for tx in txs.iter() {
        let vin_amount: u64 = tx.vin.iter().map(|vin| vin.prevout.value).sum();
        let vout_amount: u64 = tx.vout.iter().map(|vout| vout.value).sum();

        // just making sure that I get something :) as a miner
        assert!(vin_amount > vout_amount);
        fees += vin_amount - vout_amount;
    }
    debug!(fees);

    let mut wtxids: Vec<Vec<u8>> =
        vec![hex!("0000000000000000000000000000000000000000000000000000000000000000").to_vec()];

    txs.iter().for_each(|tx| {
        if let Ok(mut wtxid_bytes) = get_hex_bytes(tx.wtxid.as_ref().unwrap()) {
            wtxid_bytes.reverse();
            wtxids.push(wtxid_bytes);
        }
    });

    let wtxid_merkle_root = merkleroot(wtxids).get(0).unwrap().clone();

    let coinbase_vin = Vin {
        vout: 4294967295,
        txid: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
        witness: Some(vec![String::from(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )]),
        prevout: Pubkey {
            value: 0,
            scriptpubkey_asm: String::from(""),
            scriptpubkey: String::from(""),
            scriptpubkey_type: String::from(""),
            scriptpubkey_address: Some(String::from("")),
        },
        // copied height  of 31e9370f45eb48f6f52ef683b0737332f09f1cead75608021185450422ec1a71
        scriptsig: String::from(
            "03233708184d696e656420627920416e74506f6f6c373946205b8160a4256c0000946e0100",
        ),
        scriptsig_asm: String::from(""),
        sequence: 0xffffffff,
        is_coinbase: true,
        inner_redeemscript_asm: None,
    };

    let mut scriptpub_key_lock = vec![OP_RETURN.code, OP_PUSHBYTES.code + 0x24 - 1,
        0xaa,
        0x21,
        0xa9,
        0xed,
    ];


    let mut witness_lock: Vec<u8> = vec![];
    witness_lock.extend(wtxid_merkle_root.iter());
    witness_lock.extend(hex!["0000000000000000000000000000000000000000000000000000000000000000"].iter());

    let witness_lock_hash = double_hash256(&witness_lock);

    scriptpub_key_lock.extend(witness_lock_hash.iter());

    let scriptpubkey_lock_str: String = scriptpub_key_lock
        .iter()
        .map(|val| format!("{:02x}", val))
        .collect();

    debug!(scriptpubkey_lock_str);

    let vout: Vec<Pubkey> = vec![
        Pubkey {
            scriptpubkey_address: Some(String::from("")),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey: String::from("76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac"),
            scriptpubkey_asm: String::from(""),
            value: fees + 1250000000 as u64,
        },
        Pubkey {
            scriptpubkey_address: Some(String::from("")),
            scriptpubkey_type: String::from("OP_RETURN"),
            scriptpubkey: scriptpubkey_lock_str,
            scriptpubkey_asm: String::from(""),
            value: 0,
        },
    ];

    let mut coinbase_tx = Transaction {
        wtxid: None,
        txid: None,
        vout,
        vin: vec![coinbase_vin],
        is_segwit: Some(true),
        version: 0x01,
        locktime: 0x00000000,
        sanity_hash: Some(String::from("none")),
        tx_fee: None,
        weight: None,
    };

    coinbase_tx.assign_tx_fee();
    coinbase_tx.assign_weight();

    let coinbase_raw_tx = coinbase_tx.get_raw_bytes(false);
    let txid = double_hash256(&coinbase_raw_tx);

    let txid_str: String = txid
        .iter()
        .rev()
        .map(|val| format!("{:02x}", *val))
        .collect();
    coinbase_tx.txid = Some(txid_str);

    coinbase_tx
}


pub fn prepare_blockheader(txs: &Vec<&Transaction>) -> Vec<u8> {
    let version_bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x04];
    let prev_block_hash: Vec<u8> =
        hex!["0000000000000000000000000000000000000000000000000000000000000000"].to_vec();
    let merkle_root: Vec<u8> = prepare_merkle_root(&txs, false);
    let ts_bytes: Vec<u8> =
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32).to_le_bytes().to_vec();
    let bits: Vec<u8> =
        hex!("0000ffff00000000000000000000000000000000000000000000000000000000").to_vec();

    for i in 0..u32::max_value() {
        let nonce_bytes: Vec<u8> = i.to_le_bytes().to_vec();

        let mut raw_bytes: Vec<u8> = Vec::new();

        version_bytes.iter().for_each(|x| raw_bytes.push(*x));
        prev_block_hash.iter().for_each(|x| raw_bytes.push(*x));

        merkle_root.iter().for_each(|x| raw_bytes.push(*x));
        ts_bytes.iter().for_each(|x| raw_bytes.push(*x));
        hex!["1f00ffff"].iter().rev().for_each(|x| raw_bytes.push(*x));
        nonce_bytes.iter().for_each(|x| raw_bytes.push(*x));

        let mut block_hash = double_hash256(&raw_bytes);

        block_hash.reverse();

        let block_hash_str: String = block_hash.iter().map(|x| format!("{:02x}", *x)).collect();

        let block_hash = BigUint::from_bytes_be(block_hash.as_slice());
        let target = BigUint::from_bytes_be(bits.as_slice());

        if block_hash.le(&target) {
            println!("mining successfull with hash: {}", block_hash_str);
            debug_hex!(version_bytes);
            debug_hex!(prev_block_hash);
            debug_hex!(merkle_root);
            debug_hex!(ts_bytes);
            debug_hex!(bits);
            debug_hex!(nonce_bytes);
            return raw_bytes;
        }
    }
    vec![]
}
