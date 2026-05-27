#![allow(dead_code)]

pub struct BoxedArray;

impl BoxedArray {
    pub fn from_iter<T, I: IntoIterator<Item = T>, const N: usize>(iter: I) -> Box<[T; N]> {
        let mut data = Vec::with_capacity(N);
        let mut iter = iter.into_iter();

        for _ in 0..N {
            data.push(iter.next().expect("iterator did not yield enough elements"));
        }

        // SAFETY: We know the length is N, and we have filled it with N elements.
        unsafe { data.into_boxed_slice().try_into().unwrap_unchecked() }
    }

    pub fn from_default<T: Default, const N: usize>() -> Box<[T; N]> {
        let mut data = Vec::with_capacity(N);

        for _ in 0..N {
            data.push(T::default());
        }

        // SAFETY: We know the length is N, and we have filled it with N elements.
        unsafe { data.into_boxed_slice().try_into().unwrap_unchecked() }
    }

    pub fn from_slice<T: Clone, const N: usize>(slice: &[T; N]) -> Box<[T; N]> {
        let mut data = Vec::with_capacity(N);

        for item in slice.iter() {
            data.push(item.clone());
        }

        // SAFETY: We know the length is N, and we have filled it with N elements.
        unsafe { data.into_boxed_slice().try_into().unwrap_unchecked() }
    }

    /// Safely allocate a large array directly on the heap, initialized with zeros.
    /// T MUST be Zeroable, this is not enforced by the compiler!
    pub fn from_zeroed<T, const N: usize>() -> Box<[T; N]> {
        unsafe {
            let mut uninit = Box::<[T; N]>::new_uninit();
            uninit.as_mut_ptr().write_bytes(0, 1);
            uninit.assume_init()
        }
    }
}
