use cqrs_es2::{
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
    IQuery,
    QueryContext,
};

use super::i_event_dispatcher::IEventDispatcher;

/// The abstract central source for loading and committing
/// queries.
pub trait IQueryStore<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    Q: IQuery<C, E>,
>: IEventDispatcher<C, E> {
    /// saves the updated query
    fn save_query(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), Error>;

    /// loads the most recent query
    fn load_query(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, Error>;

    /// used as a default implementation for dispatching
    fn dispatch_events(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), Error> {
        let mut context = match self.load_query(aggregate_id) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            },
        };

        for event in events {
            context.payload.update(event);
        }

        context.version += 1;

        self.save_query(context)
    }
}
