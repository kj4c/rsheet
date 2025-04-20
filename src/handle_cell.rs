use std::cell::Cell;

use rsheet_lib::{cell_expr::CellArgument, cells::{column_name_to_number, column_number_to_name}, command::CellIdentifier};

pub fn cell_to_string(cell: CellIdentifier) -> String {
    let row = cell.row;
    let col = cell.col;
    let col_string  = column_number_to_name(col);
    let row_string = (row + 1).to_string();
    format!("{}{}", col_string, row_string)
}

pub fn cell_key(cell: CellIdentifier) -> (u32, u32) {
    let row = cell.row;
    let col = cell.col;
    (row, col)
}

pub fn handle_range(range: String) -> CellArgument {
    // handle row and handle col
    let parts: Vec<&str> = range.split('_').collect();

    let start = parts[0];
    let ending = parts[1];

    let start_col = column_name_to_number(&start.chars().filter(|c| c.is_alphabetic()).collect::<String>());
    let end_col = column_name_to_number(&ending.chars().filter(|c| c.is_alphabetic()).collect::<String>());
    let start_row: String = ending.chars().filter(|c| c.is_alphabetic()).collect();
    let end_row: String = ending.chars().filter(|c| c.is_alphabetic()).collect();

    // handle same row
    if (start_row == end_row) {
        let result;
        for 
        

    // handle same col


    // if both diff then matrix

} 