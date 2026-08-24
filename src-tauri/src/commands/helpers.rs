pub fn i64v(row: &rusqlite::Row, i: usize) -> i64 { row.get::<_,i64>(i).unwrap_or(0) }
pub fn u64v(row: &rusqlite::Row, i: usize) -> u64 { row.get::<_,i64>(i).unwrap_or(0) as u64 }
pub fn u32v(row: &rusqlite::Row, i: usize) -> u32 { row.get::<_,i64>(i).unwrap_or(0) as u32 }
pub fn u8v(row: &rusqlite::Row, i: usize) -> u8 { row.get::<_,i64>(i).unwrap_or(0) as u8 }
pub fn boolv(row: &rusqlite::Row, i: usize) -> bool { row.get::<_,i64>(i).unwrap_or(0) != 0 }
