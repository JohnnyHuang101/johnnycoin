use tokio::sync::mpsc::Sender; // Import Sender
use std::collections::HashMap;
use crate::consts::{UserMeta, LogEntry};
use crate::reader::DatabaseReader;
use crate::snapshot::load_snapshot;

// 1. Define the Message Type (What can we send to the disk?)
#[derive(Debug)]
pub enum DbMessage {
    WriteLog(LogEntry),
    WriteUser(UserMeta),
}

// 2. Add Sender to AppState
#[derive(Clone, Debug, Default)]
pub struct Portfolio {
    pub cash: i64,
    pub stocks: HashMap<u32, i64>,
}

pub struct AppState {
    pub user_index: HashMap<String, u64>,
    pub portfolios: HashMap<u64, Portfolio>,
    pub reader: DatabaseReader,
    pub db_sender: Sender<DbMessage>, // <--- NEW FIELD
}

impl AppState {
    // 3. Update Constructor to accept the Sender
    pub fn new(db_sender: Sender<DbMessage>) -> Self {
        println!("--- STARTUP SEQUENCE ---");

        let (mut portfolios, last_snapshot_index) = load_snapshot()
            .unwrap_or((HashMap::new(), 0));

        let reader = DatabaseReader::new().expect("Failed to open DB");
        
        let logs = reader.get_logs();
        let total_logs = logs.len() as u64;

        if total_logs > last_snapshot_index {
            println!("Replaying logs from {} to {}...", last_snapshot_index, total_logs);
            for entry in logs.iter().skip(last_snapshot_index as usize) {
                let portfolio = portfolios.entry(entry.user_id).or_insert(Portfolio::default());
                match entry.action_type {
                    1 => portfolio.cash += entry.amount_money,
                    2 => portfolio.cash -= entry.amount_money,
                    3 => {
                        portfolio.cash -= entry.amount_money; 
                        *portfolio.stocks.entry(entry.symbol_id).or_insert(0) += entry.quantity;
                    }
                    _ => {}
                }
            }
        }

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

        println!("Startup Complete.");

        Self { user_index, portfolios, reader, db_sender }
    }
}