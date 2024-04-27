use std::{collections::HashMap, usize};

use crate::{hash_utils::double_hash256, stack::Stack, str_utils::get_hex_bytes, transaction::Transaction};

#[path = "./test/merkle_test.rs"]
#[cfg(test)]
mod merkle_test;

pub fn merkleroot(txs: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    if txs.len() == 1 {
        return txs;
    }

    let mut result: Vec<Vec<u8>> = Vec::new();

    let len = txs.len();

    for i in (0..len).step_by(2) {
        let first = &txs[i];
        let second = if i + 1 < len {
            &txs[i+1]
        } else {
            &txs[i]
        };

        let mut msg: Vec<u8> = Vec::new();

        first.iter().for_each(|x| msg.push(*x));
        second.iter().for_each(|x| msg.push(*x));

        result.push(double_hash256(&msg));
    }
    merkleroot(result)
}

pub fn prepare_merkle_root(txs: &[&Transaction], is_wtxid: bool) -> Vec<u8> {
    let txids: Vec<Vec<u8>> = txs
        .iter()
        .map(|tx| {
            let input = if is_wtxid {
                tx.wtxid.as_ref()
            } else {
                tx.txid.as_ref()
            };
            let mut result = get_hex_bytes(input.unwrap()).unwrap().clone();
            result.reverse();
            result
        })
        .collect();

    merkleroot(txids).first().unwrap().clone()
}

// this function will reorder the tx, such that parent tx appears first before the child txs appear
pub fn reorder_txs<'a>(txs: &'a [&'a Transaction]) -> Vec<&'a Transaction> {
    // adjacency list in the form of txid(parent) -> txid(child1), txid(child2).. and so on
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    let mut incoming_edges: HashMap<String, u32> = HashMap::new();
    let mut tx_rev_map: HashMap<String, usize> = HashMap::new();
    let mut visited: HashMap<String, bool> = HashMap::new();

    for (idx, tx) in txs.iter().enumerate() {
        // edge from tx.txid to vins.txid, does vout matter here ?, I guess not
        // Safely txid because we know that when we reach here it is obvious that
        // txid will be present
        let txid = tx.txid.as_ref().unwrap();
        let mut neighbours: Vec<String> = Vec::new();

        tx.vin.iter().for_each(|child_tx| {
            neighbours.push(child_tx.txid.clone());
            if let Some(val) = incoming_edges.get_mut(&child_tx.txid.clone()) {
                *val += 1;
            } else {
                incoming_edges.insert(child_tx.txid.clone(), 1);
            }
        });

        // I hate strings, .clone.clone.clone.clone.clone
        adj_list.insert(txid.clone(), neighbours);
        tx_rev_map.insert(txid.clone(), idx);
        visited.insert(txid.clone(), false);
    }

    let mut stack: Stack<usize> = Stack::new();

    // topological order transactions
    for tx in txs.iter() {
        let txid = tx.txid.as_ref().unwrap();

        // TODO: revisit this, this looks way wrong, clone and sending reference :)
        if let Some(incoming_edge) = incoming_edges.get(&txid.clone()) {
            if *incoming_edge != 0 {
                continue;
            }
        }

        // if not visited earlier do dfs
        if let Some(is_visited) = visited.get(txid) {
            if !*is_visited {
                do_dfs(
                    txid.clone(),
                    &adj_list,
                    &tx_rev_map,
                    &mut visited,
                    &mut stack,
                );
            }
        }
    }

    let mut reordered_txs: Vec<&Transaction> = Vec::new();
    while !stack.is_empty() {
        if let Some(top) = stack.pop() {
            reordered_txs.push(txs[top]);
        }
    }

    reordered_txs
}

// this dfs function is specifically designed for topological ordering
fn do_dfs(
    txid: String,
    adj_list: &HashMap<String, Vec<String>>,
    tx_rev_idx: &HashMap<String, usize>,
    visited: &mut HashMap<String, bool>,
    stack: &mut Stack<usize>,
) {
    if let Some(val) = visited.get_mut(&txid) {
        *val = true;
    } else {
        visited.insert(txid.clone(), true);
    }

    if let Some(neighbours) = adj_list.get(&txid.clone()) {
        for neighbour in neighbours.iter() {
            if let Some(val) = visited.get_mut(&neighbour.clone()) {
                if !*val {
                    do_dfs(neighbour.clone(), adj_list, tx_rev_idx, visited, stack);
                }
            } else {
                do_dfs(neighbour.clone(), adj_list, tx_rev_idx, visited, stack);
            }
        }
    }

    if let Some(idx) = tx_rev_idx.get(&txid) {
        stack.push(*idx);
    }
}
