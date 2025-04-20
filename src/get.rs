use std::collections::HashMap;

use handle_cell::{cell_key, cell_to_string};
use rsheet_lib::{cell_value::CellValue, command::CellIdentifier, replies::Reply};

use crate::{handle_cell, spreadsheet::CellContent};

pub fn get_cell(cell_identifier: CellIdentifier, spreadsheet: &HashMap<String, CellContent>) -> Reply {
    let cell_string = cell_to_string(cell_identifier);
    let cell_num = cell_key(cell_identifier);

    if let Some(content) = spreadsheet.get(&cell_string) {
        Reply::Value(cell_string, content.value.clone())
    } else {
        Reply::Value(cell_string, CellValue::None)
    }

}