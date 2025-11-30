use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use std::mem::size_of;
// use std::slice;
use crate::consts::{UserMeta, LogEntry};

pub struct DatabaseWriter {
    user_file: File,
    log_file: File,
}

impl DatabaseWriter {
    pub fn new() -> io::Result<Self> {
        // Open with options that allow Append
        let user_file = OpenOptions::new()
            .read(true).write(true).create(true).append(true)
            .open("users.bin")?;

        let log_file = OpenOptions::new()
            .read(true).write(true).create(true).append(true)
            .open("history.bin")?;

        Ok(Self { user_file, log_file })
    }

    /// Writes a User struct directly to disk byte-wise
    pub fn append_user(&mut self, user: &UserMeta) -> io::Result<u64> {
        // 1. Convert Struct to Byte Slice (Unsafe but Fast)
        let bytes: &[u8] = bytemuck::bytes_of(user);

        // 2. Write to OS Buffer
        self.user_file.write_all(bytes)?;

        // 3. FORCE DISK WRITE (Bypass Cache logic)
        // This forces the OS to flush buffers to the physical platter/NAND immediately.
        self.user_file.sync_all()?; 

        // Return the index (ID) of this user
        let len = self.user_file.metadata()?.len();
        Ok((len / size_of::<UserMeta>() as u64) - 1)
    }

    /// Writes a Log entry directly to disk
    pub fn append_log(&mut self, entry: &LogEntry) -> io::Result<()> {
        let bytes: &[u8] = bytemuck::bytes_of(entry);
        self.log_file.write_all(bytes)?;
        
        // Critical for "Durability" - ensure it hits the disk
        self.log_file.sync_data()?; 
        Ok(())
    }
}

// Helper to create the Fixed-Size Byte Arrays
pub fn make_string<const N: usize>(s: &str) -> [u8; N] {
    let mut buf = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N); // Truncate if too long
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}
