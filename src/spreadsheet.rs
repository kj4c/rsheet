use rsheet_lib::{cell_expr::CellExpr, cell_value::CellValue};

#[derive(Clone)]
pub struct CellContent {
    pub formula: Option<String>,
    pub value: CellValue,
}
