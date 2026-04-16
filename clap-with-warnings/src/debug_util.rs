use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use quote::ToTokens;

pub struct DisplayDebugMultiple<S: AsRef<[T]>, T: ToTokens> {
    values: S,
    _phantom: PhantomData<T>,
}

impl<S: AsRef<[T]>, T: ToTokens> From<S> for DisplayDebugMultiple<S, T> {
    fn from(values: S) -> Self {
        Self {
            values,
            _phantom: PhantomData,
        }
    }
}

impl<S: AsRef<[T]>, T: ToTokens> Display for DisplayDebugMultiple<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self
            .values
            .as_ref()
            .iter()
            .map(|d| d.to_token_stream().to_string())
            .collect::<Vec<_>>();
        write!(f, "{v:?}")
    }
}

impl<S: AsRef<[T]>, T: ToTokens> Debug for DisplayDebugMultiple<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self
            .values
            .as_ref()
            .iter()
            .map(|d| d.to_token_stream().to_string())
            .collect::<Vec<_>>();
        write!(f, "{v:?}")
    }
}
