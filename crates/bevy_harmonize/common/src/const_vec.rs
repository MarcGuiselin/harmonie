use const_panic::concat_panic;
use std::{
    fmt::{Debug, Formatter, Result},
    mem::MaybeUninit,
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

    #[track_caller]
    pub const fn push(mut self, item: T) -> Self {
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
        if self.len + vec.len >= capacity {
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
