use std::{collections::HashMap, usize};

use hex_literal::hex;

use crate::stack::Stack;

use super::{do_dfs, merkleroot};

///          1
///         / \
///        /   \
///       2     3
///      / \   / \
///     4   5 6   7
///    /     /
///   8 -----

#[test]
pub fn test_topological_order() {
    // order of tx initially [7, 6, 5, 4, 3, 2, 1]
    let txs = vec!["8", "7", "6", "5", "4", "3", "2", "1"];

    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

    // neighbours of "1"
    adj_list.insert(
        String::from("1"),
        vec![String::from("2"), String::from("3")],
    );

    // neighbours of "2"
    adj_list.insert(
        String::from("2"),
        vec![String::from("4"), String::from("5")],
    );

    // neighbours of "3"
    adj_list.insert(
        String::from("3"),
        vec![String::from("6"), String::from("7")],
    );

    // neighbours of "4"
    adj_list.insert(String::from("4"), vec![String::from("8")]);

    // neighbourse of "6"
    adj_list.insert(String::from("6"), vec![String::from("8")]);

    let mut tx_rev_map: HashMap<String, usize> = HashMap::new();
    for (idx, tx) in txs.iter().enumerate() {
        tx_rev_map.insert(tx.to_string(), idx);
    }

    let mut visited: HashMap<String, bool> = HashMap::new();

    let mut stack: Stack<usize> = Stack::new();

    do_dfs(
        String::from("1"),
        &adj_list,
        &tx_rev_map,
        &mut visited,
        &mut stack,
    );

    let mut expected_stack: Stack<usize> = Stack::new();
    expected_stack.push(0);
    expected_stack.push(4);
    expected_stack.push(3);
    expected_stack.push(6);
    expected_stack.push(2);
    expected_stack.push(1);
    expected_stack.push(5);
    expected_stack.push(7);

    while stack.top.is_some() {
        if let Some(top) = stack.pop() {
            assert_eq!(top, expected_stack.pop().unwrap());
        }
    }
    println!()
}

#[test]
pub fn test_merkle_root() {
    let mut txids: Vec<Vec<u8>> = vec![
        hex!("8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87").to_vec(),
        hex!("fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4").to_vec(),
        hex!("6359f0868171b1d194cbee1af2f16ea598ae8fad666d9b012c8ed2b79a236ec4").to_vec(),
        hex!("e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d").to_vec(),
    ];

    txids.iter_mut().for_each(|x| {
        x.reverse();
    });

    // f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766
    let mut merkle_root = merkleroot(txids)[0].clone();
    merkle_root.reverse();
    let merkle_root_str: String = merkle_root.iter().map(|x| format!("{:02x}", x)).collect();
    assert_eq!(
        merkle_root_str,
        String::from("f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766")
    );
}
