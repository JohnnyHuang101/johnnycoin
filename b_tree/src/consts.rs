use bytemuck::{Pod, Zeroable};

// --- FIX 1: Do NOT derive Pod for the Enum ---
// Enums are not Pod safe by default. We store it as a u8 in the struct.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ActionType {
    None = 0,
    Deposit = 1,
    Withdraw = 2,
    Trade = 3,
}

// --- FIX 2: Ensure #[repr(C)] is present ---
#[repr(C)] 
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct UserMeta {
    pub user_id: u64,        
    pub username: [u8; 32], 
    pub email: [u8; 64],    
    pub pass_hash: [u8; 32],
    pub salt: [u8; 16],     
    pub created_at: u64,    
    pub flags: u32,         
    pub _padding: [u8; 4], // Pad to align to 8 bytes
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LogEntry {
    pub magic: u16,          
    pub version: u16,        
    pub _pad1: [u8; 4],      // Explicit padding to reach byte 8

    pub user_id: u64,        
    pub timestamp: u64,      
    
    // --- FIX 3: Change u128 to [u8; 16] ---
    // u128 requires 16-byte alignment, which caused implicit padding errors.
    // [u8; 16] fits anywhere and is safer for disk storage.
    pub request_id: [u8; 16],    
    
    pub action_type: u8,     
    pub _pad2: [u8; 3],      // Explicit padding to align next u32
    
    pub symbol_id: u32,      
    pub quantity: i64,       
    pub amount_money: i64,   
}


// ... (Keep existing UserMeta and LogEntry) ...

// 1. The Snapshot Header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SnapshotHeader {
    pub user_id: u64,
    pub cash: i64,
    pub num_stocks: u32, // How many stock records follow this header?
    pub _padding: [u8; 4], // Align to 8 bytes

}

// 2. The Snapshot Stock Record
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SnapshotStock {
    pub symbol_id: u32,
    pub _padding: [u8; 4], // Align to 8 bytes
    pub quantity: i64,
}

enum DbMessage {
    WriteLog(LogEntry),
    Shutdown,
}