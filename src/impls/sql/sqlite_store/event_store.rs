use log::{
    debug,
    trace,
};
use std::marker::PhantomData;

use rusqlite::{
    params,
    Connection,
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

static CREATE_EVENTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS
    events
    (
        aggregate_type TEXT                         NOT NULL,
        aggregate_id   TEXT                         NOT NULL,
        sequence       bigint CHECK (sequence >= 0) NOT NULL,
        payload        TEXT                         NOT NULL,
        metadata       TEXT                         NOT NULL,
        timestamp      timestamp DEFAULT (CURRENT_TIMESTAMP),
        PRIMARY KEY (aggregate_type, aggregate_id, sequence)
    );
";

static CREATE_SNAPSHOT_TABLE: &str = "
CREATE TABLE IF NOT EXISTS
    snapshots
    (
        aggregate_type TEXT                              NOT NULL,
        aggregate_id   TEXT                              NOT NULL,
        version        bigint       CHECK (version >= 0) NOT NULL,
        payload        TEXT                              NOT NULL,
        timestamp      timestamp DEFAULT (CURRENT_TIMESTAMP),
        PRIMARY KEY (aggregate_type, aggregate_id)
    );
";

/// SQLite storage
pub struct EventStore<C: ICommand, E: IEvent, A: IAggregate<C, E>> {
    conn: Connection,
    _phantom: PhantomData<(C, E, A)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    EventStore<C, E, A>
{
    /// Constructor
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            _phantom: PhantomData,
        }
    }

    fn create_events_table(&mut self) -> Result<(), Error> {
        match self
            .conn
            .execute(CREATE_EVENTS_TABLE, [])
        {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to create events table with error: \
                         {}",
                        e
                    )
                    .as_str(),
                ));
            },
        };

        debug!("Created events table",);

        Ok(())
    }

    fn create_snapshot_table(&mut self) -> Result<(), Error> {
        match self
            .conn
            .execute(CREATE_SNAPSHOT_TABLE, [])
        {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to create snapshots table with \
                         error: {}",
                        e
                    )
                    .as_str(),
                ));
            },
        };

        debug!("Created snapshots table",);

        Ok(())
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
        self.create_events_table()?;

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

            match self.conn.execute(
                INSERT_EVENT,
                params![
                    aggregate_type,
                    aggregate_id,
                    context.sequence,
                    payload,
                    metadata,
                ],
            ) {
                Ok(x) => x,
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
            };
        }

        Ok(())
    }

    /// Load all events for a particular `aggregate_id`
    fn load_events(
        &mut self,
        aggregate_id: &str,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        self.create_events_table()?;

        let aggregate_type = A::aggregate_type();

        trace!(
            "loading events for aggregate id '{}'",
            aggregate_id
        );

        let mut sql = match self.conn.prepare(SELECT_EVENTS) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to prepare events table for \
                         aggregate id '{}', error: {}",
                        &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        let rows = match sql.query_map(
            params![aggregate_type, aggregate_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to load queries table for aggregate \
                         id '{}', error: {}",
                        &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        let mut result = Vec::new();

        for row in rows {
            let row: (i64, String, String) = row.unwrap();

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
        self.create_snapshot_table()?;

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

        match self.conn.execute(
            sql,
            params![
                context.version,
                payload,
                aggregate_type,
                aggregate_id,
            ],
        ) {
            Ok(x) => x,
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
        self.create_snapshot_table()?;

        let aggregate_type = A::aggregate_type();

        trace!(
            "loading snapshot for aggregate id '{}'",
            aggregate_id
        );

        let mut sql = match self.conn.prepare(SELECT_SNAPSHOT) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to prepare snapshots table for \
                         aggregate id '{}', error: {}",
                        &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        let res = match sql.query_map(
            params![aggregate_type, aggregate_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to load snapshots table for \
                         aggregate id '{}', error: {}",
                        &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        let mut rows: Vec<(i64, String)> = Vec::new();

        for x in res {
            rows.push(x.unwrap());
        }

        if rows.len() == 0 {
            trace!(
                "returning default aggregate for aggregate id
        '{}'",
                aggregate_id
            );

            return Ok(AggregateContext::new(
                aggregate_id.to_string(),
                0,
                A::default(),
            ));
        };

        let row = rows[0].clone();

        let payload = match serde_json::from_str(row.1.as_str()) {
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
            row.0,
            payload,
        ))
    }
}
