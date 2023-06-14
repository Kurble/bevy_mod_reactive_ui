use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub struct Mount<'w, 's> {
    root: Local<'s, Vec<Child>>,
    commands: Commands<'w, 's>,
}

pub struct Updater<'a, 'w, 's> {
    parent: Option<Entity>,
    commands: &'a mut Commands<'w, 's>,
    children: &'a mut Vec<Child>,
    cursor: usize,
}

struct Child {
    id: u64,
    entity: Entity,
    children: Vec<Self>,
}

impl<'w, 's> Mount<'w, 's> {
    pub fn update<F>(&mut self, children: F)
    where
        F: FnOnce(&mut Updater),
    {
        children(&mut Updater {
            parent: None,
            commands: &mut self.commands,
            children: &mut *self.root,
            cursor: 0,
        });
    }
}

impl<'a, 'w, 's> Updater<'a, 'w, 's> {
    pub fn with<F: FnOnce(&mut Self)>(mut self, f: F) {
        f(&mut self);
    }

    /// Insert or update a node. The uid must be unique.
    /// If the entity already exists, it's bundle is not updated.
    /// The children of the node will be updated using the closure passed in `children`.
    pub fn node<'b, F, T>(&'b mut self, uid: u64, bundle: F) -> Updater<'b, 'w, 's>
    where
        F: FnOnce() -> T,
        T: Bundle,
    {
        if self.find_uid(uid) {
            self.cursor += 1;
            self.children[self.cursor - 1].updater(&mut *self.commands)
        } else {
            self.insert(uid, bundle())
        }
    }

    /// Insert or update a node. The uid must be unique.
    /// The children of the node will be updated using the closure passed in `children`.
    /// Reinserts the bundle even if entity already exists.
    pub fn dyn_node<'b, T>(&'b mut self, uid: u64, bundle: T) -> Updater<'b, 'w, 's>
    where
        T: Bundle,
    {
        if self.find_uid(uid) {
            self.commands
                .entity(self.children[self.cursor].entity)
                .insert(bundle);
            self.cursor += 1;
            self.children[self.cursor - 1].updater(&mut *self.commands)
        } else {
            self.insert(uid, bundle)
        }
    }

    /// Attempt to find a child with the queried uid. Returns `true` if it was found.
    /// After calling this function, `self.cursor` is set to the new position for the queried uid.
    fn find_uid(&mut self, uid: u64) -> bool {
        for i in self.cursor..self.children.len() {
            if self.children[i].id == uid {
                // in-between children are considered to have disappeared.
                for mut child in self.children.drain(self.cursor..i) {
                    child.destroy(&mut self.commands);
                }
                // uid has been found, and it's now at self.cursor.
                return true;
            }
        }
        // uid not found, this child is considered to have newly appeared.
        false
    }

    /// Spawn and insert a new entity at `self.cursor`.
    fn insert<'b, T>(&'b mut self, uid: u64, bundle: T) -> Updater<'b, 'w, 's>
    where
        T: Bundle,
    {
        self.children.insert(
            self.cursor,
            Child {
                id: uid,
                entity: if let Some(parent) = self.parent {
                    self.commands.spawn(bundle).set_parent(parent).id()
                } else {
                    self.commands.spawn(bundle).id()
                },
                children: vec![],
            },
        );
        self.cursor += 1;
        self.children[self.cursor - 1].updater(&mut *self.commands)
    }
}

impl<'a, 'w, 's> Drop for Updater<'a, 'w, 's> {
    fn drop(&mut self) {
        for mut child in self.children.drain(self.cursor..self.children.len()) {
            child.destroy(&mut *self.commands);
        }
    }
}

impl Child {
    fn updater<'a, 'w, 's>(
        &'a mut self,
        commands: &'a mut Commands<'w, 's>,
    ) -> Updater<'a, 'w, 's> {
        Updater {
            parent: Some(self.entity),
            children: &mut self.children,
            commands,
            cursor: 0,
        }
    }

    fn destroy(&mut self, commands: &mut Commands) {
        // TODO: replace this with animated destruction
        commands.entity(self.entity).despawn_recursive();
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
