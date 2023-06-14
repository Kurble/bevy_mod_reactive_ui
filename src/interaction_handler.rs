use bevy::prelude::*;
use std::{marker::PhantomData, sync::Arc};

use crate::base_handler::{Handler, HandlerImpl, HandlerParam};

#[derive(Component)]
pub struct InteractionHandler<Filter: InteractionFilter> {
    handler: Arc<dyn Handler<In = (), Out = ()>>,
    previous: Interaction,
    marker: PhantomData<Filter>,
}

pub struct OnClick;

pub struct OnClickEnd;

pub struct OnHover;

pub struct OnHoverEnd;

pub trait InteractionFilter: Send + Sync + 'static {
    fn filter(from: &Interaction, to: &Interaction) -> bool;
}

pub trait SetInteractionHandler: Sized {
    fn on_click<T, U>(self, on_click: T) -> (Self, InteractionHandler<OnClick>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static;

    fn on_click_end<T, U>(self, on_click_end: T) -> (Self, InteractionHandler<OnClickEnd>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static;

    fn on_hover<T, U>(self, on_hover: T) -> (Self, InteractionHandler<OnHover>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static;

    fn on_hover_end<T, U>(self, on_hover_end: T) -> (Self, InteractionHandler<OnHoverEnd>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static;
}

impl<B> SetInteractionHandler for B
where
    B: Bundle,
{
    fn on_click<T, U>(self, handler: T) -> (Self, InteractionHandler<OnClick>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static,
    {
        (self, InteractionHandler::new(handler))
    }

    fn on_click_end<T, U>(self, handler: T) -> (Self, InteractionHandler<OnClickEnd>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static,
    {
        (self, InteractionHandler::new(handler))
    }

    fn on_hover<T, U>(self, handler: T) -> (Self, InteractionHandler<OnHover>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static,
    {
        (self, InteractionHandler::new(handler))
    }

    fn on_hover_end<T, U>(self, handler: T) -> (Self, InteractionHandler<OnHoverEnd>)
    where
        T: SystemParamFunction<U, In = (), Out = ()>,
        U: 'static,
    {
        (self, InteractionHandler::new(handler))
    }
}

impl<Filter: InteractionFilter> InteractionHandler<Filter> {
    pub fn new<F, FMarker>(handler: F) -> Self
    where
        F: SystemParamFunction<FMarker, In = (), Out = ()>,
        FMarker: 'static,
    {
        Self {
            handler: Arc::new(HandlerImpl::new(handler)),
            previous: Interaction::None,
            marker: PhantomData,
        }
    }
}

impl InteractionFilter for OnClick {
    fn filter(_from: &Interaction, to: &Interaction) -> bool {
        matches!(to, &Interaction::Clicked)
    }
}

impl InteractionFilter for OnClickEnd {
    fn filter(from: &Interaction, _to: &Interaction) -> bool {
        matches!(from, &Interaction::Clicked)
    }
}

impl InteractionFilter for OnHover {
    fn filter(from: &Interaction, to: &Interaction) -> bool {
        matches!(from, &Interaction::None) && matches!(to, &Interaction::Hovered)
    }
}

impl InteractionFilter for OnHoverEnd {
    fn filter(from: &Interaction, to: &Interaction) -> bool {
        matches!(from, &Interaction::Hovered) && matches!(to, &Interaction::None)
    }
}

pub(crate) fn make_interaction_handler_system<Filter: InteractionFilter>(
) -> impl System<In = (), Out = ()> {
    gather::<Filter>.pipe(run)
}

fn gather<Filter: InteractionFilter>(
    mut handlers: Query<(&mut InteractionHandler<Filter>, &Interaction), Changed<Interaction>>,
    mut unapplied: HandlerParam<(), ()>,
) -> Vec<Arc<dyn Handler<In = (), Out = ()>>> {
    unapplied.clear();
    unapplied.extend(
        handlers
            .iter_mut()
            .filter_map(|(mut handler, interaction)| {
                if Filter::filter(&handler.previous, interaction) {
                    handler.previous = interaction.clone();
                    Some(handler.handler.clone())
                } else {
                    handler.previous = interaction.clone();
                    None
                }
            }),
    );
    unapplied.clone()
}

fn run(In(handlers): In<Vec<Arc<dyn Handler<In = (), Out = ()>>>>, world: &mut World) {
    for handler in handlers {
        handler.handle(world, ());
    }
}
