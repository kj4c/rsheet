use std::collections::{HashMap, HashSet};

use rsheet_lib::{cell_expr::{CellArgument, CellExpr}, cell_value::CellValue};

use crate::{handle_cell::handle_range, spreadsheet::CellContent};

pub fn set_cell(
    cell_string: String,
    formula: String,
    spreadsheet: &mut HashMap<String, CellContent>,
    depends_on: &mut HashMap<String, HashSet<String>>,
    depends_by: &mut HashMap<String, HashSet<String>>,
) {
    let expression = CellExpr::new(&formula);
    let vars = expression.find_variable_names();
    let has_formula = !vars.is_empty();

    // get rid of prev dependencies
    let old_deps = depends_on
        .get(&cell_string)
        .map(|s| s.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    for dep in old_deps {
        if let Some(set) = depends_by.get_mut(&dep) {
            set.remove(&cell_string);
        }
    }

    depends_on.entry(cell_string.clone()).or_default().clear();

    // evaluate the current formula
    let mut var_to_value = HashMap::new();

    for var in &vars {
        depends_on.entry(cell_string.clone()).or_default().insert(var.clone());
        depends_by.entry(var.clone()).or_default().insert(cell_string.clone());

        let value = spreadsheet.get(var).map(|c| c.value.clone()).unwrap_or(CellValue::None);

        if var.contains('_') {
            let vec_val = handle_range(var.clone(), spreadsheet);
            var_to_value.insert(var.clone(), vec_val);
        } else {
            var_to_value.insert(var.clone(), CellArgument::Value(value));
        }
    }

    // println!("{:?}", var_to_value);

    let evaluated = expression.evaluate(&var_to_value);
    let value = match evaluated {
        Ok(v) => v,
        Err(_) => match formula.parse::<i64>() {
            Ok(n) => CellValue::Int(n),
            Err(_) => CellValue::String(formula.trim_matches('"').to_string()),
        },
    };

    spreadsheet.insert(
        cell_string.clone(),
        CellContent {
            formula: if has_formula { Some(formula.clone()) } else { None },
            value,
        },
    );

    // update all dependents by running the set function again
    if let Some(dependents) = depends_by.get(&cell_string) {
        for dep in dependents.clone() {
            if let Some(content) = spreadsheet.get(&dep) {
                if let Some(dep_formula) = &content.formula {
                    set_cell(dep, dep_formula.clone(), spreadsheet, depends_on, depends_by);
                }
            }
        }
    }
    
}
