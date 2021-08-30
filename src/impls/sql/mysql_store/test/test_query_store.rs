use mysql::{
    Error,
    Opts,
    Pool,
};

use cqrs_es2::{
    example_impl::*,
    QueryContext,
};

use crate::{
    mysql_store::QueryStore,
    IQueryStore,
};

use super::common::*;

type ThisQueryStore = QueryStore<
    CustomerCommand,
    CustomerEvent,
    Customer,
    CustomerContactQuery,
>;

type ThisQueryContext = QueryContext<
    CustomerCommand,
    CustomerEvent,
    CustomerContactQuery,
>;

fn check_save_load_queries(uri: &str) -> Result<(), Error> {
    let opts = Opts::from_url(uri)?;
    let pool = Pool::new(opts)?;
    let conn = pool.get_conn()?;

    let mut store = ThisQueryStore::new(conn);

    let id = uuid::Uuid::new_v4().to_string();

    let stored_context = store.load_query(&id).unwrap();

    assert_eq!(
        stored_context,
        QueryContext::new(id.to_string(), 0, Default::default())
    );

    let context = QueryContext::new(
        id.to_string(),
        1,
        CustomerContactQuery {
            name: "test name".to_string(),
            email: "test@email.com".to_string(),
            latest_address: "one address".to_string(),
        },
    );

    store
        .save_query(context.clone())
        .unwrap();

    let stored_context = store.load_query(&id).unwrap();

    assert_eq!(stored_context, context);

    let context = QueryContext::new(
        id.to_string(),
        2,
        CustomerContactQuery {
            name: "test name2".to_string(),
            email: "test2@email.com".to_string(),
            latest_address: "second address".to_string(),
        },
    );

    store
        .save_query(context.clone())
        .unwrap();

    let stored_context = store.load_query(&id).unwrap();

    assert_eq!(stored_context, context);

    Ok(())
}

#[test]
fn test_mariadb_save_load_queries() {
    check_save_load_queries(CONNECTION_STRING_MARIADB).unwrap();
}

#[test]
fn test_mysql_save_load_queries() {
    check_save_load_queries(CONNECTION_STRING_MYSQL).unwrap();
}
