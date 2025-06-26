'''use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let transactions_str = serde_json::to_string(&self.transactions).unwrap();
        let record = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, transactions_str, self.previous_hash, self.nonce
        );
        hasher.update(record.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: usize,
}

impl Blockchain {
    fn new(difficulty: usize) -> Self {
        let mut genesis_block = Block::new(0, vec![], "0".to_string());
        let prefix = "0".repeat(difficulty);
        while &genesis_block.calculate_hash()[..difficulty] != prefix {
            genesis_block.nonce += 1;
        }
        genesis_block.hash = genesis_block.calculate_hash();

        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: vec![],
            difficulty,
        }
    }

    fn new_transaction(&mut self, sender: String, recipient: String, amount: f64) {
        self.pending_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });
    }

    fn mine_block(&mut self) -> &Block {
        self.new_transaction("0".to_string(), "my-wallet-address".to_string(), 1.0); // Miner's reward

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_hash,
        );

        let prefix = "0".repeat(self.difficulty);
        while &new_block.calculate_hash()[..self.difficulty] != prefix {
            new_block.nonce += 1;
        }
        new_block.hash = new_block.calculate_hash();
        println!("Mined block with nonce: {}", new_block.nonce);

        self.chain.push(new_block);
        self.pending_transactions = vec![];
        self.chain.last().unwrap()
    }
}

struct AppState {
    blockchain: Mutex<Blockchain>,
}

async fn get_chain(data: web::Data<AppState>) -> impl Responder {
    let blockchain = data.blockchain.lock().unwrap();
    HttpResponse::Ok().json(&blockchain.chain)
}

async fn new_transaction(req: web::Json<Transaction>, data: web::Data<AppState>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    blockchain.new_transaction(req.sender.clone(), req.recipient.clone(), req.amount);
    HttpResponse::Ok().body("Transaction added to pending transactions")
}

async fn mine_block(data: web::Data<AppState>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    let new_block = blockchain.mine_block();
    HttpResponse::Ok().json(new_block)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let difficulty = 2;
    let blockchain = web::Data::new(AppState {
        blockchain: Mutex::new(Blockchain::new(difficulty)),
    });

    println!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(blockchain.clone())
            .route("/chain", web::get().to(get_chain))
            .route("/transactions/new", web::post().to(new_transaction))
            .route("/mine", web::get().to(mine_block)) // Changed to GET for simplicity
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}''
