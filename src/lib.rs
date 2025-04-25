pub mod get;
mod handle_cell;
pub mod set;
mod spreadsheet;

use handle_cell::cell_to_string;
use rsheet_lib::command::Command;
use rsheet_lib::connect::{
    Connection, Manager, ReadMessageResult, Reader, WriteMessageResult, Writer,
};
use rsheet_lib::replies::Reply;
use set::set_cell;
use spreadsheet::CellContent;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use std::cell::{self, Cell};
use std::collections::{HashMap, HashSet};
use std::error::Error;

use log::info;
pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    // make everything mutex
    // start the spreadsheet instance.
    let mut spreadsheet: HashMap<String, CellContent> = HashMap::new();
    // dependency graph to see what affects what
    // so key = A1, set = A2 => A1 has an equation with A2
    let mut depends_on: HashMap<String, HashSet<String>> = HashMap::new();
    // so key = A1, set = A2 => A2 has an equation with A1
    let mut depends_by: HashMap<String, HashSet<String>> = HashMap::new();

    type SpreadsheetState = Arc<
        Mutex<(
            HashMap<String, CellContent>,
            HashMap<String, HashSet<String>>,
            HashMap<String, HashSet<String>>,
        )>,
    >;

    let state: SpreadsheetState =
        Arc::new(Mutex::new((HashMap::new(), HashMap::new(), HashMap::new())));

    let mut sender_map: HashMap<String, Sender<Command>> = HashMap::new();

    thread::scope(|s| loop {
        match manager.accept_new_connection() {
            Connection::NewConnection {
                mut reader,
                mut writer,
            } => {
                let state_clone = Arc::clone(&state);
                


                // create a thread and return a handle
                s.spawn(move || loop {
                    match reader.read_message() {
                        ReadMessageResult::Message(msg) => {
                            let reply = match msg.parse::<Command>() {
                                Ok(command) => match command {
                                    Command::Get { cell_identifier } => {
                                        let lock = state_clone.lock().unwrap();
                                        let spreadsheet = &lock.0;
                                        get::get_cell(cell_identifier, spreadsheet)
                                    }
                                    Command::Set {
                                        cell_identifier,
                                        cell_expr,
                                    } => {
                                        let mut lock = state_clone.lock().unwrap();
                                        let (spreadsheet, depends_on, depends_by) = &mut *lock;
                                        set_cell(
                                            cell_to_string(cell_identifier),
                                            cell_expr,
                                            spreadsheet,
                                            depends_on,
                                            depends_by,
                                        );
                                        continue;
                                    }
                                },
                                Err(e) => Reply::Error(e),
                            };

                            if let WriteMessageResult::ConnectionClosed =
                                writer.write_message(reply)
                            {
                                break;
                            }
                        }
                        ReadMessageResult::ConnectionClosed => {
                            break;
                        }
                        ReadMessageResult::Err(e) => {
                            eprintln!("Error reading message: {}", e);
                            break;
                        }
                    }
                });
            }
            Connection::NoMoreConnections => {
                break;
            }
        }
    });

    Ok(())
}
