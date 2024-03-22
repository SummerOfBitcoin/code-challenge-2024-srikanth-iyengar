use std::{collections::HashSet, fs};
use tx_type::Transaction;
mod tx_type;

fn main() {
    let mut transactions: Vec<Transaction> = Vec::new();

    let paths = fs::read_dir("mempool").unwrap();

    for path in paths {
        let raw_json_tx =
            fs::read_to_string(path.as_ref().unwrap().path().display().to_string()).expect("");
        let result = Transaction::new(&raw_json_tx);
        match result {
            Ok(result) => {
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

    let mut unique_script_type: HashSet<String> = HashSet::new();
    for tx in transactions {
        for pubkeys in tx.vin {
            unique_script_type.insert(pubkeys.prevout.scriptpubkey_type);
        }
    }

    println!("{:?}", unique_script_type);
}
