use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Instance<T: ?Sized>(pub NonNull<T>);

impl<T: ?Sized> Copy for Instance<T> {}

impl<T: ?Sized> Clone for Instance<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T: ?Sized + Send> Send for Instance<T> {}
unsafe impl<T: ?Sized + Sync> Sync for Instance<T> {}

impl<T: ?Sized> Instance<T> {
    #[inline]
    pub fn new(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    #[inline]
    pub fn new_unchecked(ptr: *mut T) -> Self {
        unsafe {
            Self(NonNull::new_unchecked(ptr))
        }
    }

    #[inline]
    pub fn from_mut(reference: &mut T) -> Self {
        Self(NonNull::from(reference))
    }

    #[inline]
    pub fn from_ref(reference: &T) -> Self {
        Self(NonNull::from(reference))
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub fn as_ref<'a>(&self) -> &'a T {
        unsafe {
            self.0.as_ref()
        }
    }

    #[inline]
    pub fn as_mut<'a>(&self) -> &'a mut T {
        unsafe {
            &mut *self.0.as_ptr()
        }
    }
}