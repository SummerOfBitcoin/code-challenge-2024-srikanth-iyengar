
// list of txids

use std::collections::HashMap;

use crate::transaction::Transaction;

// this function will reorder the tx, such that parent tx appears first before the child txs appear
pub fn reorder_tx<'a> (txs: Vec<&'a Transaction>) -> Vec<&'a Transaction> {
    // adjacency list in the form of txid(parent) -> txid(child1), txid(child2).. and so on
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    let mut incoming_edges: HashMap<String, u32> = HashMap::new();
    for tx in txs.iter() {
        // edge from tx.txid to vins.txid, does vout matter here ?, I guess not
        // Safely txid because we know that when we reach here it is obvious that txid will be
        // present
        let txid = tx.txid.as_ref().unwrap().clone();
        let mut neighbours: Vec<String> = Vec::new();

        tx.vin.iter().for_each(|child_tx| {
            neighbours.push(child_tx.txid.clone());
        });

        adj_list.insert(txid, neighbours);
    }

    let mut visited: HashMap<String, bool> = HashMap::new();
    

    txs
}

