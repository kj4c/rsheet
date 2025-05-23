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
use spreadsheet::CellContent;
use std::sync::{Arc, Mutex};
use std::thread;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Instant;
use std::sync::RwLock;


fn get_current_timestamp() -> u64 {
    Instant::now().elapsed().as_millis() as u64
}

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    type SpreadsheetState = Arc<
        Mutex<(
            // spreadsheet instance with CelLContent struct which stores formula and value
            HashMap<String, CellContent>,
            // dependson set so key = A1, set = A2 => A1 has an equation with A2
            HashMap<String, HashSet<String>>,
            // depends_by set so key = A1, set = A2 => A2 has an equation with A1
            HashMap<String, HashSet<String>>,
        )>,
    >;

    let state: SpreadsheetState =
        Arc::new(Mutex::new((HashMap::new(), HashMap::new(), HashMap::new())));

    let timestamp_map: Arc<RwLock<HashMap<String, u64>>> = Arc::new(RwLock::new(HashMap::new()));

    // creates a scope to prevent lifetime issues and join everything in the end
    thread::scope(|s| {
        while let Connection::NewConnection { mut reader, mut writer } = manager.accept_new_connection() {
            let state_clone = Arc::clone(&state);
        
            let timestamp_map_clone = Arc::clone(&timestamp_map);
            s.spawn(move || {
                loop {
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
                                        // get curr time
                                        let update_timestamp = get_current_timestamp();
                                        let cell_string = cell_to_string(cell_identifier);

                                        // get a peek of the current values by locking for abit
                                        // once it goes out of scope lock drops so its unlocked
                                        let spreadsheet_clone = {
                                            let lock = state_clone.lock().unwrap();
                                            lock.0.clone()
                                        };
                                    
                                        // evaluate formula outside the lock this will wait but not lock the spreadsheet
                                        let prepared = set::prepare_set(cell_to_string(cell_identifier), cell_expr, &spreadsheet_clone);

                                        
                                        let mut timestamp_lock = timestamp_map_clone.write().unwrap();
                                        if let Some(last_update_time) = timestamp_lock.get(&cell_string) {
                                            if *last_update_time > update_timestamp {
                                                // if the last time u updated is more recent so don't perfrom this
                                                continue;
                                            }
                                        }

                                        // insert latest timestamp
                                        timestamp_lock.insert(cell_string.clone(), update_timestamp);
                                    
                                        // once value is returned we lock it to set the stuff up
                                        let mut lock = state_clone.lock().unwrap();
                                        let (spreadsheet, depends_on, depends_by) = &mut *lock;
                                        set::apply_set(prepared, spreadsheet, depends_on, depends_by);
                                    
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
                }
            });
        }
    });
    
    Ok(())
}
