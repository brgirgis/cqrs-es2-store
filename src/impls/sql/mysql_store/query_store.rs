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
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
    IQuery,
    QueryContext,
};

use crate::repository::{
    IEventDispatcher,
    IQueryStore,
};

use super::super::mysql_constants::*;

/// Sync MySql/MariaDB query store
pub struct QueryStore<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    Q: IQuery<C, E>,
> {
    conn: PooledConn,
    _phantom: PhantomData<(C, E, A, Q)>,
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > QueryStore<C, E, A, Q>
{
    /// Constructor
    pub fn new(conn: PooledConn) -> Self {
        let x = Self {
            conn,
            _phantom: PhantomData,
        };

        trace!("Created new sync MySQL query store");

        x
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IQueryStore<C, E, A, Q> for QueryStore<C, E, A, Q>
{
    /// saves the updated query
    fn save_query(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), Error> {
        let aggregate_type = A::aggregate_type();
        let query_type = Q::query_type();

        let aggregate_id = context.aggregate_id;

        debug!(
            "storing a new query '{}' for aggregate id '{}'",
            query_type, &aggregate_id
        );

        let sql = match context.version {
            1 => INSERT_QUERY,
            _ => UPDATE_QUERY,
        };

        let payload = match serde_json::to_string(&context.payload) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to serialize the payload of query \
                         '{}' with aggregate id '{}', error: {}",
                        &query_type, &aggregate_id, e,
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
                &query_type,
            ),
        ) {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to insert/update query for \
                         aggregate id '{}' with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        Ok(())
    }

    /// loads the most recent query
    fn load_query(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, Error> {
        let aggregate_type = A::aggregate_type();
        let query_type = Q::query_type();

        trace!(
            "loading query '{}' for aggregate id '{}'",
            query_type,
            aggregate_id
        );

        let result: Option<Row> = match self.conn.exec_first(
            SELECT_QUERY,
            (
                &aggregate_type,
                &aggregate_id,
                &query_type,
            ),
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "unable to load queries table for query \
                         '{}' with aggregate id '{}', error: {}",
                        &query_type, &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        let row = match result {
            Some(x) => x,
            None => {
                trace!(
                    "returning default query '{}' for aggregate id \
                     '{}'",
                    query_type,
                    aggregate_id
                );

                return Ok(QueryContext::new(
                    aggregate_id.to_string(),
                    0,
                    Default::default(),
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
                        "bad payload found in queries table for \
                         query '{}' with aggregate id '{}', error: \
                         {}",
                        &query_type, &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        Ok(QueryContext::new(
            aggregate_id.to_string(),
            version,
            payload,
        ))
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IEventDispatcher<C, E> for QueryStore<C, E, A, Q>
{
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &Vec<EventContext<C, E>>,
    ) -> Result<(), Error> {
        self.dispatch_events(aggregate_id, events)
    }
}
