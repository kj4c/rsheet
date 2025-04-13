mod handle_cell;
mod spreadsheet;

use handle_cell::{cell_key, cell_to_string};
use rsheet_lib::cell_expr::{self, CellExpr};
use spreadsheet::CellContent;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::cells::column_number_to_name;
use rsheet_lib::command::Command;
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
    let mut spreadsheet: HashMap<(u32, u32), CellContent> = HashMap::new();
    // dependency graph to see what affects what
    let mut dependency: HashMap<(u32, u32), HashSet<(u32, u32)>> = HashMap::new();

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

                            if let Some(content) = spreadsheet.get(&cell_num) {
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
