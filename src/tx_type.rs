use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PubkeyScript {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    #[serde(default = "default_str")]
    pub scriptpubkey_address: String,
    pub value: u64,
}

#[derive(Deserialize, Debug)]
pub struct Vin {
    pub txid: String,
    pub vout: u64,
    pub prevout: PubkeyScript,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    #[serde(default = "empty_vec")]
    pub witness: Vec<String>,
    pub is_coinbase: bool,
    pub sequence: u64,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub version: u64,
    pub locktime: u64,
    pub vout: Vec<PubkeyScript>,
    pub vin: Vec<Vin>,
}

fn empty_vec() -> Vec<String> {
    vec![]
}

fn default_str() -> String {
    String::from("")
}
// Function to deserialize JSON string into Transaction struct
impl Transaction {
    pub fn new(raw_str: &String) -> Result<Transaction, serde_json::Error> {
        let tx: Transaction = serde_json::from_str(raw_str)?;
        Ok(tx)
    }
}
