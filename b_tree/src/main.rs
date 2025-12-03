mod consts;
mod writer;
mod reader;
mod state;
mod snapshot;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task;
use tokio::sync::mpsc; // Import Channel

use state::{AppState, DbMessage}; // Import DbMessage
use snapshot::save_snapshot;
use writer::{DatabaseWriter, make_string};
use consts::{UserMeta, LogEntry, ActionType};

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    println!("Initializing Engine...");

    // 1. SETUP CHANNEL (The Buffer)
    // Capacity 10,000 means we can hold 10k pending writes in RAM before slowing down.
    let (tx, mut rx) = mpsc::channel::<DbMessage>(10_000);

    // 2. SPAWN PERSISTER THREAD (Dedicated Disk Worker)
    // We use std::thread because file I/O is blocking.
    std::thread::spawn(move || {
        println!("[Persister] Disk Thread Started");
        let mut db = DatabaseWriter::new().unwrap();

        // Loop forever, waiting for messages
        while let Some(msg) = rx.blocking_recv() {
            match msg {
                DbMessage::WriteLog(entry) => {
                    if let Err(e) = db.append_log(&entry) {
                        eprintln!("[Persister] LOG WRITE FAILED: {}", e);
                    }
                }
                DbMessage::WriteUser(user) => {
                    if let Err(e) = db.append_user(&user) {
                        eprintln!("[Persister] USER WRITE FAILED: {}", e);
                    }
                }
            }
        }
    });
    
    // 3. START ENGINE (Pass 'tx' to AppState)
    let app_state = AppState::new(tx);
    let shared_state = Arc::new(RwLock::new(app_state));

    // 4. SPAWN BACKGROUND SNAPSHOTTER
    let bg_state = shared_state.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(600)).await;
            let app = bg_state.read().unwrap();
            let log_len = app.reader.get_live_log_length(); 
            println!("[Snapshot] Saving state...");
            let _ = save_snapshot(&app.portfolios, log_len);
        }
    });

    // 5. API ROUTES
    let app = Router::new()
        .route("/balance/:username", get(get_balance))
        .route("/trade", post(execute_trade))
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 High-Frequency Engine Ready at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLERS (Now Non-Blocking!) ---

#[derive(Deserialize)]
struct TradeRequest {
    username: String,
    symbol_id: u32,
    amount: i64, 
    is_cash: bool, 
}

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
    email: Option<String>,
}

async fn execute_trade(
    State(state): State<SharedState>,
    Json(payload): Json<TradeRequest>,
) -> Json<serde_json::Value> {
    // 1. Lock RAM (Fast)
    let mut app = state.write().unwrap();

    let user_id = match app.user_index.get(&payload.username) {
        Some(id) => *id,
        None => return Json(serde_json::json!({"error": "User not found"})),
    };

    // 2. Update RAM Logic (Instant)
    let db_sender = app.db_sender.clone();
    let portfolio = app.portfolios.entry(user_id).or_default();
    let cost = payload.amount * 100;

    // ... (Your existing validation logic) ...
    if !payload.is_cash {
        if payload.amount > 0 && portfolio.cash < cost {
             return Json(serde_json::json!({"status": "Insufficient Funds"}));
        }
        if payload.amount < 0 {
             let qty = portfolio.stocks.entry(payload.symbol_id).or_default();
             if *qty < payload.amount.abs() {
                 return Json(serde_json::json!({"status": "Insufficient Stock"}));
             }
        }
    }

    // Apply RAM changes
    if payload.is_cash {
        portfolio.cash += payload.amount;
    } else {
        portfolio.cash -= cost;
        *portfolio.stocks.entry(payload.symbol_id).or_default() += payload.amount;
    }

    // 3. Construct Log Entry
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let action = if payload.is_cash { 
        if payload.amount > 0 { ActionType::Deposit } else { ActionType::Withdraw }
    } else { ActionType::Trade };

    let entry = LogEntry {
        magic: 0xAABB,
        version: 1,
        _pad1: [0; 4],
        user_id,
        timestamp: now,
        request_id: [0; 16],
        action_type: action as u8,
        _pad2: [0; 3],
        symbol_id: payload.symbol_id,
        quantity: if payload.is_cash { 0 } else { payload.amount },
        amount_money: if payload.is_cash { payload.amount.abs() } else { 100 }, 
    };

    // 4. FIRE AND FORGET (Send to Persister)
    // This puts the message in the channel buffer. It returns instantly.
    // The Persister thread will handle the disk write whenever it can.
    
    let _ = db_sender.try_send(DbMessage::WriteLog(entry));

    Json(serde_json::json!({"status": "Trade Executed", "new_cash": portfolio.cash}))
}

async fn register_user(
    State(state): State<SharedState>,
    Json(payload): Json<AuthRequest>,
) -> Json<serde_json::Value> {
    let mut app = state.write().unwrap();

    if app.user_index.contains_key(&payload.username) {
        return Json(serde_json::json!({"error": "Username taken"}));
    }

    let new_id = app.user_index.len() as u64;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Create User Struct
    let new_user = UserMeta {
        user_id: new_id,
        username: make_string(&payload.username),
        email: make_string(payload.email.as_deref().unwrap_or("")),
        pass_hash: [0; 32], 
        salt: [0; 16],
        created_at: now,
        flags: 1,
        _padding: [0; 4],
    };

    // 1. Update RAM Immediately
    app.user_index.insert(payload.username.clone(), new_id);
    app.portfolios.insert(new_id, Default::default());

    // 2. Queue Disk Write
    let _ = app.db_sender.try_send(DbMessage::WriteUser(new_user));

    Json(serde_json::json!({"status": "User Registered", "user_id": new_id}))
}

// ... (get_balance and login_user remain the same as before) ...
async fn get_balance(
    State(state): State<SharedState>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    let app = state.read().unwrap();

    let user_id = match app.user_index.get(&username) {
        Some(id) => *id,
        None => return Json(serde_json::json!({"error": "User not found"})),
    };

    if let Some(p) = app.portfolios.get(&user_id) {
        Json(serde_json::json!({
            "user": username,
            "cash": p.cash,
            "stocks": p.stocks
        }))
    } else {
        Json(serde_json::json!({"error": "Portfolio not found"}))
    }
}

async fn login_user(
    State(state): State<SharedState>,
    Json(payload): Json<AuthRequest>,
) -> Json<serde_json::Value> {
    let app = state.read().unwrap();

    let user_idx = match app.user_index.get(&payload.username) {
        Some(idx) => *idx as usize,
        None => return Json(serde_json::json!({"error": "User not found"})),
    };

    let users = app.reader.get_users();
    
    if user_idx >= users.len() {
        return Json(serde_json::json!({"error": "Index mismatch"}));
    }
    
    let user = &users[user_idx];

    // Simple password check (In real life use Argon2)
    // Here we assume empty password for demo since we passed [0;32]
    Json(serde_json::json!({
        "status": "Login Success", 
        "user_id": user.user_id
    }))
}