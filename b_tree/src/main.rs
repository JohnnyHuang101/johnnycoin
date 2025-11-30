mod consts;
mod writer;
mod reader;
mod state;    // NEW
mod snapshot; // NEW

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use consts::{UserMeta, LogEntry, ActionType};
use writer::{DatabaseWriter, make_string};
use state::AppState;
use snapshot::save_snapshot;

fn main() -> std::io::Result<()> {
    // 1. INITIALIZE THE BRAIN (RAM State)
    let state = Arc::new(RwLock::new(AppState::new()));

    // 2. BACKGROUND SNAPSHOT THREAD
    let state_bg = state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10)); // Save every 10 seconds
            println!("\n[Background] Creating Snapshot...");
            
            let app = state_bg.read().unwrap();
            let log_len = app.reader.get_live_log_length() as u64;
            
            // Dump RAM to Disk
            println!("[Background] Saving log length. next startup log index should be {}", log_len);

            if let Err(e) = save_snapshot(&app.portfolios, log_len) {
                eprintln!("Snapshot failed: {}", e);
            } else {
                println!("[Background] Snapshot Saved!");
            }
        }
    });

    // 3. SIMULATE A NEW USER & TRADE
    {
        println!("\n--- SIMULATION STARTING ---");
        
        // A. Open the Writer (Append Only)
        let mut db = DatabaseWriter::new()?;

        // B. Create a User
        let new_user = UserMeta {
            user_id: 1,
            username: make_string("crypto_king"),
            email: make_string("king@btc.com"),
            pass_hash: [0; 32],
            salt: [0; 16],
            created_at: 12345,
            flags: 1,
            _padding: [0; 4],
        };
        db.append_user(&new_user)?;

        // C. Create a Log Entry (Deposit $50,000)
        let deposit = LogEntry {
            magic: 0xAABB,
            version: 1,
            _pad1: [0; 4],
            user_id: 1,
            timestamp: 123456,
            request_id: [0; 16],
            action_type: ActionType::Deposit as u8,
            _pad2: [0; 3],
            symbol_id: 0,
            quantity: 0,
            amount_money: 50_000,
        };
        
        // --- THE KEY MOMENT ---
        // 1. Write to Disk (Durability)
        db.append_log(&deposit)?; 

        // 2. Update RAM (Speed)
        // 2. Update RAM (Speed)
        {
            let mut app = state.write().unwrap();
            
            app.user_index.insert("crypto_king".to_string(), 1);

            let id = *app.user_index.get("crypto_king").unwrap_or(&0);

            let portfolio = app.portfolios.entry(1).or_insert_with(Default::default);
            portfolio.cash += 50_000;

            println!("Processed Deposit. User {} Balance: ${}", id, portfolio.cash);
        }
    }

    // Keep main thread alive to let the snapshotter run a bit
    thread::sleep(Duration::from_secs(12)); 
    Ok(())
}