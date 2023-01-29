use std::{mem, ops::Deref, sync::Arc};

pub trait HolderRef {
    type HolderType;

    fn set_holder(&self, holder_ref: Option<&'static Self::HolderType>);
}

pub struct WithHolderRef<'a, T> {
    object: Arc<T>,
    drop_fun: Option<Box<dyn FnOnce() + Send + 'a>>,
}

impl<'a, T, U> WithHolderRef<'a, T>
where
    T: HolderRef<HolderType = U> + Send + Sync + 'static,
    U: 'static,
{
    pub fn new(holder: &'a U, object: Arc<T>) -> WithHolderRef<'a, T> {
        let holder: &'static U = unsafe { mem::transmute(holder) };
        object.set_holder(Some(holder));

        let drop_fun = {
            let object = object.clone();
            move || {
                object.set_holder(None);
            }
        };

        Self {
            object,
            drop_fun: Some(Box::new(drop_fun)),
        }
    }
}

impl<'a, T> Deref for WithHolderRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref()
    }
}

impl<'a, T> Drop for WithHolderRef<'a, T> {
    fn drop(&mut self) {
        if let Some(drop_fun) = self.drop_fun.take() {
            drop_fun();
        }
    }
}
