use std::{cell::Cell, collections::HashMap};

use rsheet_lib::{cell_expr::{self, CellArgument}, cell_value::CellValue, cells::{column_name_to_number, column_number_to_name}, command::CellIdentifier};

use crate::spreadsheet::{self, CellContent};

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

pub fn handle_range(range: String, spreadsheet: &HashMap<String, CellContent>) -> CellArgument {
    // handle row and handle col
    let parts: Vec<&str> = range.split('_').collect();

    let start = parts[0];
    let ending = parts[1];

    let start_col = column_name_to_number(&start.chars().filter(|c| c.is_alphabetic()).collect::<String>());
    let end_col = column_name_to_number(&ending.chars().filter(|c| c.is_alphabetic()).collect::<String>());
    let start_row: u32 = start
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect::<String>()
    .parse()
    .expect("Invalid number in start");

    let end_row: u32 = ending
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect::<String>()
    .parse()
    .expect("Invalid number in end");
    let mut result: Vec<CellValue> = Vec::new();

    // handle same row
    if start_row == end_row {
        let mut curr_col = start_col;
        let mut result: Vec<CellValue> = Vec::new();
        
        while curr_col <= end_col {
            let cell_key = format!("{}{}", column_number_to_name(curr_col), start_row);
            let value = spreadsheet
            .get(&cell_key)
            .map(|cell| cell.value.clone()) // extract and clone the CellValue
            .unwrap_or(CellValue::None);    // fallback if cell doesn't exist

            result.push(value);
            curr_col += 1;
        }

        return rsheet_lib::cell_expr::CellArgument::Vector(result);
    }

    // handle same col
    if start_col == end_col {
        let mut curr_row = start_row;
        let mut result: Vec<CellValue> = Vec::new();
        
        while curr_row <= end_row {
            let cell_key = format!("{}{}", column_number_to_name(start_col), curr_row);
            let value = spreadsheet
            .get(&cell_key)
            .map(|cell| cell.value.clone()) // extract and clone the CellValue
            .unwrap_or(CellValue::None);    // fallback if cell doesn't exist

            result.push(value);
            curr_row += 1;
        }

        return rsheet_lib::cell_expr::CellArgument::Vector(result);
    }

    // handle matrices
    // row first then column
    let mut curr_col = start_col;
    let mut curr_row = start_row;

    let mut matrix_results: Vec<Vec<CellValue>> = Vec::new();
    while curr_col <= end_col {
        let mut row_results: Vec<CellValue> = Vec::new();
        curr_row = start_row;
        while curr_row <= end_row {
            let cell_key = format!("{}{}", column_number_to_name(curr_col), curr_row);
            let value = spreadsheet
            .get(&cell_key)
            .map(|cell| cell.value.clone()) // extract and clone the CellValue
            .unwrap_or(CellValue::None);    // fallback if cell doesn't exist

            row_results.push(value);
            curr_row += 1;
        }

        matrix_results.push(row_results);
        curr_col += 1;
    }

    return CellArgument::Matrix(matrix_results);

} 