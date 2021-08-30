use rusqlite::Connection;

use cqrs_es2::{
    example_impl::*,
    QueryContext,
};

use crate::{
    sqlite_store::QueryStore,
    IQueryStore,
};

use super::common::*;

type ThisQueryStore = QueryStore<
    CustomerCommand,
    CustomerEvent,
    Customer,
    CustomerContactQuery,
>;

#[test]
fn test_save_load_queries() {
    // "sqlite://demo.db"
    let conn = Connection::open(DB_NAME).unwrap();

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
}
