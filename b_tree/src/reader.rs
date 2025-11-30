use std::fs::File;
use std::io;
use std::mem::size_of;
use memmap2::MmapOptions;
use crate::consts::{UserMeta, LogEntry};

pub struct DatabaseReader {
    user_mmap: memmap2::Mmap,
    log_mmap: memmap2::Mmap,
}

impl DatabaseReader {
    pub fn new() -> io::Result<Self> {
        let u_file = File::open("users.bin")?;
        let l_file = File::open("history.bin")?;

        // Map the files into Virtual Memory
        // This does NOT read the disk yet. It just reserves the addresses.
        let user_mmap = unsafe { MmapOptions::new().map(&u_file)? };
        let log_mmap = unsafe { MmapOptions::new().map(&l_file)? };

        Ok(Self { user_mmap, log_mmap })
    }

    // CAST RAW BYTES TO STRUCT SLICE
    pub fn get_users(&self) -> &[UserMeta] {
        let count = self.user_mmap.len() / size_of::<UserMeta>();
        // Zero-Copy Cast: The struct lies directly on the OS file buffer
        bytemuck::cast_slice(&self.user_mmap[0..count * size_of::<UserMeta>()])
    }

    pub fn get_logs(&self) -> &[LogEntry] {//maps memory in RAM for access
        let count = self.log_mmap.len() / size_of::<LogEntry>();
        bytemuck::cast_slice(&self.log_mmap[0..count * size_of::<LogEntry>()])
    }

    pub fn get_live_log_length(&self) -> u64 {
        // We open the file fresh just to check metadata
        let file = File::open("history.bin").unwrap();
        let len = file.metadata().unwrap().len();
        
        // Calculate number of entries
        len / size_of::<LogEntry>() as u64
    }
}