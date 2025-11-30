const PAGE_SIZE: usize = 4096;

// HEADER LAYOUT: [Type(1) | Root(1) | NumCells(2) | RightPtr(4)] = 8 Bytes
const HEADER_SIZE: usize = 8;
const NODE_TYPE_OFFSET: usize = 0;
const NUM_CELLS_OFFSET: usize = 2;
const NUM_CELLS_SIZE: usize = 2;
const RIGHT_CHILD_OFFSET: usize = 4;

// CELL LAYOUT: [Key(4) | Value(4)] = 8 Bytes
// Note: Value is 'Balance' for Leaves, or 'PageID' for Internal Nodes
const CELL_SIZE: usize = 8;
const KEY_OFFSET: usize = 0;
const VALUE_OFFSET: usize = 4;
