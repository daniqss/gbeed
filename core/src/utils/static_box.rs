use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// # Static Box
/// Container for values that allows to store them in a single type
#[derive(Debug)]
pub struct StaticBox<T: ?Sized, const N: usize = 1> {
    data: [u32; N],
    vtable: *const (),
    _phantom: PhantomData<*const T>,
}

// Constructor solo para T sized
impl<T: Copy + 'static, const N: usize> StaticBox<T, N> {
    pub fn new(val: T) -> Self {
        const {
            assert!(
                core::mem::size_of::<T>() <= N * 4,
                "value too big to fit in StaticBox"
            );
        }
        const {
            assert!(
                core::mem::align_of::<T>() <= 4,
                "alignment too large to fit in StaticBox"
            );
        }

        let mut data = [0u32; N];
        unsafe { core::ptr::write(data.as_mut_ptr() as *mut T, val) };

        // we get the vtable pointer when it gets converted to a trait object
        Self {
            data,
            vtable: core::ptr::null(),
            _phantom: PhantomData,
        }
    }
}

impl<T: ?Sized, const N: usize> StaticBox<T, N> {
    /// Constructor used in From macro to create a StaticBox from a value of type V that implements the trait T
    #[doc(hidden)]
    pub(crate) unsafe fn __from_raw_parts(data: [u32; N], vtable: *const ()) -> Self {
        Self {
            data,
            vtable,
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub(crate) fn as_data_ptr(&self) -> *const () { self.data.as_ptr() as *const () }

    #[doc(hidden)]
    pub(crate) fn into_data(self) -> [u32; N] { self.data }
}

impl<T: ?Sized, const N: usize> Deref for StaticBox<T, N> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            let ptr: *const T = core::mem::transmute_copy(&(self.data.as_ptr() as *const (), self.vtable));
            &*ptr
        }
    }
}

impl<T: ?Sized, const N: usize> DerefMut for StaticBox<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            let ptr: *mut T = core::mem::transmute_copy(&(self.data.as_mut_ptr() as *mut (), self.vtable));
            &mut *ptr
        }
    }
}

impl<T: ?Sized, const N: usize> Clone for StaticBox<T, N> {
    fn clone(&self) -> Self { *self }
}
impl<T: ?Sized, const N: usize> Copy for StaticBox<T, N> {}

#[macro_export]
macro_rules! impl_static_box {
    // from implementation for a specific trait
    ($trait:path) => {
        impl<V: $trait + Copy + 'static, const N: usize> From<$crate::utils::StaticBox<V, N>>
            for $crate::utils::StaticBox<dyn $trait, N>
        {
            fn from(sb: $crate::utils::StaticBox<V, N>) -> Self {
                unsafe {
                    let val_ref: &V = &*(sb.as_data_ptr() as *const V);
                    let (_, vtable): (*const (), *const ()) =
                        core::mem::transmute::<&dyn $trait, _>(val_ref as &dyn $trait);
                    $crate::utils::StaticBox::__from_raw_parts(sb.into_data(), vtable)
                }
            }
        }
    };

    // debug implementation for StaticBox
    () => {
        impl<T: ?Sized, const N: usize> core::fmt::Debug for StaticBox<T, N> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "StaticBox {{ data: {:?} }}", self.data)
            }
        }
    };
}
