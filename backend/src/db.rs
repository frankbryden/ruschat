use rusqlite::{Connection, OpenFlags};
use std::time::SystemTime;
use crate::event::{event_kind_as_char, get_event_username, get_event_text, Event, EventKind};
use tokio_tungstenite::tungstenite::protocol::Message;


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
const READ_EVENTS_SQL: &str = "SELECT * FROM (SELECT * FROM events WHERE timestamp < ?1 LIMIT ?2) subquery ORDER BY timestamp DESC";

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
            (e.timestamp, event_type.to_string(), event_user, event_text, &e.channel),
        ) {
            Ok(res) => {println!("Wrote {res} rows")}
            Err(e) => println!("Failed to insert event: {e}"),
        }
        self.conn.cache_flush().unwrap();
    }

    pub fn read_events(&self, start: SystemTime, count: u32) -> Vec<Event> {
        let mut res = self.conn.prepare_cached(READ_EVENTS_SQL).unwrap();
        let events = res.query_map((start.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64, count), |row| {
            Ok(event_from_db_entry(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        });
        events.unwrap().map(|res| res.unwrap()).collect()
    }
}

fn event_from_db_entry(timestamp: u64, kind: String, user: String, text: String, channel: String) -> Event {
    let kind = match kind.as_str() {
        "M" => EventKind::ClientMessage((user, Message::text(text))),
        "J" => EventKind::ClientLogin((user, Vec::new())), //J for join
        "L" => EventKind::ClientLogout((user, Vec::new())), //L for leave
        _ => panic!("Unknown kind {kind}"),
    };
    Event {
        timestamp,
        kind,
        channel,
    }
}