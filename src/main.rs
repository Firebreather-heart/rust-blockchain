'''use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let record = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.previous_hash, self.nonce
        );
        hasher.update(record.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new(difficulty: usize) -> Self {
        let mut genesis_block = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        let prefix = "0".repeat(difficulty);
        while &genesis_block.calculate_hash()[..difficulty] != prefix {
            genesis_block.nonce += 1;
        }
        genesis_block.hash = genesis_block.calculate_hash();

        Blockchain {
            chain: vec![genesis_block],
            difficulty,
        }
    }

    fn add_block(&mut self, data: String) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut new_block = Block::new(self.chain.len() as u64, data, previous_hash);

        let prefix = "0".repeat(self.difficulty);
        while &new_block.calculate_hash()[..self.difficulty] != prefix {
            new_block.nonce += 1;
        }
        new_block.hash = new_block.calculate_hash();
        println!("Mined block with nonce: {}", new_block.nonce);
        self.chain.push(new_block);
    }
}

struct AppState {
    blockchain: Mutex<Blockchain>,
}

#[derive(Deserialize)]
struct MineRequest {
    data: String,
}

async fn get_chain(data: web::Data<AppState>) -> impl Responder {
    let blockchain = data.blockchain.lock().unwrap();
    HttpResponse::Ok().json(&blockchain.chain)
}

async fn mine_block(req: web::Json<MineRequest>, data: web::Data<AppState>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    blockchain.add_block(req.data.clone());
    HttpResponse::Ok().json(&blockchain.chain)
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
            .route("/mine", web.post().to(mine_block))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}''
