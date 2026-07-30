pub(crate) type InstancePointer<T> = *mut T; 

#[derive(Debug)]
pub(crate) struct Instance<T>(pub InstancePointer<T>);

unsafe impl <T>Send for Instance<T> {}
unsafe impl <T>Sync for Instance<T> {}

impl <T>Instance<T> {
    pub fn as_mut(&self) -> &mut T {
        return unsafe { &mut *self.0 };
    }

    pub fn as_ref(&self) -> &T {
        return unsafe { & *self.0 };
    }
}

impl <T>Clone for Instance<T> {
    fn clone(&self) -> Self {
        return Self(self.0.clone());
    }
}