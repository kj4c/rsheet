use std::collections::HashMap;

use handle_cell::{cell_key, cell_to_string};
use rsheet_lib::{cell_value::CellValue, command::CellIdentifier, replies::Reply};

use crate::{handle_cell, spreadsheet::CellContent};

pub fn get_cell(
    cell_identifier: CellIdentifier,
    spreadsheet: &HashMap<String, CellContent>,
) -> Reply {
    let cell_name = cell_to_string(cell_identifier);

    match spreadsheet.get(&cell_name) {
        Some(content) => match &content.value {
            CellValue::Error(e) => {
                if content.formula.is_some() {
                    // since it came from the formula then it's bad
                    Reply::Error(e.clone())
                } else {
                    // if not put the cell name as well.
                    Reply::Value(cell_name, CellValue::Error(e.clone()))
                }
            }
            value => Reply::Value(cell_name, value.clone()),
        },
        None => Reply::Value(cell_name, CellValue::None),
    }
}
