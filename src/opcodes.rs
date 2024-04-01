#[derive(Debug)]
pub struct Opcode {
    pub code: u8,
    pub max_range: Option<u8>,
}

impl PartialEq<Opcode> for Opcode {
    fn eq(&self, other: &Opcode) -> bool {
        match self.max_range {
            Some(val) => self.code <= other.code && val >= other.code,
            None => self.code == other.code,
        }
    }
}

pub mod all_opcodes {
    use super::Opcode;

    pub const OP_HASH160: Opcode = Opcode {
        code: 0xa9,
        max_range: None,
    };
    pub const OP_EQUAL: Opcode = Opcode {
        code: 0x87,
        max_range: None,
    };
    pub const OP_EQUALVERIFY: Opcode = Opcode {
        code: 0x88,
        max_range: None,
    };
    pub const OP_RETURN: Opcode = Opcode {
        code: 0x6a,
        max_range: None,
    };
    pub const OP_CHECKSIG: Opcode = Opcode {
        code: 0xac,
        max_range: None,
    };
    pub const OP_DUP: Opcode = Opcode {
        code: 0x76,
        max_range: None,
    };
    pub const OP_CHECKMULTISIG: Opcode = Opcode {
        code: 0xae,
        max_range: None,
    };
    pub const OP_PUSHDATA1: Opcode = Opcode {
        code: 0x4c,
        max_range: None,
    };
    pub const OP_PUSHDATA2: Opcode = Opcode {
        code: 0x4d,
        max_range: None,
    };
    pub const OP_PUSHDATA4: Opcode = Opcode {
        code: 0x4e,
        max_range: None,
    };
    pub const OP_0: Opcode = Opcode {
        code: 0x00,
        max_range: None,
    };
    pub const OP_1: Opcode = Opcode {
        code: 0x51,
        max_range: None,
    };
    pub const OP_PUSHBYTES: Opcode = Opcode {
        code: 0x01,
        max_range: Some(0x4b),
    };
}
