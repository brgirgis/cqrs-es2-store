use log::{
    debug,
    trace,
};
use std::marker::PhantomData;

use mysql::PooledConn;

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

        // let query_instance_id = &self.query_instance_id;
        let payload = match serde_json::to_value(&context.payload) {
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

        match self.conn.execute(
            sql,
            &[
                context.version,
                &payload,
                &aggregate_type,
                &aggregate_id,
                &query_type,
            ],
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(Error::new(e.to_string().as_str()));
            },
        }
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

        let result = match self.conn.query(
            SELECT_QUERY,
            &[
                &aggregate_type,
                &aggregate_id,
                &query_type,
            ],
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(e.to_string().as_str()));
            },
        };

        let row = match result.iter().next() {
            Some(x) => x,
            None => {
                return Ok(QueryContext::new(
                    aggregate_id,
                    0,
                    Default::default(),
                ));
            },
        };

        let version = row.get(0);

        let payload = match serde_json::from_value(row.get(1)) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(e.to_string().as_str()));
            },
        };

        Ok(QueryContext::new(
            aggregate_id,
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
