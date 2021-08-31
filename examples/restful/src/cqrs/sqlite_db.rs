use rusqlite::Connection;

pub fn db_connection() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("test.db").unwrap();

    Ok(conn)
}
