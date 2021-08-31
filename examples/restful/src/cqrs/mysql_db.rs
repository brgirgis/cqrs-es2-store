use mysql::{
    Opts,
    Pool,
    PooledConn,
};

pub fn db_connection() -> Result<PooledConn, mysql::Error> {
    let opts = Opts::from_url(
        "mysql://test_user:test_pass@localhost:9083/test",
    )?;
    let pool = Pool::new(opts)?;
    let conn = pool.get_conn()?;

    Ok(conn)
}
