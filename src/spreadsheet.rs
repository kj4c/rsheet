use rsheet_lib::{cell_expr::CellExpr, cell_value::CellValue};

pub struct CellContent {
    pub formula: Option<String>,
    pub value: CellValue
}