// TODO: Use https://docs.rs/crate/generational-arena/0.2.8

pub type Index = usize;

#[derive(Clone, Debug)]
pub struct Arena<T> {
    members: Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArenaError {
    IndexEmpty(String, Index),
    IndexOutOfBounds(Index, Index),
}
use ArenaError::*;

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

pub struct ArenaIterator<'a, T> {
    arena: &'a Arena<T>,
    index: Index,
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.arena.members.len();
        if self.index < len {
            let value = &self.arena.members[self.index];
            self.index += 1;
            return Some(value);
        }
        None
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
    index: Index,
}

impl<'a, T> Iterator for ArenaIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.arena.members.len();
        if self.index < len {
            let value = &mut self.arena.members[self.index];
            let ptr: *mut T = value;
            self.index += 1;
            unsafe {
                // This is safe because the iterator cannot outlive the Arena.
                return Some(&mut *ptr);
            }
        }
        None
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
        let mut members = Vec::new();
        members.reserve(1000);
        Self { members }
    }

    pub fn capacity(&self) -> usize {
        self.members.len()
    }

    pub fn active_mem_usage(&self) -> usize {
        self.members.len() * std::mem::size_of::<T>()
    }

    pub fn mem_usage(&self) -> usize {
        self.members.capacity() * std::mem::size_of::<T>()
    }

    pub fn add_with_id<S: Into<T>, F: FnOnce(Index) -> S>(&mut self, value: F) -> Index {
        let id = self.members.len();
        self.members.push(value(id).into());
        id
    }

    pub fn add<S: Into<T>>(&mut self, value: S) -> Index {
        self.add_with_id(|_id| value)
    }

    pub fn set(&mut self, id: Index, value: T) -> Result<(), ArenaError> {
        if id >= self.members.len() {
            return Err(IndexOutOfBounds(id, self.members.len()));
        }
        self.members[id] = value;
        Ok(())
    }

    pub fn get(&self, id: Index) -> Result<&T, ArenaError> {
        if id >= self.members.len() {
            return Err(IndexOutOfBounds(id, self.members.len()));
        }
        Ok(&self.members[id])
    }

    pub fn get_mut(&mut self, id: Index) -> Result<&mut T, ArenaError> {
        if id >= self.members.len() {
            return Err(IndexOutOfBounds(id, self.members.len()));
        }
        Ok(&mut self.members[id])
    }

    pub fn remove_by_swap(&mut self, id: Index) -> Result<T, ArenaError> {
        let len = self.members.len();
        if id >= len {
            return Err(IndexOutOfBounds(id, len));
        }
        let mut value = self.members.pop().unwrap(); // There is always at least one member.
        if id < len - 1 {
            // swap if the id wasn't the last one...
            std::mem::swap(&mut self.members[id], &mut value);
        }
        Ok(value) // Return the removed item. The 'user' must swap `id` and `len-1`
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
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![1, 2, 3]);
    }

    #[test]
    fn arena_items_can_be_removed() {
        let mut a: Arena<i32> = Arena::new();
        a.add(1);
        let id_to_remove = a.add(2);
        a.add(3);

        let value = a.remove_by_swap(id_to_remove);
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![1, 3]);
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
        assert_eq!(a.into_iter().cloned().collect::<Vec<i32>>(), vec![2, 3, 4]);
    }
}
