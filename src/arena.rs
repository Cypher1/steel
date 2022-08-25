pub type ID=usize;

#[derive(Debug)]
pub struct Arena<T> {
    members: Vec<Item<T>>,
}

#[derive(Debug, PartialEq)]
pub enum ArenaError {
    IndexEmpty(ID),
    IndexOutOfBounds(ID, ID),
}
use ArenaError::*;

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

#[derive(Debug)]
pub enum Item<T> {
    Tombstone, // TODO: Add an index to the 'next' live entry (update lazily).
    Entry(T),
}

use Item::*;

pub struct ArenaIterator<'a, T> {
    arena: &'a Arena<T>,
    index: ID,
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.arena.members.len() {
            if let Entry(value) = &self.arena.members[self.index] {
                self.index += 1;
                return Some(value);
            }
            self.index+=1;
        }
        return None
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a T;
    type IntoIter = ArenaIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArenaIterator {
            arena: self,
            index: 0,
        }
    }
}

pub struct ArenaIteratorMut<'a, T> {
    arena: &'a mut Arena<T>,
    index: ID,
}

impl<'a, T> Iterator for ArenaIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.arena.members.len();
        while self.index < len {
            if let Entry(value) = &mut self.arena.members[self.index] {
                let ptr: *mut T = value;
                self.index += 1;
                unsafe {
                    // This is safe because the iterator cannot outlive the Arena.
                    return Some(&mut *ptr);
                }
            }
            self.index+=1;
        }
        return None
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = &'a mut T;
    type IntoIter = ArenaIteratorMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArenaIteratorMut {
            arena: self,
            index: 0,
        }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    pub fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID {
        let id = self.members.len();
        self.members.push(Entry(value(id)));
        id
    }

    pub fn add(&mut self, value: T) -> ID {
        self.add_with_id(|_id| value)
    }
    pub fn remove(&mut self, id: ID) -> Result<T, ArenaError> {
        if id >= self.members.len() {
            return Err(IndexOutOfBounds(id, self.members.len()));
        }
        let mut value = Tombstone;
        std::mem::swap(&mut self.members[id], &mut value);
        if let Entry(value) = value {
            return Ok(value);
        }
        Err(IndexEmpty(id))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn arena_holds_items() {
        let mut a: Arena<i32> = Arena::new();
        a.add(1);
        a.add(2);
        a.add(3);
        for x in &a {
            dbg!(x);
        }
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![1,2,3]);
    }

    #[test]
    fn arena_items_can_be_removed() {
        let mut a: Arena<i32> = Arena::new();
        a.add(1);
        let id_to_remove = a.add(2);
        a.add(3);

        let value = a.remove(id_to_remove);
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![1,3]);
        assert_eq!(value, Ok(2));
    }

    #[test]
    fn arena_items_can_be_modified() {
        let mut a: Arena<i32> = Arena::new();
        a.add(1);
        a.add(2);
        a.add(3);

        for x in &mut a {
            *x += 1;
        }
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![2,3,4]);
    }
}
