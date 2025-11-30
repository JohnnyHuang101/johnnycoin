
pub struct Node<'a> {
    bytes: &'a mut [u8],
}

impl<'a> Node<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes }
    }

    pub fn get_num_cells(&self) -> u16 {
        let start = NUM_CELLS_OFFSET;
        let end = NUM_CELLS_OFFSET + NUM_CELLS_SIZE;
        u16::from_le_bytes(self.bytes[start..end].try_into().unwrap())
    }

    pub fn is_leaf(&self) -> bool {
        self.bytes[NODE_TYPE_OFFSET] == 1
    }

    pub fn get_key_at_index(&self, index: u16) -> u32 {
        let offset = HEADER_SIZE + (index as usize * CELL_SIZE) + KEY_OFFSET;
        let bytes = &self.bytes[offset..offset + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    // Returns Balance (if Leaf) or PageID (if Internal)
    pub fn get_value_at_index(&self, index: u16) -> u32 {
        let offset = HEADER_SIZE + (index as usize * CELL_SIZE) + VALUE_OFFSET;
        let bytes = &self.bytes[offset..offset + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn get_right_child(&self) -> u32 {
        let start = RIGHT_CHILD_OFFSET;
        let bytes = &self.bytes[start..start + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn binary_search(&self, target: u32) -> Result<u16, u16> {
        let mut low: u16 = 0;
        let mut high: u16 = self.get_num_cells();

        while low < high {
            let mid = low + (high - low) / 2;
            let key_at_mid = self.get_key_at_index(mid);

            match key_at_mid.cmp(&target) {
                Ordering::Equal => return Ok(mid),
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
            }
        }
        Err(low)
    }
}