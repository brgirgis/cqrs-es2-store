use std::collections::HashMap;

use rusqlite::Connection;

use cqrs_es2::{
    example_impl::*,
    AggregateContext,
    EventContext,
};

use crate::{
    sqlite_store::EventStore,
    IEventStore,
};

use super::common::*;

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

pub fn get_metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn test_save_load_events() {
    // "sqlite://demo.db"
    let conn = Connection::open(DB_NAME).unwrap();

    let mut store = ThisEventStore::new(conn);

    let id = uuid::Uuid::new_v4().to_string();

    let stored_events = store.load_events(&id).unwrap();
    assert_eq!(0, stored_events.len());

    let metadata = get_metadata();

    let mut contexts_0 = vec![EventContext::new(
        id.to_string(),
        1,
        CustomerEvent::NameAdded(NameAdded {
            changed_name: "test_event_A".to_string(),
        }),
        metadata,
    )];

    store.save_events(&contexts_0).unwrap();

    let stored_events = store.load_events(&id).unwrap();
    assert_eq!(stored_events, contexts_0);

    let metadata = get_metadata();

    let mut contexts_1 = vec![
        EventContext::new(
            id.to_string(),
            2,
            CustomerEvent::EmailUpdated(EmailUpdated {
                new_email: "test A".to_string(),
            }),
            metadata.clone(),
        ),
        EventContext::new(
            id.to_string(),
            3,
            CustomerEvent::EmailUpdated(EmailUpdated {
                new_email: "test B".to_string(),
            }),
            metadata.clone(),
        ),
        EventContext::new(
            id.to_string(),
            4,
            CustomerEvent::AddressUpdated(AddressUpdated {
                new_address: "something else happening here"
                    .to_string(),
            }),
            metadata.clone(),
        ),
    ];

    store.save_events(&contexts_1).unwrap();
    let stored_events = store.load_events(&id).unwrap();

    contexts_0.append(&mut contexts_1);
    assert_eq!(stored_events, contexts_0);
}

#[test]
fn test_save_load_snapshots() {
    // "sqlite://demo.db"
    let conn = Connection::open(DB_NAME).unwrap();

    let mut store = ThisEventStore::new(conn);

    let id = uuid::Uuid::new_v4().to_string();

    let stored_context = store
        .load_aggregate_from_snapshot(&id)
        .unwrap();

    assert_eq!(
        stored_context,
        AggregateContext::new(id.to_string(), 0, Default::default())
    );

    let context = AggregateContext::new(
        id.to_string(),
        1,
        Customer {
            customer_id: "customer 1".to_string(),
            name: "test name".to_string(),
            email: "test@email.com".to_string(),
            addresses: vec!["initial address".to_string()],
        },
    );

    store
        .save_aggregate_snapshot(context.clone())
        .unwrap();

    let stored_context = store
        .load_aggregate_from_snapshot(&id)
        .unwrap();

    assert_eq!(stored_context, context);

    let context = AggregateContext::new(
        id.to_string(),
        2,
        Customer {
            customer_id: "customer 2".to_string(),
            name: "test name 2".to_string(),
            email: "test2@email.com".to_string(),
            addresses: vec![
                "initial address".to_string(),
                "second address".to_string(),
            ],
        },
    );

    store
        .save_aggregate_snapshot(context.clone())
        .unwrap();

    let stored_context = store
        .load_aggregate_from_snapshot(&id)
        .unwrap();

    assert_eq!(stored_context, context);
}
