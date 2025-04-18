mod handle_cell;
mod spreadsheet;
pub mod set;

use handle_cell::{cell_key, cell_to_string};
use rsheet_lib::cell_expr::{self, CellArgument, CellExpr};
use spreadsheet::CellContent;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::cells::column_number_to_name;
use rsheet_lib::command::{CellIdentifier, Command};
use regex::Regex;
use rsheet_lib::connect::{
    Connection, Manager, ReadMessageResult, Reader, WriteMessageResult, Writer,
};
use rsheet_lib::replies::Reply;

use std::cell::{self, Cell};
use std::collections::{HashMap, HashSet};
use std::error::Error;

use log::info;

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    // start the spreadsheet instance. 
    let mut spreadsheet: HashMap<String, CellContent> = HashMap::new();
    // dependency graph to see what affects what
    // so key = A1, set = A2 => A1 has an equation with A2
    let mut depends_on: HashMap<String, HashSet<String>> = HashMap::new();
    // so key = A1, set = A2 => A2 depends on A1
    let mut depends_by: HashMap<String, HashSet<String>> = HashMap::new();

    // This initiates a single client connection, and reads and writes messages
    // indefinitely.
    let (mut recv, mut send) = match manager.accept_new_connection() {
        Connection::NewConnection { reader, writer } => (reader, writer),
        Connection::NoMoreConnections => {
            // There are no more new connections to accept.
            return Ok(());
        }
    };
    loop {
        info!("Just got message");
        match recv.read_message() {
            ReadMessageResult::Message(msg) => {
                // rsheet_lib already contains a FromStr<Command> (i.e. parse::<Command>)
                // implementation for parsing the get and set commands. This is just a
                // demonstration of how to use msg.parse::<Command>, you may want/have to
                // change this code.
                let reply = match msg.parse::<Command>() {
                    Ok(command) => match command {
                        Command::Get { cell_identifier } => {
                            // number = row, letter = collumn.
                            
                            // TODO: handle invalid cells.
                            let cell_string = cell_to_string(cell_identifier);
                            let cell_num = cell_key(cell_identifier);

                            if let Some(content) = spreadsheet.get(&cell_string) {
                                Reply::Value(cell_string, content.value.clone())
                            } else {
                                Reply::Value(cell_string, CellValue::None)
                            }
                        },
                        Command::Set {
                            cell_identifier,
                            cell_expr,
                        } => {
                            // first find all the variables related to this function
                            let expression = CellExpr::new(&cell_expr);
                            let vars = expression.find_variable_names();

                            // curr cell key
                            let cell_string = cell_to_string(cell_identifier);

                            // check that the value is sleep_then
                            let regex_sleep = Regex::new(r#"^sleep_then\((\d+),(.+)\)$"#).unwrap();
                            let cell_expr = if let Some(caps) = regex_sleep.captures(&cell_expr) {
                                caps[2].trim().to_string()
                            } else {
                                cell_expr.clone()
                            };

                            // need to update all dependencies.
                            // clear depends_on for that cell
                            let old_deps: Vec<String> = depends_on
                            .get(&cell_string)
                            .map(|set| set.iter().cloned().collect())
                            .unwrap_or_default();

                            // clear everything it previously depended on
                            for dep in old_deps {
                                if let Some(set) = depends_by.get_mut(&dep) {
                                    set.remove(&cell_string);
                                }
                            }

                            // clear this too
                            depends_on.entry(cell_string.clone()).or_default().clear();
                            
                            // get all the values
                            let mut var_to_value: HashMap<String, CellArgument> = HashMap::new();
                            let mut has_formula = false;
                            if vars.len() != 0 {
                                has_formula = true;
                            }

                            // this adds dependencies for each variable cell used
                            for var in vars {
                                // says that this cell depends on these variables
                                depends_on.entry(cell_string.clone()).or_default().insert(var.clone());
                                // otherway around
                                depends_by.entry(var.clone()).or_default().insert(cell_string.clone());

                                let final_val = if let Some(content) = spreadsheet.get(&var) {
                                    content.value.clone()
                                } else {
                                    CellValue::None
                                };

                                // get that value make a hashmap
                                var_to_value.insert(var, CellArgument::Value(final_val));
                            }

                            // go through everything that depended on this cell
                            for dep in depends_by
                            .get(&cell_string)
                            .map(|set| set.iter().cloned().collect()::<Vec<_>>())
                            .unwrap_or_default() {
                                // run the setting for each dependency

                            }

                            let evaluated = CellExpr::new(&cell_expr).evaluate(&var_to_value);

                            // if it fails means that you're missing something
                            let value = match evaluated {
                                Ok(val) => val,
                                Err(_) => {
                                    match cell_expr.parse::<i64>() {
                                        Ok(n) => CellValue::Int(n),
                                        Err(_) => CellValue::String(cell_expr.trim_matches('"').to_string()),
                                    }
                                }
                            };

                            // set the thing in the hashmap and that.
                            spreadsheet.insert(cell_string.clone(), CellContent {
                                formula: {
                                    if has_formula {
                                        Some(expression)
                                    } else {
                                        None
                                    }
                                },
                                value,
                            });

                            // skip the reply
                            continue;
                        },
                    },
                    Err(e) => Reply::Error(e),
                };

                match send.write_message(reply) {
                    WriteMessageResult::Ok => {
                        // Message successfully sent, continue.
                    }
                    WriteMessageResult::ConnectionClosed => {
                        // The connection was closed. This is not an error, but
                        // should terminate this connection.
                        break;
                    }
                    WriteMessageResult::Err(e) => {
                        // An unexpected error was encountered.
                        return Err(Box::new(e));
                    }
                }
            }
            ReadMessageResult::ConnectionClosed => {
                // The connection was closed. This is not an error, but
                // should terminate this connection.
                break;
            }
            ReadMessageResult::Err(e) => {
                // An unexpected error was encountered.
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}