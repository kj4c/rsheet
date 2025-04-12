use std::cell::Cell;

use rsheet_lib::{cells::column_number_to_name, command::CellIdentifier};

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