
pub struct Pager {
    file: File,
    pages: HashMap<u32, Box<[u8]>>,
}

impl Pager {
    pub fn new(filename: &str) -> Self {
        // OPEN WITH O_DIRECT (Bypass OS Cache)
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(libc::O_DIRECT)
            .open(filename)
            .expect("Unable to open DB file with O_DIRECT");

        if file.metadata().unwrap().len() == 0 {
            // Allocate a temporary aligned buffer of zeros
            let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
            let ptr = unsafe { alloc(layout) };
            let mut zeros = unsafe { slice::from_raw_parts_mut(ptr, PAGE_SIZE) };
            
            // Initialize byte 0 to 1 (IsLeaf = True) so it's a valid empty root
            zeros[0] = 1; 

            // Write it to disk
            file.write_all(zeros).unwrap();
            
            // Note: We leak 'ptr' here for brevity, usually you'd dealloc
        }

        Pager {
            file,
            pages: HashMap::new(),
        }
    }

    pub fn get_page(&mut self, page_id: u32) -> &mut [u8] {
        if self.pages.contains_key(&page_id) {
            return self.pages.get_mut(&page_id).unwrap();
        }

        // MEMORY ALLOCATION (Aligned to 4096)
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        let mut buffer: Box<[u8]> = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate aligned memory");
            }
            let slice = slice::from_raw_parts_mut(ptr, PAGE_SIZE);
            Box::from_raw(slice)
        };

        // READ FROM DISK
        let offset = page_id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.read_exact(&mut buffer).unwrap();

        self.pages.insert(page_id, buffer);
        self.pages.get_mut(&page_id).unwrap()
    }
}
