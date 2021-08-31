use log::{
    debug,
    trace,
};
use std::marker::PhantomData;

use mysql::{
    prelude::Queryable,
    PooledConn,
    Row,
};

use cqrs_es2::{
    AggregateContext,
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
};

use crate::repository::IEventStore;

use super::super::mysql_constants::*;

/// Sync MySql/MariaDB event store
pub struct EventStore<C: ICommand, E: IEvent, A: IAggregate<C, E>> {
    conn: PooledConn,
    _phantom: PhantomData<(C, E, A)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    EventStore<C, E, A>
{
    /// Constructor
    pub fn new(conn: PooledConn) -> Self {
        let x = Self {
            conn,
            _phantom: PhantomData,
        };

        trace!("Created new sync MySQL event store");

        x
    }
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>> IEventStore<C, E, A>
    for EventStore<C, E, A>
{
    /// Save new events
    fn save_events(
        &mut self,
        contexts: &Vec<EventContext<C, E>>,
    ) -> Result<(), Error> {
        if contexts.len() == 0 {
            trace!("Skip saving zero contexts");
            return Ok(());
        }

        let aggregate_type = A::aggregate_type();

        let aggregate_id = contexts
            .first()
            .unwrap()
            .aggregate_id
            .clone();

        debug!(
            "storing '{}' new events for aggregate id '{}'",
            contexts.len(),
            &aggregate_id
        );

        for context in contexts {
            let payload =
                match serde_json::to_string(&context.payload) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(Error::new(
                            format!(
                                "unable to serialize the event \
                                 payload for aggregate id '{}' with \
                                 error: {}",
                                &aggregate_id, e
                            )
                            .as_str(),
                        ));
                    },
                };

            let metadata =
                match serde_json::to_string(&context.metadata) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(Error::new(
                            format!(
                                "unable to serialize the event \
                                 metadata for aggregate id '{}' \
                                 with error: {}",
                                &aggregate_id, e
                            )
                            .as_str(),
                        ));
                    },
                };

            match self.conn.exec_drop(
                INSERT_EVENT,
                (
                    &aggregate_type,
                    &aggregate_id,
                    context.sequence,
                    &payload,
                    &metadata,
                ),
            ) {
                Ok(_) => {},
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "unable to insert new event for \
                             aggregate id '{}' with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            }
        }

        Ok(())
    }

    /// Load all events for a particular `aggregate_id`
    fn load_events(
        &mut self,
        aggregate_id: &str,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        let aggregate_type = A::aggregate_type();

        trace!(
            "loading events for aggregate id '{}'",
            aggregate_id
        );

        let rows: Vec<(i64, String, String)> = match self.conn.exec(
            SELECT_EVENTS,
            (&aggregate_type, &aggregate_id),
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to load events table for aggregate \
                         id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        let mut result = Vec::new();

        for row in rows {
            let payload = match serde_json::from_str(row.1.as_str()) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "bad payload found in events table for \
                             aggregate id '{}' with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            let metadata = match serde_json::from_str(row.2.as_str())
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "bad metadata found in events table for \
                             aggregate id '{}' with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            result.push(EventContext::new(
                aggregate_id.to_string(),
                row.0,
                payload,
                metadata,
            ));
        }

        Ok(result)
    }

    /// save a new aggregate snapshot
    fn save_aggregate_snapshot(
        &mut self,
        context: AggregateContext<C, E, A>,
    ) -> Result<(), Error> {
        let aggregate_type = A::aggregate_type();

        let aggregate_id = context.aggregate_id;

        debug!(
            "storing a new snapshot for aggregate id '{}'",
            &aggregate_id
        );

        let sql = match context.version {
            1 => INSERT_SNAPSHOT,
            _ => UPDATE_SNAPSHOT,
        };

        let payload = match serde_json::to_string(&context.payload) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to serialize aggregate snapshot for \
                         aggregate id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        match self.conn.exec_drop(
            sql,
            (
                context.version,
                &payload,
                &aggregate_type,
                &aggregate_id,
            ),
        ) {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to insert/update snapshot for \
                         aggregate id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        Ok(())
    }

    /// Load aggregate at current state from snapshots
    fn load_aggregate_from_snapshot(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error> {
        let aggregate_type = A::aggregate_type();

        trace!(
            "loading snapshot for aggregate id '{}'",
            aggregate_id
        );

        let result: Option<Row> = match self.conn.exec_first(
            SELECT_SNAPSHOT,
            (&aggregate_type, &aggregate_id),
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to load snapshots table for \
                         aggregate id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        let row = match result {
            Some(x) => x,
            None => {
                trace!(
                    "returning default aggregate for aggregate id \
                     '{}'",
                    aggregate_id
                );

                return Ok(AggregateContext::new(
                    aggregate_id.to_string(),
                    0,
                    A::default(),
                ));
            },
        };

        let version: i64 = row.get(0).unwrap();
        let payload: String = row.get(1).unwrap();

        let payload = match serde_json::from_str(payload.as_str()) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "bad payload found in snapshots table for \
                         aggregate id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        Ok(AggregateContext::new(
            aggregate_id.to_string(),
            version,
            payload,
        ))
    }
}
