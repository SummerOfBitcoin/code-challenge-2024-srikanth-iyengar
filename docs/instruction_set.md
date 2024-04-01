## Script instruction set getting used in the given transaction

| Instruction       | Hex valuen | Description                                |
|-------------------|----------- | ------------------------------------------ |
| OP_HASH160        | 0xa9       | The input is hashed twice 256 + RIPEMD160  |
| OP_EQUAL          | 0x87       | Returns if the inputs are equal            |
| OP_RETURN         | 0x6a       | Ends script with stack top value as result |
| OP_CHECKSIG       | 0xac       | The entire transaction's outputs, inputs, and script (from the most recently-executed OP_CODESEPARATOR to the end) are hashed. The signature used by OP_CHECKSIG must be a valid signature for this hash and public key. If it is, 1 is returned, 0 otherwise. |
| OP_DUP            | 0x76       | Duplicates the top stack item              |
| OP_CHECKMULTISIG  | 0xae       | Compares the first signature against each public key until it finds an ECDSA match. Starting with the subsequent public key, it compares the second signature against each remaining public key until it finds an ECDSA match. The process is repeated until all signatures have been checked or not enough public keys remain to produce a successful result. All signatures need to match a public key. Because public keys are not checked again if they fail any signature comparison, signatures must be placed in the scriptSig using the same order as their corresponding public keys were placed in the scriptPubKey or redeemScript. If all signatures are valid, 1 is returned, 0 otherwise. Due to a bug, an extra unused value (x) is removed from the stack. Script spenders must account for this by adding a junk value (typically zero) to the stack. |
| OP_PUSHDATA1      | 0x4c       | The next byte contains the number of bytes to be pushed onto the stack. |
| OP_PUSHDATA2      | 0x4d       | The next byte contains the number of bytes to be pushed onto the stack. |
| OP_PUSHDATA4      | 0x4e       | The next byte contains the number of bytes to be pushed onto the stack. |
| OP_0              | 0x00       | Constant which means false                 |
| OP_PUSHBYTES_[X]  | 0x01-0x4b  | The next opcode bytes is data to be pushed onto the stack |


## Script assembler docs
Key components of assember
- [ ] a list of opcodes
- [ ] a stack
- [ ] transaction context

