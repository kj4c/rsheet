mod handle_cell;
mod spreadsheet;
pub mod set;
pub mod get;

use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::thread;
use handle_cell::{cell_to_string};
use set::set_cell;
use spreadsheet::CellContent;
use rsheet_lib::command::{Command};
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
    // make everything mutex
    // start the spreadsheet instance. 
    let mut spreadsheet: HashMap<String, CellContent> = HashMap::new();
    // dependency graph to see what affects what
    // so key = A1, set = A2 => A1 has an equation with A2
    let mut depends_on: HashMap<String, HashSet<String>> = HashMap::new();
    // so key = A1, set = A2 => A2 has an equation with A1
    let mut depends_by: HashMap<String, HashSet<String>> = HashMap::new();
    
    type SpreadsheetState = Arc<Mutex<(
        HashMap<String, CellContent>,
        HashMap<String, HashSet<String>>,
        HashMap<String, HashSet<String>>,
    )>>;

    let state: SpreadsheetState = Arc::new(Mutex::new((
        HashMap::new(),
        HashMap::new(),
        HashMap::new()
    )));

    let mut sender_map: HashMap<String, Sender<Command>> = HashMap::new();

    // This initiates a single client connection, and reads and writes messages
    // indefinitely.
    // let (mut recv, mut send) = match manager.accept_new_connection() {
    //     Connection::NewConnection { reader, writer } => (reader, writer),
    //     Connection::NoMoreConnections => {
    //         // There are no more new connections to accept.
    //         return Ok(());
    //     }
    // };

    let mut threads = vec![];

    loop {
        match manager.accept_new_connection() {
            Connection::NewConnection { mut reader, mut writer } => {
                let state_clone = Arc::clone(&state);
                
                let thread = thread::spawn(move || {
                    loop {
                        match reader.read_message() {
                            ReadMessageResult::Message(msg) => {
                                let reply = match msg.parse::<Command>() {
                                    Ok(command) => match command {
                                        Command::Get { cell_identifier } => {
                                            let lock = state_clone.lock().unwrap();
                                            let spreadsheet = &lock.0;
                                            get::get_cell(cell_identifier, spreadsheet)
                                        },
                                        Command::Set { cell_identifier, cell_expr } => {
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
                                        },
                                    },
                                    Err(e) => Reply::Error(e),
                                };
    
                                if let WriteMessageResult::ConnectionClosed = writer.write_message(reply) {
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
                    }
                });

                threads.push(handle);
            }
            Connection::NoMoreConnections => {
                break;
            }
        }
    }
    for thread in threads {
        let _ = thread.join();
    }

    Ok(())
}
