use std::alloc::{alloc, Layout};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt; // Required for O_DIRECT
use std::ptr;
use std::slice;

struct BTree {
    pager: Pager,
    root_page_id: u32,
}

impl BTree {
    pub fn search(&mut self, key: u32) -> Option<u32> {
        let mut current_page_id = self.root_page_id;

        loop {
            // 1. Load the Page (Directly from RAM cache or SSD)
            let raw_page = self.pager.get_page(current_page_id);
            let node = Node::new(raw_page);

            // 2. Search inside the Page
            match node.binary_search(key) {
                Ok(index) => {
                    if node.is_leaf() {
                        // HIT: We found the key in a leaf. Return the Balance.
                        return Some(node.get_value_at_index(index));
                    } else {
                        // INTERNAL MATCH: In B-Trees, we keep digging right.
                        current_page_id = node.get_value_at_index(index + 1);
                    }
                }
                Err(index) => {
                    // MISS: Need to go deeper
                    if node.is_leaf() {
                        // Bottom of tree, key doesn't exist.
                        return None;
                    }

                    // Internal Node Logic: Pick the correct child
                    if index == node.get_num_cells() {
                        current_page_id = node.get_right_child();
                    } else {
                        current_page_id = node.get_value_at_index(index);
                    }
                }
            }
        }
    }
}
