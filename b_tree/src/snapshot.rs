use std::fs::File;
use std::io::{self, Read, Write, BufWriter, BufReader};
use std::collections::HashMap;
use std::mem::size_of;
use crate::consts::{SnapshotHeader, SnapshotStock};
use crate::state::Portfolio; // Assuming Portfolio is in state.rs

// --- SAVING (Dump RAM to Disk) ---
pub fn save_snapshot(
    portfolios: &HashMap<u64, Portfolio>, 
    last_log_index: u64
) -> io::Result<()> {
    
    // We use a temporary file to avoid corruption if we crash mid-write
    let file = File::create("snapshot.bin.tmp")?;
    let mut writer = BufWriter::new(file);

    // 1. Write the "Last Log Index" first
    // This tells us: "This snapshot includes all history up to Log #X"
    writer.write_all(&last_log_index.to_le_bytes())?;

    // 2. Iterate through every user in RAM
    for (user_id, portfolio) in portfolios {
        // A. Create and Write Header
        let header = SnapshotHeader {
            user_id: *user_id,
            cash: portfolio.cash,
            num_stocks: portfolio.stocks.len() as u32,
            _padding: [0; 4],            
        };
        
        let header_bytes = bytemuck::bytes_of(&header);
        writer.write_all(header_bytes)?;

        // B. Write Stock List
        for (symbol_id, qty) in &portfolio.stocks {
            let stock = SnapshotStock {
                symbol_id: *symbol_id,
                _padding: [0; 4],
                quantity: *qty,
            };
            let stock_bytes = bytemuck::bytes_of(&stock);
            writer.write_all(stock_bytes)?;
        }
    }

    writer.flush()?;
    
    // 3. atomic swap (Rename tmp -> actual)
    // This ensures snapshot.bin is never half-written.
    std::fs::rename("snapshot.bin.tmp", "snapshot.bin")?;
    
    println!("Snapshot saved. Last Log Index: {}", last_log_index);
    Ok(())
}

// --- LOADING (Restore RAM from Disk) ---
pub fn load_snapshot() -> io::Result<(HashMap<u64, Portfolio>, u64)> {
    let file = match File::open("snapshot.bin") {
        Ok(f) => f,
        Err(_) => return Ok((HashMap::new(), 0)), // File doesn't exist yet
    };

    let mut reader = BufReader::new(file);
    let mut portfolios = HashMap::new();

    // 1. Read the "Last Log Index" (first 8 bytes)
    let mut idx_buf = [0u8; 8];
    reader.read_exact(&mut idx_buf)?;
    let last_log_index = u64::from_le_bytes(idx_buf);

    // 2. Loop until EOF
    loop {
        // Try to read a Header
        let mut header_buf = [0u8; size_of::<SnapshotHeader>()];
        match reader.read_exact(&mut header_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // Done
            Err(e) => return Err(e),
        }

        let header: SnapshotHeader = bytemuck::cast(header_buf);
        
        // Reconstruct Portfolio
        let mut stocks = HashMap::new();

        // Read the N stocks following this header
        for _ in 0..header.num_stocks {
            let mut stock_buf = [0u8; size_of::<SnapshotStock>()];
            reader.read_exact(&mut stock_buf)?;
            let stock: SnapshotStock = bytemuck::cast(stock_buf);
            stocks.insert(stock.symbol_id, stock.quantity);
        }

        portfolios.insert(header.user_id, Portfolio {
            cash: header.cash,
            stocks,
        });
    }

    println!("Snapshot loaded. Resuming from Log Index: {}", last_log_index);
    Ok((portfolios, last_log_index))
}