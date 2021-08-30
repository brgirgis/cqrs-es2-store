use postgres::{
    Client,
    Error,
    NoTls,
};

pub fn db_connection() -> Result<Client, Error> {
    let conn = Client::connect(
        "postgresql://test_user:test_pass@localhost:9084/test",
        NoTls,
    )
    .unwrap();

    Ok(conn)
}
