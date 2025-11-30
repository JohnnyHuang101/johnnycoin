use std::collections::HashMap;
// use crate::consts::{UserMeta, LogEntry};
use crate::reader::DatabaseReader;
use crate::snapshot::load_snapshot; // We will define this next

// What a user owns (In RAM)
#[derive(Clone, Debug, Default)]
pub struct Portfolio {
    pub cash: i64,
    pub stocks: HashMap<u32, i64>, // SymbolID -> Quantity
}

// The Global State container
pub struct AppState {
    pub user_index: HashMap<String, u64>,   // Username -> Index in users.bin
    pub portfolios: HashMap<u64, Portfolio>, // index in users.bin -> Balance
    pub reader: DatabaseReader,             // Keeps the mmap open
}

impl AppState {
    pub fn new() -> Self {
        println!("--- STARTUP SEQUENCE ---");

        // STEP 1: Load Snapshot (Fast)
        let (mut portfolios, last_snapshot_index) = load_snapshot()
            .unwrap_or((HashMap::new(), 0));


        let reader = DatabaseReader::new().expect("Failed to open DB");
        
        // STEP 2: Replay History (The Delta)
        let logs = reader.get_logs();
        let total_logs = logs.len() as u64;
        println!("Snapshot loaded. Last Index: {}. The total amount of logs equal {}", last_snapshot_index, total_logs);

        if total_logs > last_snapshot_index {
            println!("Replaying logs from {} to {}...", last_snapshot_index, total_logs);
            
            // Skip the logs we already know about
            for entry in logs.iter().skip(last_snapshot_index as usize) {
                let portfolio = portfolios.entry(entry.user_id).or_insert(Portfolio::default());
                
                match entry.action_type {
                    1 => portfolio.cash += entry.amount_money, // Deposit
                    2 => portfolio.cash -= entry.amount_money, // Withdraw
                    3 => { // Trade
                        portfolio.cash -= entry.amount_money; 
                        *portfolio.stocks.entry(entry.symbol_id).or_insert(0) += entry.quantity;
                    }
                    _ => {}
                }
            }
        }

        // STEP 3: Build User Index (For Login)
        let mut user_index = HashMap::new();
        let users = reader.get_users();
        for (idx, user) in users.iter().enumerate() {
            let name = std::str::from_utf8(&user.username)
                .unwrap_or("")
                .trim_matches('\0')
                .to_string();
            if !name.is_empty() {
                user_index.insert(name, idx as u64);
            }
        }

        println!("Startup Complete. Users: {}, Portfolios: {}", user_index.len(), portfolios.len());

        Self { user_index, portfolios, reader }
    }
}