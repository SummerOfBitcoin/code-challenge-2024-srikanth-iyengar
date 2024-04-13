use crate::str_utils::get_hex_bytes;

use super::{Pubkey, Transaction, Vin};

#[test]
pub fn segwit_serialize_test() {
    let tx_in = Vin {
        vout: 1,
        sequence: 0xffffffff,
        prevout: Pubkey {
            value: 30000,
            scriptpubkey_asm: String::from(
                "OP_0 OP_PUSHBYTES_20 aa966f56de599b4094b61aa68a2b3df9e97e9c48",
            ),
            scriptpubkey_type: String::from("v0_p2wpkh"),
            scriptpubkey: String::from("0014aa966f56de599b4094b61aa68a2b3df9e97e9c48"),
            scriptpubkey_address: Some(String::from("")),
        },
        scriptsig_asm: String::from(""),
        scriptsig: String::from(""),
        witness: Some(vec![]),
        txid: String::from("6ae73833e5f58616445bfe35171e89b23c5b59ef585637537f6ba34a019449ac"),
        is_coinbase: false,
        inner_redeemscript_asm: None,
    };

    let tx_vout = Pubkey {
        scriptpubkey_address: None,
        scriptpubkey: String::from("76a914ce72abfd0e6d9354a660c18f2825eb392f060fdc88ac"),
        scriptpubkey_type: String::from("p2pkh"),
        scriptpubkey_asm: String::from(""),
        value: 20000,
    };

    let tx = Transaction {
        txid: Some(String::from(
            "04f7bc0296fe70799762e628445fa9f0ccc2a2646ee5b369047d86ff964bd74e",
        )),
        vout: vec![tx_vout],
        vin: vec![tx_in],
        sanity_hash: None,
        version: 0x02,
        locktime: 0x00,
        is_segwit: Some(true),
    };

    let actual_preimage = String::from("02000000cbfaca386d65ea7043aaac40302325d0dc7391a73b585571e28d3287d6b162033bb13029ce7b1f559ef5e747fcac439f1455a2ec7c5f09b72290795e70665044ac4994014aa36b7f53375658ef595b3cb2891e1735fe5b441686f5e53338e76a010000001976a914aa966f56de599b4094b61aa68a2b3df9e97e9c4888ac3075000000000000ffffffff900a6c6ff6cd938bf863e50613a4ed5fb1661b78649fe354116edaf5d4abb95200000000");
    let preimage_bytes = get_hex_bytes(&actual_preimage).unwrap();

    let calculate_preimage = tx.get_raw_tx_for_vin(0);

    assert_eq!(preimage_bytes, calculate_preimage);
}
