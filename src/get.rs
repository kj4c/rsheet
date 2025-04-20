pub fn get(cell_identifier: CellIdentifier, spreadsheet: &mut HashMap<String, CellContent>) -> Reply {
    let cell_string = cell_to_string(cell_identifier);
    let cell_num = cell_key(cell_identifier);

    if let Some(content) = spreadsheet.get(&cell_string) {
        Reply::Value(cell_string, content.value.clone())
    } else {
        Reply::Value(cell_string, CellValue::None)
    }

}