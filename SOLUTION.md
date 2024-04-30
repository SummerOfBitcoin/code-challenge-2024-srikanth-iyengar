# SOB 2024 Mining Mempool Transactions

## Design Approach

The first step I took approaching this assignment is to identify steps a miner would take to mine a block. The key concepts of creating a valid block are as follows:

- Fetch the transactions from mempool -> Read the mempool directory and store it somewhere in the memory.
- Remove transaction with double spending
- Validate the unlocking script of all the inputs of a transaction
- Since we cannot mine all the verified transactions, I need to pick transactions in such a way that weight and transaction fee both are maximized.
- Once I have decided on some transaction to mine, I have to order them in topological order
- After topological ordering I have to prepare the coinbase transaction
- At last create the blockheader

I choose rust as the programming language for this assignment as the organiation I am targeting also using rust. It was a fun experience to write the mining logic in rust.

So now lets deep dive all the steps in more detail
### Fetch the transactions from mempool
- To complete this I had to create types in rust which resembble the schema in the json files. Once that was doneI had to derive the Deserialize trait for each of the struct/types. I used serde\_json to read the json files and store them in a vector of transactions.
- So once I had the transaction in memory I serialized the transaction in the standard format and calculated txid using Double SHA256.

### Remove transaction with double spending
- To remove the transaction with double spending I had to create a hashset to check whether element with `txid#vout` is already present in the set or not. If it is present then I had ignore the transaction completely and mark as invalid.

### Validate the unlocking script of all the inputs of a transaction
- To validate a unlocking script I basically required a intepreter which can run on its one and will just return the last result once the script is executed.
- To reduce the scope of the assignment I built a interpreter which hahd a very smaller of instructions avaialbe which are only present in the transactions of the mempool.
- Then I built a interpreter with those constrained instruction set and ran the script to validate the unlocking script.
- Depending on the stack top result and went on marking the transaction as valid or invalid.

### Pick transactions in such a way that weight and transaction fee both are maximized
- So my first approach was to sort the transactions in descending order of the fee and then pick the transactions with highest fee and then check if the weight is not exceeding the limit.
- I though this greedy solution would work and will give me good enough score. But on analyzing the problem this problem seemed pretty similar to the knapsack problem. So I gave it thought on can I solve this using the knapsack standard problem. But since standard problem will be inefficient in this case because constraints are too high.
- So I planned on switching to randomizes algorithm where I considered the trnasaction in the sorted order first because why not. Then I randomly shuffled the transactions for a couple of rounds and picked the optimal solution from the random shuffle.

### Topological ordering
- Once I had the transactions to mine I had to order them in topological order. I used the standard topological ordering algorithm to order the transactions.
- A basic DFS algorithm was used to order the transactions.
- After the DFS I had stack with the transactions in the topological order. So I popped one by one and prepared the final tranactions to be mined.

### Prepare the coinbase transaction
- The coinbase transaction is the first transaction in the block. It is the transaction that rewards the miner with the block reward.
- So the coinbase transaction contains a input and I constrained the coinbase output to have only 2 outputs once of which is the block reward + transaction fee and the other is a op\_return output containing the witness commitment.
- Once I had the coinbase transaction I modified the vector in such a way that the coinbase transaction is the first transaction in the vector.

### Merkle root calculation
- The merkle root is the hash of all the transaction Ids in the block. The merkle root is calculated by hashing the transaction Ids in pairs and then hashing the result again until we have only one hash left.
- I used the standard merkle root calculation algorithm to calculate the merkle root.

### Create the blockheader
- The blockheader is the header of the block which contains the following fields:
    - version
    - prev\_block
    - merkle\_root
    - time
    - bits
    - nonce
- To calculate the merkle root I copied the transaction Ids in a seperate list and then calculated the merkle root using the standard merkle root calculation algorithm.
- I considered prev\_block hash to be `0000000000000000000000000000000000000000000000000000000000000000` which means this is the genesis block.
- I considered the time to be the current time in seconds.
- I considered the bits to be `0x1d00ffff` which was the given difficulty.
- I considered the nonce to be 0 initially and then started incrementing the nonce until the hash of the blockheader is less than the target.
- For comparing the very big 256 bit number I used the num-bigint crate in rust.
- There could be a optimal way to compare the compressed target with the hash of the blockheader but I was not able to find it.

## Implementation Details
I followed the following steps for the implementation:
1. Create the structs in rust for the transaction to be parsed in.
```rust
#[derive(Deserialize)]
pub struct Pubkey {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Deserialize)]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    pub prevout: Pubkey,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    pub sequence: u32,
    pub inner_redeemscript_asm: Option<String>,
}

#[derive(Deserialize)]
pub struct Transaction {
    pub txid: Option<String>,
    pub wtxid: Option<String>,
    // this is the sha256 hash of the txid (reverse order, again just a bitcoin thing)
    pub sanity_hash: Option<String>,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Pubkey>,
    pub is_segwit: Option<bool>,
    pub weight: Option<usize>,
    pub tx_fee: Option<u64>,
}
```
2. Read each transaction from mempool in the form of strings and parse using serde\_json.
```rust
impl Transaction {
    // Constructor for Transaction from raw json string
    pub fn new(raw_json_tx: &str) -> Result<Transaction, serde_json::Error> {
        let mut tx: Transaction = serde_json::from_str(raw_json_tx)?;
        Ok(tx)
    }
}
```
3. Serialize the transaction in the standard format and calculate the txid.
```rust
impl Transaction {
    pub fn get_raw_bytes(&self, include_witness: bool) -> Vec<u8> {
        // followed learn me a bitcoin and constructed a vector of bytes
    }
}
```
4. To verify that the serialization of transaction is correct I hashed the txid and matched it with the filename.
```rust
let mut txs: Vec<Transaction> = get_txs();
txs.iter_mut().for_each(|tx| {
    let raw_tx: Vec<u8> = tx.get_raw_bytes(false);

    let mut result = double_hash256(&raw_tx);
    result.reverse();

    // after this step, result will have the actual txid
    let txid: String = result.iter().fold(String::new(), |acc, val| format!("{}{:02x}", acc, val));

    tx.txid = Some(txid.clone());

    // this is just a sanity check whether, the serialzed data is correct or not
    let result = hash256(&result);

    let hash_txid: String = result.iter().fold(String::new(), |acc, val| format!("{}{:02x}", acc, val));
    assert_eq!(hash_txid, *tx.sanity_hash.as_ref().unwrap());
});
```
5. Remove the transaction with double spending.
```rust
pub fn remove_double_spending_tx(txs: &mut [Transaction]) -> Vec<&Transaction> {
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

                should_accept &= used_tx.get(&key).is_none();

                // push the txid#vout in the map
                used_tx.insert(vin.txid.clone() + "#" + vout_str.as_str());
            });

            if should_accept {
                Some(tx)
            } else {
                None
            }
        })
        .flatten()
        .collect();
    filtered_txs
}
```
6. Developing the actual interpreter with the minimal instrction set.
```rust
pub struct Interpreter<'a> {
    pub instructions: &'a [u8],
    pub stack: Stack<Vec<u8>>,
    exec_ctx: &'a u8,
    instructions_executed: usize,
    is_halted: bool,
    vin_idx: u32,
    tx: &'a Transaction,
}

fn jump_next(&mut self) {
    // executes the current instruction and increments the instruction pointer to next
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
```
7. Randomly shuffle the transactions and pick the optimal solution using `psuedo random generator rand`.
```rust
pub fn pick_best_transactions<'a> (txs: &'a [&Transaction], rounds: u32) -> Vec<&'a Transaction>  {
    let mut best_fee_till_now = 0;
    let mut best_weight_till_now = 0;

    let mut result: Vec<&Transaction> = Vec::new();

    let mut shuffled_txs: Vec<&Transaction> = Vec::new();
    shuffled_txs.extend_from_slice(txs);
    
    for _ in (0..rounds) {
        // shuffle the transactions

        let current_fee = txs.iter().map(|tx| tx.tx_fee.unwrap()).sum();
        let current_weight = txs.iter().map(|tx| tx.weight.unwrap()).sum();
        // Assuming max fee I can get is 3000000 satoshis
        let miner_fee: u64 = transactions_to_consider.iter().map(|tx| tx.tx_fee.unwrap()).sum();
        let score : f64 = weights_filled as f64 / MAX_WEIGHT_ALLOWED as f64  + miner_fee as f64 / 3000000 as f64;
        let prev_score : f64 = current_weight as f64 / MAX_WEIGHT_ALLOWED as f64 + current_fee  as f64 / 3000000 as f64;

        if score >= prev_score {
            result = transactions_to_consider;
            current_fee = miner_fee;
            current_weight = weights_filled;
        }
    }
}
```
8. Implementing the topological sort algorithm using DFS.
```rust
struct Graph {
    adj: Vec<Vec<usize>>,
    visited: Vec<bool>,
    ans: Vec<usize>,
}

impl Graph {
    fn new(n: usize) -> Self {
        Graph {
            adj: vec![vec![]; n],
            visited: vec![false; n],
            ans: vec![],
        }
    }

    fn dfs(&mut self, v: usize) {
        self.visited[v] = true;
        for &u in self.adj[v].iter() {
            if !self.visited[u] {
                self.dfs(u);
            }
        }
        self.ans.push(v);
    }

    fn topological_sort(&mut self) {
        self.visited = vec![false; self.visited.len()];
        self.ans.clear();
        for i in 0..self.adj.len() {
            if !self.visited[i] {
                self.dfs(i);
            }
        }
        self.ans.reverse();
    }
}
```
9. Preparing the coinbase transaction.
```rust
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
// I just utilized the helper function in transaction to assign the Optional fieilds
coinbase_tx.assign_optional_fields();
```
10. Calculating the merkle root.
```rust
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
```
11. Creating the blockheader and mining the block.
```rust
let version_bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x04];
let prev_block_hash: Vec<u8> =
    hex!["0000000000000000000000000000000000000000000000000000000000000000"].to_vec();
let merkle_root: Vec<u8> = prepare_merkle_root(txs, false);
let ts_bytes: Vec<u8> =
(SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs() as u32).to_le_bytes().to_vec();
let bits: Vec<u8> =
    hex!("0000ffff00000000000000000000000000000000000000000000000000000000").to_vec();
// once I had these details I just kept changing the nonce checked if block hash is less then target
```

## Results and Performance
After getting the first CI run to pass I kept optimizing the program to maximize the score
- Sorting based on the `weights / fee` this approach yeilded a score of 91 and was executed under 3-4 seconds and 30-40 seconds for compilation time.
- After when I switched over to randomized approach I experimeneted with variation in the number of rounds I performed following is the result for it

| Rounds  | Score | Time in secs |
|---------|-------|--------------|
| 1       | 91    | 3-4          |
| 400     | 91    | 5-6          |
| 5000000 | 96    | 5 mins       |
| 8000000 | 97    | 8 mins       |

- The randomized approach was able to give me a score of 97 which was the highest I could achieve.
- Although due to short of time I could not optimize the code in terms of runtime.


## Conclusion
It was fun completing this assignment. Since I was new to rust I learned a lot of things about rust which helped in learning the basics of rust.
It was the first time I ever wrote a program which is this close to systems programming. Working with bytes, VarInt, Little Endian, Big Endian was a fun experience.
Due to lack of time I couldn't contribute my 100% to this assignment and had to cut corners in some places. But I am happy with the result I got.
Overall it was a great learning experience and I am looking forward to more fun in the future.
Happy Coding!
