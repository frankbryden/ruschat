use rusqlite::{Connection, OpenFlags};
use std::time::SystemTime;
use crate::event::{event_kind_as_char, get_event_username, get_event_text, Event};


/* Type is a char representing the event type:
    * M: message
    * J: join
    * L: leave
    * 
*/
const DB_CREATE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS events(
    timestamp INTEGER,
    type CHAR(1),
    user VARCHAR(64),
    text VARCHAR(150),
    channel VARCHAR(64)
  );";

const RECORD_EVENT_SQL: &str = "INSERT INTO events (timestamp, type, user, text, channel) VALUES (?1, ?2, ?3, ?4, ?5)";

pub struct DbHandler {
    conn: Connection,
}

impl DbHandler {
    pub fn new() -> Self {
        DbHandler {
            conn: Connection::open_with_flags("messages.db", OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE)
                .unwrap(),
        }
    }

    pub fn initialize(&mut self) {
        match self.conn.execute(DB_CREATE_TABLE_SQL, ()) {
            Ok(_) => {}
            Err(e) => println!("Failed to create database: {e}"),
        }
    }

    pub fn record_event(&mut self, e: Event) {
        let event_type = event_kind_as_char(&e.kind);
        let event_user = match get_event_username(&e) {
            Some(user) => user,
            None => return,
            };
        println!("Record message");
        let event_text = get_event_text(&e).unwrap_or_default();
        match self.conn.execute(
            RECORD_EVENT_SQL,
            (e.timestamp, event_type.to_string(), event_user, event_text, "default"),
        ) {
            Ok(res) => {println!("Wrote {res} rows")}
            Err(e) => println!("Failed to insert event: {e}"),
        }
        self.conn.cache_flush().unwrap();
    }
}
