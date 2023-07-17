use bevy::ecs::system::{Command, SystemParam};
use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};

use crate::transition::{DefaultTransition, Transition};

#[derive(SystemParam)]
pub struct ShadowScene<'w, 's> {
    root: Local<'s, Container>,
    commands: Commands<'w, 's>,
}

pub struct Shadow<'a, 'w, 's> {
    begin: usize,
    cursor: usize,
    len: usize,
    count: usize,
    changed: bool,

    parent: Option<Entity>,
    commands: &'a mut Commands<'w, 's>,
    children: &'a mut Vec<Child>,
    parent_cursor: &'a mut usize,

    transition: &'a dyn Transition,
    transition_root: bool,
}

struct Container(Vec<Child>);

struct Child {
    uid: u64,
    entity: Entity,
    size: usize,
    count: usize,
}

struct InsertChildrenInOrder {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl<'w, 's> ShadowScene<'w, 's> {
    pub fn update<F>(&mut self, fragment: F)
    where
        F: FnOnce(&mut Shadow),
    {
        self.update_with_transition(&DefaultTransition, fragment);
    }

    pub fn update_with_transition<F>(&mut self, transition: &dyn Transition, fragment: F)
    where
        F: FnOnce(&mut Shadow),
    {
        let mut cursor = 0;
        let mut updater = Shadow {
            begin: 1,
            cursor: 1,
            len: self.root.len(),
            count: 0,
            changed: false,

            parent: None,
            commands: &mut self.commands,
            children: self.root.get(),
            parent_cursor: &mut cursor,

            transition,
            transition_root: true,
        };
        fragment(&mut updater);
    }
}

impl<'a, 'w, 's> Shadow<'a, 'w, 's> {
    pub fn with<F: FnOnce(&mut Shadow)>(mut self, fragment: F) {
        fragment(&mut self);
    }

    pub fn with_transition<F: FnOnce(&mut Shadow)>(
        mut self,
        transition: &dyn Transition,
        fragment: F,
    ) {
        let mut updater = Shadow {
            begin: self.begin,
            cursor: self.cursor,
            count: self.count,
            len: self.len,
            changed: false,

            children: self.children,
            commands: self.commands,
            parent_cursor: self.parent_cursor,

            parent: self.parent.take(),
            transition,
            transition_root: self.transition_root,
        };

        fragment(&mut updater);

        self.cursor = updater.cursor;
        self.count = updater.count;
        self.len = updater.len;
    }

    /// Spawn or update an entity. The uid must be unique.
    /// If the entity already exists, it's bundle is not updated.
    /// The children of the node will be updated using the closure passed in `children`.
    pub fn spawn<'b, F, B>(&'b mut self, uid: u64, bundle: F) -> Shadow<'b, 'w, 's>
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        if self.find_uid(uid) {
            self.inner(self.children[self.cursor].entity, self.transition_root)
        } else {
            self.insert(uid, bundle())
        }
    }

    /// Insert or update a node. The uid must be unique.
    /// If the entity already exists, it's bundle is only updated if `update` is true.
    /// The children of the node will be updated using the closure passed in `children`.
    pub fn spawn_dyn<'b, F, B>(
        &'b mut self,
        uid: u64,
        update: bool,
        bundle: F,
    ) -> Shadow<'b, 'w, 's>
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        if self.find_uid(uid) {
            if update {
                self.commands
                    .entity(self.children[self.cursor].entity)
                    .insert(bundle());
            }
            self.inner(self.children[self.cursor].entity, self.transition_root)
        } else {
            self.insert(uid, bundle())
        }
    }

    /// Attempt to find a child with the queried uid. Returns `true` if it was found.
    /// After calling this function, `self.cursor` is set to the new position for the queried uid.
    fn find_uid(&mut self, uid: u64) -> bool {
        let mut i = self.cursor;
        for j in self.count..self.len {
            if self.children[i].uid == uid {
                // in-between children are considered to have disappeared
                self.remove(j - self.count);
                // uid has been found, and it's now at self.cursor.
                return true;
            } else {
                i += self.children[i].size;
            }
        }
        // uid not found, this child is considered to have newly appeared.
        false
    }

    fn remove(&mut self, count: usize) {
        if count > 0 {
            let mut i = self.cursor;
            for _ in 0..count {
                self.transition
                    .remove(self.commands.entity(self.children[i].entity));
                i += self.children[i].size;
            }
            self.children.drain(self.cursor..i);
            self.len -= count;
            self.changed = true;
        }
    }

    /// Spawn and insert a new entity at `self.cursor`.
    fn insert<'b, B>(&'b mut self, uid: u64, bundle: B) -> Shadow<'b, 'w, 's>
    where
        B: Bundle,
    {
        let mut entity = self.commands.spawn(bundle);
        self.transition.insert(&mut entity, self.transition_root);
        self.children
            .insert(self.cursor, Child::new(uid, entity.id()));
        self.len += 1;
        self.changed = true;
        self.inner(self.children[self.cursor].entity, false)
    }

    fn inner<'b>(&'b mut self, parent: Entity, root: bool) -> Shadow<'b, 'w, 's> {
        self.count += 1;

        Shadow {
            begin: self.cursor + 1,
            cursor: self.cursor + 1,
            len: self.children[self.cursor].count,
            count: 0,
            changed: false,

            parent: Some(parent),
            commands: self.commands,
            children: self.children,
            parent_cursor: &mut self.cursor,

            transition: self.transition,
            transition_root: root,
        }
    }
}

impl<'a, 'w, 's> Drop for Shadow<'a, 'w, 's> {
    fn drop(&mut self) {
        self.remove(self.len - self.count);

        if self.changed {
            let size = self.cursor - *self.parent_cursor;
            self.children[*self.parent_cursor].size = size;
            self.children[*self.parent_cursor].count = self.count;

            if let Some(parent) = self.parent {
                let mut i = self.begin;
                let mut children = smallvec![];
                for _ in 0..self.count {
                    children.push(self.children[i].entity);
                    i += self.children[i].size;
                }
                self.commands
                    .add(InsertChildrenInOrder { parent, children });
            }
        }

        *self.parent_cursor = self.cursor;
    }
}

impl Child {
    fn new(uid: u64, entity: Entity) -> Self {
        Self {
            entity,
            uid,
            size: 1,
            count: 0,
        }
    }
}

impl Command for InsertChildrenInOrder {
    fn apply(self, world: &mut World) {
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

impl Container {
    fn len(&self) -> usize {
        self.0[0].count
    }

    fn get(&mut self) -> &mut Vec<Child> {
        &mut self.0
    }
}

impl FromWorld for Container {
    fn from_world(_: &mut World) -> Self {
        Container(vec![Child {
            count: 0,
            size: 1,
            entity: Entity::PLACEHOLDER,
            uid: 0,
        }])
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
