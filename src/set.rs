use std::collections::{HashMap, HashSet};

use rsheet_lib::{
    cell_expr::{CellArgument, CellExpr},
    cell_value::CellValue,
};

use crate::{handle_cell::handle_range, spreadsheet::CellContent};

/// Everything needed to update a cell after evaluation
pub struct PreparedSet {
    pub cell_string: String,
    pub content: CellContent,
    pub new_depends_on: HashSet<String>,
}

/// Step 1: Run this outside the lock to do heavy work (can sleep)
pub fn prepare_set(
    cell_string: String,
    formula: String,
    spreadsheet_snapshot: &HashMap<String, CellContent>,
) -> PreparedSet {
    let expression = CellExpr::new(&formula);
    let vars = expression.find_variable_names();
    let has_formula = !vars.is_empty();

    let mut var_to_value = HashMap::new();
    let mut new_depends_on = HashSet::new();

    for var in &vars {
        new_depends_on.insert(var.clone());

        let value = spreadsheet_snapshot
            .get(var)
            .map(|c| c.value.clone())
            .unwrap_or(CellValue::None);

        if var.contains('_') {
            let (variables_used, vec_vals) = handle_range(var.clone(), spreadsheet_snapshot);

            for var_in_range in variables_used {
                new_depends_on.insert(var_in_range);
            }

            var_to_value.insert(var.clone(), vec_vals);
        } else {
            var_to_value.insert(var.clone(), CellArgument::Value(value));
        }
    }

    let evaluated = expression.evaluate(&var_to_value);
    let value = match evaluated {
        Ok(v) => v,
        Err(_) => match formula.parse::<i64>() {
            Ok(v) => CellValue::Int(v),
            Err(e) => CellValue::Error(e.to_string()),
        },
    };

    let content = CellContent {
        formula: if has_formula { Some(formula) } else { None },
        value,
    };

    PreparedSet {
        cell_string,
        content,
        new_depends_on,
    }
}

/// Step 2: Apply the result to the actual spreadsheet (under lock)
pub fn apply_set(
    prepared: PreparedSet,
    spreadsheet: &mut HashMap<String, CellContent>,
    depends_on: &mut HashMap<String, HashSet<String>>,
    depends_by: &mut HashMap<String, HashSet<String>>,
) {
    let cell_string = &prepared.cell_string;

    // Remove old dependencies
    if let Some(old) = depends_on.get(cell_string) {
        for var in old {
            if let Some(dependents) = depends_by.get_mut(var) {
                dependents.remove(cell_string);
            }
        }
    }

    // Add new dependencies
    depends_on
        .insert(cell_string.clone(), prepared.new_depends_on.clone());

    for var in &prepared.new_depends_on {
        depends_by.entry(var.clone()).or_default().insert(cell_string.clone());
    }

    // Update the cell
    spreadsheet.insert(cell_string.clone(), prepared.content.clone());

    // Recalculate any dependent cells recursively
    if let Some(dependents) = depends_by.get(cell_string) {
        for dep in dependents.clone() {
            if let Some(dep_content) = spreadsheet.get(&dep) {
                if let Some(dep_formula) = &dep_content.formula {
                    let snapshot = spreadsheet.clone();
                    let prepared_dep = prepare_set(dep.clone(), dep_formula.clone(), &snapshot);
                    apply_set(prepared_dep, spreadsheet, depends_on, depends_by);
                }
            }
        }
    }
}
