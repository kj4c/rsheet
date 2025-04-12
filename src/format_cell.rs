pub fn format_cell_key(row: u32, col: u32) -> String {
    let col_letter = ((col as u8) + b'A') as char;
    format!("{col_letter}{}", row + 1)
}