use std::{marker::PhantomData, mem, ops::Deref, sync::Arc};

pub trait HolderRef {
    type HolderType;

    fn set_holder(&self, holder_ref: &'static Self::HolderType);
}

pub struct WithHolderRef<'a, T> {
    object: Arc<T>,
    holder: PhantomData<&'a ()>,
}

impl<'a, T, U> WithHolderRef<'a, T>
where
    T: HolderRef<HolderType = U> + Send + Sync + 'static,
    U: 'static,
{
    /// # Safety
    ///
    /// Whenever holder passes T to external code (returning it from a function,
    /// passing it as a paremeter etc.), it must make sure to do so using
    /// `WithHolderRef<T>`.
    pub unsafe fn new(holder: &'a U, object: Arc<T>) -> WithHolderRef<'a, T> {
        let holder: &'static U = unsafe { mem::transmute(holder) };
        object.set_holder(holder);

        Self {
            object,
            holder: PhantomData,
        }
    }
}

impl<'a, T> Deref for WithHolderRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref()
    }
}
