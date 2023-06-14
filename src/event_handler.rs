use bevy::prelude::*;
use std::sync::Arc;

use crate::base_handler::{Handler, HandlerImpl, HandlerParam};

#[derive(Component)]
pub struct EventHandler<E: 'static> {
    handler: Arc<dyn Handler<In = E, Out = ()>>,
}

pub trait SetEventHandler<E>: Sized
where
    E: Event,
{
    fn on_event<F, T>(self, on_event: F) -> (Self, EventHandler<E>)
    where
        F: SystemParamFunction<T, In = E, Out = ()>,
        T: 'static;
}

impl<B, E> SetEventHandler<E> for B
where
    B: Bundle,
    E: Event,
{
    fn on_event<F, T>(self, on_event: F) -> (Self, EventHandler<E>)
    where
        F: SystemParamFunction<T, In = E, Out = ()>,
        T: 'static,
    {
        (
            self,
            EventHandler {
                handler: Arc::new(HandlerImpl::new(on_event)),
            },
        )
    }
}

pub(crate) fn make_event_handler_system<T: Event + Clone>() -> impl System<In = (), Out = ()> {
    gather::<T>.pipe(run::<T>)
}

fn gather<T: Event + Clone>(
    mut events: EventReader<T>,
    mut handlers: Query<&mut EventHandler<T>>,
    mut unapplied: HandlerParam<T, ()>,
) -> (Vec<T>, Vec<Arc<dyn Handler<In = T, Out = ()>>>) {
    unapplied.clear();

    if events.is_empty() {
        return default();
    }

    let events = events.iter().cloned().collect();

    unapplied.extend(handlers.iter_mut().map(|h| h.handler.clone()));

    (events, unapplied.clone())
}

fn run<T: Event + Clone>(
    In((events, mut handlers)): In<(Vec<T>, Vec<Arc<dyn Handler<In = T, Out = ()>>>)>,
    world: &mut World,
) {
    for handler in &mut handlers {
        for event in &events {
            handler.handle(world, event.clone());
        }
    }
}
