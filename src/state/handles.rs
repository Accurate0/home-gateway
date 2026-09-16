use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

pub trait RequiredHandle: Any + Send + Sync {
    const LABEL: &'static str;
}

#[macro_export]
macro_rules! required_handles {
    ($($handle:ty => $label:literal),+ $(,)?) => {
        $(
            impl $crate::state::RequiredHandle for $handle {
                const LABEL: &'static str = $label;
            }
        )+

        fn assert_required(handles: &$crate::state::HandleRegistry) {
            $(
                if !handles.contains::<$handle>() {
                    panic!(
                        "the {} handle was not registered before startup",
                        <$handle as $crate::state::RequiredHandle>::LABEL
                    );
                }
            )+
        }
    };
}

#[derive(Default)]
pub struct HandleRegistryBuilder {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl HandleRegistryBuilder {
    pub fn insert<T: Any + Send + Sync>(mut self, handle: T) -> Self {
        self.inner.insert(TypeId::of::<T>(), Box::new(handle));

        self
    }

    pub fn insert_optional<T: Any + Send + Sync>(self, handle: Option<T>) -> Self {
        match handle {
            Some(handle) => self.insert(handle),
            None => self,
        }
    }

    pub fn build(self) -> HandleRegistry {
        HandleRegistry {
            inner: Arc::new(self.inner),
        }
    }
}

#[derive(Clone, Default)]
pub struct HandleRegistry {
    inner: Arc<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl HandleRegistry {
    pub fn builder() -> HandleRegistryBuilder {
        HandleRegistryBuilder::default()
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.inner.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    pub fn require<T: RequiredHandle>(&self) -> &T {
        self.get()
            .unwrap_or_else(|| panic!("the {} handle was never registered", T::LABEL))
    }

    pub fn expect<T: Any + Send + Sync>(&self) -> &T {
        self.get()
            .unwrap_or_else(|| panic!("handle {} was never registered", type_name::<T>()))
    }
}
