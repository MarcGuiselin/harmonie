#![feature(const_type_name)]

use const_panic::concat_panic;
use std::{
    fmt::{Debug, Formatter, Result},
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy)]
pub struct ConstVec<T: Copy, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> ConstVec<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    #[track_caller]
    pub const fn push(&mut self, item: T) -> &mut Self {
        let capacity = const { N };
        if self.len >= capacity {
            let type_name = std::any::type_name::<T>();
            concat_panic!(
                "\nReached ConstVec<",
                type_name,
                ", ",
                capacity,
                "> capacity"
            );
        }
        self.buffer[self.len] = MaybeUninit::new(item);
        self.len += 1;
        self
    }

    #[track_caller]
    pub const fn append(mut self, vec: Self) -> Self {
        let capacity = const { N };
        if self.len + vec.len > capacity {
            let type_name = std::any::type_name::<T>();
            concat_panic!(
                "\nCannot append ",
                vec.len,
                " items into a ConstVec<",
                type_name,
                ", ",
                capacity,
                "> with ",
                self.len,
                " items, since that would exceed capacity"
            );
        }

        let mut i = 0;
        while i < vec.len {
            self.buffer[self.len + i] = vec.buffer[i];
            i += 1;
        }
        self.len += vec.len;
        self
    }

    #[track_caller]
    pub const fn from_slice(slice: &[T]) -> Self {
        let mut vec = Self::new();
        let capacity = const { N };

        if slice.len() > capacity {
            let type_name = std::any::type_name::<T>();
            concat_panic!(
                "\nSlice length exceeds ConstVec<",
                type_name,
                ", ",
                capacity,
                "> capacity"
            );
        }

        let mut i = 0;
        while i < slice.len() {
            vec.buffer[i] = MaybeUninit::new(slice[i]);
            i += 1;
        }
        vec.len = slice.len();
        vec
    }

    pub fn into_vec(&self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len);
        for i in 0..self.len {
            // SAFETY: We are certain that all items in the buffer up to length are initialized
            vec.push(unsafe { self.buffer[i].assume_init() });
        }
        vec
    }

    pub fn into_slice(&self) -> &[T] {
        // SAFETY: We are certain that all items in the buffer up to length are initialized
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr() as *const T, self.len) }
    }
}

impl<T: Copy + Debug, const N: usize> Debug for ConstVec<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let entries: Vec<_> = self.buffer[0..self.len]
            .iter()
            // SAFETY: We are certain that all items in the buffer up to length are initialized
            .map(|system| unsafe { system.assume_init() })
            .collect();
        f.debug_list().entries(entries).finish()
    }
}

impl<T: Copy, const N: usize> Index<usize> for ConstVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("Index out of bounds");
        }
        // SAFETY: We are certain that all items in the buffer up to length are initialized
        unsafe { &*self.buffer[index].as_ptr() }
    }
}

impl<T: Copy, const N: usize> IndexMut<usize> for ConstVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("Index out of bounds");
        }
        // SAFETY: We are certain that all items in the buffer up to length are initialized
        unsafe { &mut *self.buffer[index].as_mut_ptr() }
    }
}

impl<T: Copy, const N: usize> Default for ConstVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_slice() {
        let mut vec = ConstVec::<u8, 128>::from_slice(&[1, 2, 3]);
        assert!(vec.len() == 3);

        vec.push(4);
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.into_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn from_slice_panic() {
        let slice = &[1, 2, 3, 4, 5];
        let result = std::panic::catch_unwind(|| {
            ConstVec::<u32, 4>::from_slice(slice);
        });
        assert!(result.is_err());
    }

    #[test]
    fn into_vec() {
        let mut vec = ConstVec::<&'static str, 10>::new();
        vec.push("a");
        vec.push("b");
        vec.push("c");
        assert!(vec.len() == 3);

        assert_eq!(vec.into_vec(), vec!["a", "b", "c"]);
    }

    #[test]
    fn length_panic() {
        let mut vec = ConstVec::<u32, 2>::new();

        vec.push(1);
        vec.push(2);

        let result = std::panic::catch_unwind(|| {
            let mut vec = vec;
            vec.push(3);
        });
        assert!(result.is_err());
    }

    #[test]
    fn append() {
        let vec1 = ConstVec::<u32, 4>::from_slice(&[1, 2]);
        let vec2 = ConstVec::from_slice(&[3, 4]);

        let result = vec1.append(vec2);
        assert_eq!(result.into_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn append_panic() {
        let vec1 = ConstVec::<u32, 4>::from_slice(&[1, 2]);
        let vec2 = ConstVec::from_slice(&[3, 4, 5]);

        let result = std::panic::catch_unwind(|| {
            vec1.append(vec2);
        });
        assert!(result.is_err());
    }

    #[test]
    fn get_index() {
        let vec = ConstVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
    }

    #[test]
    fn get_index_panic() {
        let vec = ConstVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);
        let result = std::panic::catch_unwind(|| {
            let _ = vec[4];
        });
        assert!(result.is_err());
    }

    #[test]
    fn set_index() {
        let mut vec = ConstVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);
        vec[0] = 5;
        vec[1] = 6;
        vec[2] = 7;
        vec[3] = 8;

        assert_eq!(vec.into_slice(), &[5, 6, 7, 8]);
    }

    #[test]
    fn set_index_panic() {
        let vec = ConstVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);
        let result = std::panic::catch_unwind(|| {
            let mut vec = vec;
            vec[4] = 5;
        });
        assert!(result.is_err());
    }
}
