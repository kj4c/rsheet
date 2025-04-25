use std::collections::HashMap;

use rsheet_lib::{
    cell_expr::CellArgument,
    cell_value::CellValue,
    cells::{column_name_to_number, column_number_to_name},
    command::CellIdentifier,
};

use crate::spreadsheet::CellContent;

pub fn cell_to_string(cell: CellIdentifier) -> String {
    let row = cell.row;
    let col = cell.col;
    let col_string = column_number_to_name(col);
    let row_string = (row + 1).to_string();
    format!("{}{}", col_string, row_string)
}

// handles the range provided returning all the variabels used within this range and the evaluated value
pub fn handle_range(
    range: String,
    spreadsheet: &HashMap<String, CellContent>,
) -> (Vec<String>, CellArgument) {
    // handle row and handle col
    let parts: Vec<&str> = range.split('_').collect();

    let start = parts[0];
    let ending = parts[1];

    let start_col = column_name_to_number(
        &start
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>(),
    );
    let end_col = column_name_to_number(
        &ending
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>(),
    );
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
    let mut variables: Vec<String> = Vec::new();

    // handle same row
    if start_row == end_row {
        let mut curr_col = start_col;
        let mut result: Vec<CellValue> = Vec::new();

        while curr_col <= end_col {
            let cell_key = format!("{}{}", column_number_to_name(curr_col), start_row);

            // get the val or none.
            let value = spreadsheet
                .get(&cell_key)
                .map(|cell| cell.value.clone())
                .unwrap_or(CellValue::None);

            variables.push(cell_key.clone());
            result.push(value);
            curr_col += 1;
        }

        return (variables, CellArgument::Vector(result));
    }

    // handle same col
    if start_col == end_col {
        let mut curr_row = start_row;
        let mut result: Vec<CellValue> = Vec::new();

        while curr_row <= end_row {
            let cell_key = format!("{}{}", column_number_to_name(start_col), curr_row);
            let value = spreadsheet
                .get(&cell_key)
                .map(|cell| cell.value.clone())
                .unwrap_or(CellValue::None);

            variables.push(cell_key);
            result.push(value);
            curr_row += 1;
        }

        return (variables, CellArgument::Vector(result));
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
                .unwrap_or(CellValue::None); // fallback if cell doesn't exist

            variables.push(cell_key);
            row_results.push(value);
            curr_row += 1;
        }

        matrix_results.push(row_results);
        curr_col += 1;
    }

    return (variables, CellArgument::Matrix(matrix_results));
}
