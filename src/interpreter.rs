use std::{usize, vec};

use crate::{
    opcodes::{
        all_opcodes::{
            OP_CHECKMULTISIG, OP_CHECKSIG, OP_DUP, OP_EQUAL, OP_HASH160, OP_PUSHBYTES,
            OP_PUSHDATA1, OP_PUSHDATA2, OP_PUSHDATA4, OP_RETURN,
        },
        Opcode,
    },
    stack::Stack,
    transaction::Transaction,
};

use libsecp256k1::{verify, Message, PublicKey, PublicKeyFormat, Signature};
// use libsecp256k1::{verify, Message, PublicKey, Signature};
use ripemd::{Digest as Ripemd160Digest, Ripemd160};
use sha2::Sha256;

use super::opcodes::all_opcodes::OP_EQUALVERIFY;

/// This assembler handles the following types of unlock scripts
/// 1. P2PKH
/// 2. P2SH
/// 3. P2WKH
/// 4. P2WSH
/// 5. P2TR
pub struct Interpreter<'a> {
    pub instructions: &'a [u8],
    pub stack: Stack<Vec<u8>>,
    exec_ctx: &'a u8,
    instructions_executed: usize,
    is_halted: bool,
    vin_idx: u32,
    tx: &'a Transaction,
}

impl<'a> Interpreter<'a> {
    pub fn new(instructions: &'a [u8], vin_idx: u32, tx: &'a Transaction) -> Self {
        Interpreter {
            instructions,
            stack: Stack::new(),
            exec_ctx: &instructions[0],
            instructions_executed: 0,
            is_halted: false,
            vin_idx,
            tx,
        }
    }

    fn jump_next(&mut self) {
        if self.is_halted {
            return;
        }

        let opcode: Opcode = Opcode {
            code: self.get_ctx_val(),
            max_range: None,
        };

        // println!("opcode {:?}", opcode);

        if OP_HASH160 == opcode {
            // Take the top element of the stack hash it using sha256 then use ripemd160 ->
            // push the 20 byte output into the stack
            let top = self.stack.pop();
            match top {
                Some(val) => {
                    let mut hasher = Sha256::new();
                    hasher.update(val);
                    let digest = hasher.finalize();
                    let mut hasher = Ripemd160::new();
                    hasher.update(digest);
                    let pkh = hasher.finalize();
                    self.stack.push(pkh.to_vec());
                }
                None => {
                    // crash the program
                }
            }
        } else if OP_EQUAL == opcode {
            // Take the top two element from the stack and compare if they are equal
            let arg1 = self.stack.pop();
            let arg2 = self.stack.pop();
            if arg1.is_some() && arg2.is_some() {
                match arg1
                    .unwrap()
                    .iter()
                    .zip(&arg2.unwrap())
                    .filter(|&(a, b)| *a != *b)
                    .count()
                {
                    0 => self.stack.push(vec![0x01]),
                    _ => self.stack.push(vec![0x00]),
                }
            } else {
                self.stack.push(vec![0x00]);
            }
        } else if OP_EQUALVERIFY == opcode {
            let arg1 = self.stack.pop();
            let arg2 = self.stack.pop();
            if arg1.is_some() && arg2.is_some() {
                match arg1
                    .unwrap()
                    .iter()
                    .zip(&arg2.unwrap())
                    .filter(|&(a, b)| *a != *b)
                    .count()
                {
                    0 => self.is_halted = false,
                    _ => self.is_halted = true,
                }
            } else {
                self.is_halted = true;
            }
        } else if OP_DUP == opcode {
            // Take the top element duplicate it and push into the stack
            let top = self.stack.pop().unwrap();
            let mut copy = Vec::new();
            for num in top.iter() {
                copy.push(*num);
            }
            self.stack.push(top);
            self.stack.push(copy);
        } else if OP_RETURN == opcode {
            // Stop the program and stack top is the result
            self.is_halted = true;
            return;
        } else if OP_PUSHBYTES == opcode {
            let len = opcode.code - OP_PUSHBYTES.code + 1;
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..len {
                data.push(self.get_ctx_val());
            }
            self.stack.push(data);
        } else if OP_PUSHDATA1 == opcode {
            let len = self.get_ctx_val();
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..len {
                data.push(self.get_ctx_val());
            }
            self.stack.push(data);
        } else if OP_PUSHDATA2 == opcode {
            let arg1 = self.get_ctx_val();
            let len: u16 = 0x0100 * self.get_ctx_val() as u16 + arg1 as u16;
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..len {
                data.push(self.get_ctx_val());
            }
            self.stack.push(data);
        } else if OP_PUSHDATA4 == opcode {
            let arg1 = self.get_ctx_val();
            let arg2 = self.get_ctx_val();
            let len: u32 =
                0x010000 * self.get_ctx_val() as u32 + 0x0100 * arg2 as u32 + arg1 as u32;
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..len {
                data.push(self.get_ctx_val());
            }
            self.stack.push(data);
        } else if OP_CHECKSIG == opcode {
            if let (Some(pubkey), Some(signature)) = (self.stack.pop(), self.stack.pop()) {
                let mut serialized_tx = self.tx.get_raw_tx_for_vin(self.vin_idx);
                let sighash_type = *signature.last().unwrap() as u32;
                sighash_type
                    .to_le_bytes()
                    .iter()
                    .for_each(|val| serialized_tx.push(*val));

                let mut hasher = Sha256::new();
                hasher.update(serialized_tx);
                let result = hasher.finalize();

                let mut hasher = Sha256::new();
                hasher.update(result);
                let serialized_hash = hasher.finalize();

                if let (Ok(msg), Ok(sig), Ok(pk)) = (
                    Message::parse_slice(serialized_hash.as_slice()),
                    Signature::parse_der_lax(signature.as_slice()),
                    PublicKey::parse_slice(pubkey.as_slice(), Some(PublicKeyFormat::Compressed)),
                ) {
                    let is_valid = verify(&msg, &sig, &pk);
                    if is_valid {
                        self.stack.push(vec![0x01]);
                    } else {
                        self.stack.push(vec![0x00]);
                    }
                } else {
                    self.is_halted = true;
                    self.stack.push(vec![0]);
                }
            } else {
                self.is_halted = true;
                self.stack.push(vec![0]);
            }
        } else if OP_CHECKMULTISIG == opcode {
        } else {
            // should we crash :) ?
        }
    }

    fn inc_ctx(&mut self) {
        self.instructions_executed += 1;
        if self.instructions_executed < self.instructions.len() {
            self.exec_ctx = &self.instructions[self.instructions_executed];
        } else {
            self.exec_ctx = &self.instructions[0];
        }
    }

    pub fn exec_all(&mut self) -> Option<Vec<u8>> {
        while self.instructions_executed < self.instructions.len() && !self.is_halted {
            self.jump_next();
        }
        self.stack.pop()
    }

    fn get_ctx_val(&mut self) -> u8 {
        let val = *self.exec_ctx;
        self.inc_ctx();
        val
    }
}
