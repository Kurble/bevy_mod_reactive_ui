use bevy::ecs::system::{Command, SystemParam};
use bevy::prelude::*;
use smallvec::SmallVec;

use crate::transition::{DefaultTransition, Transition};

#[derive(SystemParam)]
pub struct Mount<'w, 's> {
    root: Local<'s, Vec<Child>>,
    query: Query<'w, 's, (&'static Node, &'static Transform)>,
    commands: Commands<'w, 's>,
}

pub struct Updater<'a, 'w, 's, T: Transition> {
    parent: Option<Entity>,
    commands: &'a mut Commands<'w, 's>,
    children: &'a mut Vec<Child>,
    query: &'a Query<'w, 's, (&'static Node, &'static Transform)>,
    cursor: usize,
    transition: T,
    skip_transition: bool,
}

struct Child {
    uid: u64,
    entity: Entity,
    children: Vec<Self>,
}

struct InsertChildrenInOrder {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl<'w, 's> Mount<'w, 's> {
    pub fn update<F>(&mut self, children: F)
    where
        F: FnOnce(&mut Updater<DefaultTransition>),
    {
        self.update_with_animation(DefaultTransition, children);
    }

    pub fn update_with_animation<F, T>(&mut self, transition: T, children: F)
    where
        F: FnOnce(&mut Updater<T>),
        T: Transition,
    {
        children(&mut Updater {
            parent: None,
            commands: &mut self.commands,
            children: &mut *self.root,
            query: &self.query,
            cursor: 0,
            transition,
            skip_transition: false,
        });
    }
}

impl<'a, 'w, 's, T> Updater<'a, 'w, 's, T>
where
    T: Transition,
{
    pub fn with<F: FnOnce(&mut Self)>(mut self, f: F) {
        f(&mut self);
    }

    pub fn with_animation<F: for<'b> FnOnce(&mut Updater<'b, 'w, 's, U>), U: Transition>(
        mut self,
        transition: U,
        f: F,
    ) {
        let mut sub = Updater::<U> {
            children: &mut *self.children,
            commands: &mut *self.commands,
            query: self.query,
            cursor: self.cursor,
            parent: self.parent.take(),
            transition,
            skip_transition: self.skip_transition,
        };

        f(&mut sub);

        self.cursor = sub.cursor;
    }

    /// Insert or update a node. The uid must be unique.
    /// If the entity already exists, it's bundle is not updated.
    /// The children of the node will be updated using the closure passed in `children`.
    pub fn node<'b, F, B>(&'b mut self, uid: u64, bundle: F) -> Updater<'b, 'w, 's, T>
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        if self.find_uid(uid) {
            self.cursor += 1;
            self.children[self.cursor - 1].updater(
                &mut *self.commands,
                self.query,
                self.transition,
                self.skip_transition,
            )
        } else {
            self.insert(uid, bundle())
        }
    }

    /// Insert or update a node. The uid must be unique.
    /// The children of the node will be updated using the closure passed in `children`.
    /// Reinserts the bundle even if entity already exists.
    pub fn dyn_node<'b, B>(&'b mut self, uid: u64, bundle: B) -> Updater<'b, 'w, 's, T>
    where
        B: Bundle,
    {
        if self.find_uid(uid) {
            self.commands
                .entity(self.children[self.cursor].entity)
                .insert(bundle);
            self.cursor += 1;
            self.children[self.cursor - 1].updater(
                &mut *self.commands,
                self.query,
                self.transition,
                self.skip_transition,
            )
        } else {
            self.insert(uid, bundle)
        }
    }

    /// Attempt to find a child with the queried uid. Returns `true` if it was found.
    /// After calling this function, `self.cursor` is set to the new position for the queried uid.
    fn find_uid(&mut self, uid: u64) -> bool {
        for i in self.cursor..self.children.len() {
            if self.children[i].uid == uid {
                // in-between children are considered to have disappeared.
                for child in self.children.drain(self.cursor..i) {
                    self.transition.remove(self.commands.entity(child.entity));
                }
                // uid has been found, and it's now at self.cursor.
                let entity = self.children[self.cursor].entity;
                if let Ok((node, transform)) = self.query.get(entity) {
                    self.transition.update(
                        self.commands.entity(entity),
                        Rect::from_center_size(transform.translation.truncate(), node.size()),
                    );
                }
                return true;
            }
        }
        // uid not found, this child is considered to have newly appeared.
        false
    }

    /// Spawn and insert a new entity at `self.cursor`.
    fn insert<'b, B>(&'b mut self, uid: u64, bundle: B) -> Updater<'b, 'w, 's, T>
    where
        B: Bundle,
    {
        let entity = self.commands.spawn(bundle);
        let child = Child {
            uid,
            entity: if self.skip_transition {
                entity.id()
            } else {
                self.transition.insert(entity)
            },
            children: vec![],
        };

        self.children.insert(self.cursor, child);
        self.cursor += 1;
        self.children[self.cursor - 1].updater(
            &mut *self.commands,
            self.query,
            self.transition,
            true,
        )
    }
}

impl<'a, 'w, 's, T> Drop for Updater<'a, 'w, 's, T>
where
    T: Transition,
{
    fn drop(&mut self) {
        for child in self.children.drain(self.cursor..self.children.len()) {
            self.transition.remove(self.commands.entity(child.entity));
        }

        if let Some(parent) = self.parent {
            self.commands.add(InsertChildrenInOrder {
                parent,
                children: self.children.iter().map(|c| c.entity).collect(),
            });
        }
    }
}

impl Child {
    fn updater<'a, 'w, 's, T>(
        &'a mut self,
        commands: &'a mut Commands<'w, 's>,
        query: &'a Query<'w, 's, (&'static Node, &'static Transform)>,
        transition: T,
        skip_transition: bool,
    ) -> Updater<'a, 'w, 's, T>
    where
        T: Transition,
    {
        Updater {
            parent: Some(self.entity),
            children: &mut self.children,
            commands,
            query,
            cursor: 0,
            transition,
            skip_transition,
        }
    }
}

impl Command for InsertChildrenInOrder {
    fn write(self, world: &mut World) {
        let mut parent = world.entity_mut(self.parent);

        if let Some(existing) = parent
            .get::<Children>()
            .map(|c| c.iter().copied().collect::<SmallVec<[Entity; 8]>>())
        {
            let mut offset = 0;
            for new in self.children {
                if let Some(i) = existing[offset..].iter().position(|&e| e == new) {
                    offset += i;
                } else {
                    parent.insert_children(offset, &[new]);
                }
                offset += 1;
            }
        } else {
            parent.push_children(&self.children);
        }
    }
}

#[macro_export]
macro_rules! id {
    () => {{
        use std::hash::Hasher;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        h.write(file!().as_bytes());
        h.write_u32(line!());
        h.write_u32(column!());
        h.finish()
    }};

    ($hashable:expr) => {{
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        h.write(file!().as_bytes());
        h.write_u32(line!());
        h.write_u32(column!());
        $hashable.hash(&mut h);
        h.finish()
    }};
}
