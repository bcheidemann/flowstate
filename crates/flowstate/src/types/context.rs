use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
    marker::PhantomData,
};

#[derive(Debug)]
pub struct TypedKey<T: ?Sized> {
    inner: RawKey,
    _type: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawKey {
    key: String,
    type_id: TypeId,
}

impl<T: 'static> TypedKey<T> {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            inner: RawKey {
                key: key.into(),
                type_id: TypeId::of::<T>(),
            },
            _type: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct Context {
    values: BTreeMap<RawKey, Box<dyn Any>>,
}

impl Context {
    pub fn keys(&self) -> impl Iterator<Item = &RawKey> {
        self.values.keys()
    }

    pub fn get<T: 'static>(&self, key: &TypedKey<T>) -> Option<&T> {
        self.values
            .get(&key.inner)
            .and_then(|value| value.downcast_ref())
    }

    pub fn insert<T: 'static>(&mut self, key: TypedKey<T>, value: T) -> Option<T> {
        self.values
            .insert(key.inner, Box::new(value))
            .map(downcast_and_deref)
    }

    pub fn remove<T: 'static>(&mut self, key: &TypedKey<T>) -> Option<T> {
        self.values.remove(&key.inner).map(downcast_and_deref)
    }
}

#[cfg(feature = "async")]
#[derive(Default)]
pub struct AsyncContext {
    values: BTreeMap<RawKey, Box<dyn Any + Send + Sync>>,
}

#[cfg(feature = "async")]
impl AsyncContext {
    pub fn get<T: Send + Sync + 'static>(&self, key: &TypedKey<T>) -> Option<&T> {
        self.values
            .get(&key.inner)
            .and_then(|value| value.downcast_ref())
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, key: TypedKey<T>, value: T) -> Option<T> {
        self.values
            .insert(key.inner, Box::new(value))
            .map(|value| downcast_and_deref(value))
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self, key: &TypedKey<T>) -> Option<T> {
        self.values
            .remove(&key.inner)
            .map(|value| downcast_and_deref(value))
    }
}

fn downcast_and_deref<T: 'static>(value: Box<dyn Any>) -> T {
    *value.downcast::<T>().expect("invalid downcast")
}
