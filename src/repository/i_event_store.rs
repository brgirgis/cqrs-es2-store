use cqrs_es2::{
    AggregateContext,
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
};

/// The abstract central source for loading past events and committing
/// new events.
pub trait IEventStore<C: ICommand, E: IEvent, A: IAggregate<C, E>> {
    /// Save new events
    fn save_events(
        &mut self,
        contexts: &Vec<EventContext<C, E>>,
    ) -> Result<(), Error>;

    /// Load all events for a particular `aggregate_id`
    fn load_events(
        &mut self,
        aggregate_id: &str,
    ) -> Result<Vec<EventContext<C, E>>, Error>;

    /// save a new aggregate snapshot
    fn save_aggregate_snapshot(
        &mut self,
        context: AggregateContext<C, E, A>,
    ) -> Result<(), Error>;

    /// Load aggregate at current state from snapshots
    fn load_aggregate_from_snapshot(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error>;
}
