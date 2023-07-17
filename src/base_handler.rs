use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use std::sync::{Arc, Mutex};

pub(crate) trait Handler: Send + Sync + 'static {
    type In;

    type Out;

    fn handle(&self, world: &mut World, input: Self::In) -> Self::Out;

    fn apply(&self, world: &mut World);
}

#[derive(Deref, DerefMut)]
pub(crate) struct HandlerParam<'s, In, Out>(
    pub(crate) &'s mut Vec<Arc<dyn Handler<In = In, Out = Out>>>,
);

pub(crate) struct HandlerImpl<S, M, I, O>
where
    S: SystemParamFunction<M, In = I, Out = O>,
    S::Param: 'static,
{
    inner: Mutex<HandlerImplInner<SystemState<S::Param>, S>>,
}

struct HandlerImplInner<T, U> {
    state: Option<T>,
    system: U,
}

impl<S, M, I, O> HandlerImpl<S, M, I, O>
where
    S: SystemParamFunction<M, In = I, Out = O>,
    S::Param: 'static,
{
    pub fn new(sys: S) -> Self {
        Self {
            inner: Mutex::new(HandlerImplInner {
                state: None,
                system: sys,
            }),
        }
    }

    fn handle(
        inner: &mut HandlerImplInner<SystemState<S::Param>, S>,
        world: &mut World,
        input: I,
    ) -> O {
        inner.system.run(
            input,
            inner
                .state
                .get_or_insert_with(|| SystemState::<S::Param>::new(world))
                .get_mut(world),
        )
    }
}

impl<S, M, I, O> Handler for HandlerImpl<S, M, I, O>
where
    S: SystemParamFunction<M, In = I, Out = O>,
    S::Param: 'static,
    M: 'static,
    I: 'static,
    O: 'static,
{
    type In = I;

    type Out = O;

    fn handle(&self, world: &mut World, input: I) -> O {
        let Ok(mut inner) = self.inner.lock() else {
            panic!();
        };
        Self::handle(&mut inner, world, input)
    }

    fn apply(&self, world: &mut World) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        let Some(ref mut state) = inner.state else {
            return;
        };
        state.apply(world);
    }
}

unsafe impl<'s, In: 'static, Out: 'static> SystemParam for HandlerParam<'s, In, Out> {
    type State = Vec<Arc<dyn Handler<In = In, Out = Out>>>;

    type Item<'world, 'state> = HandlerParam<'state, In, Out>;

    fn init_state(_: &mut World, _: &mut bevy::ecs::system::SystemMeta) -> Self::State {
        vec![]
    }

    fn apply(state: &mut Self::State, _: &bevy::ecs::system::SystemMeta, world: &mut World) {
        for handler in state.iter_mut() {
            handler.apply(world);
        }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        _: &bevy::ecs::system::SystemMeta,
        _: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        _: bevy::ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        HandlerParam(state)
    }
}
