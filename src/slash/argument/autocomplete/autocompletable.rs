use crate::{serenity_prelude as serenity, SlashArgError};
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

/// Types that can be marked autocompletable in a slash command parameter.
///
/// Includes almost all types that can be used as a slash command parameter in general,
/// except some built-in model types (User, Member, Role...)
pub trait Autocompletable {
    /// Type of the partial input. This should be `Self` except in cases where a partial input
    /// cannot be parsed into `Self` (e.g. an IP address)
    type Partial;

    /// Try extracting the partial input from the JSON value
    ///
    /// Equivalent to [`crate::SlashArgument::extract`]
    fn extract_partial(value: &serde_json::Value) -> Result<Self::Partial, SlashArgError>;

    /// Serialize an autocompletion choice as a JSON value.
    ///
    /// This is the counterpart to [`Self::extract_partial`]
    fn into_json(self) -> serde_json::Value;
}

#[doc(hidden)]
pub trait AutocompletableHack<T> {
    type Partial;

    fn extract_partial(self, value: &serde_json::Value) -> Result<Self::Partial, SlashArgError>;

    fn into_json(self, value: T) -> serde_json::Value;
}

/// Handles arbitrary types that can be parsed from string.
#[async_trait::async_trait]
impl<T> AutocompletableHack<T> for PhantomData<T>
where
    T: serenity::ArgumentConvert + ToString + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    type Partial = String;

    fn extract_partial(self, value: &serde_json::Value) -> Result<String, SlashArgError> {
        let string = value
            .as_str()
            .ok_or(SlashArgError::CommandStructureMismatch("expected string"))?;
        Ok(string.to_owned())
    }

    fn into_json(self, value: T) -> serde_json::Value {
        serde_json::Value::String(value.to_string())
    }
}

// Handles all integers, signed and unsigned.
#[async_trait::async_trait]
impl<T: TryFrom<i64> + Into<serde_json::Number> + Send + Sync> AutocompletableHack<T>
    for &PhantomData<T>
{
    type Partial = T;

    fn extract_partial(self, value: &serde_json::Value) -> Result<T, SlashArgError> {
        value
            .as_i64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected integer"))?
            .try_into()
            .map_err(|_| SlashArgError::IntegerOutOfBounds)
    }

    fn into_json(self, value: T) -> serde_json::Value {
        serde_json::Value::Number(value.into())
    }
}

#[async_trait::async_trait]
impl AutocompletableHack<f32> for &&PhantomData<f32> {
    type Partial = f32;

    fn extract_partial(self, value: &serde_json::Value) -> Result<f32, SlashArgError> {
        Ok(value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))? as f32)
    }

    fn into_json(self, value: f32) -> serde_json::Value {
        serde_json::Value::Number(
            serde_json::Number::from_f64(value as _).unwrap_or_else(|| serde_json::Number::from(0)),
        )
    }
}

#[async_trait::async_trait]
impl AutocompletableHack<f64> for &&PhantomData<f64> {
    type Partial = f64;

    fn extract_partial(self, value: &serde_json::Value) -> Result<f64, SlashArgError> {
        value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))
    }

    fn into_json(self, value: f64) -> serde_json::Value {
        serde_json::Value::Number(
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0)),
        )
    }
}

#[async_trait::async_trait]
impl<T: Autocompletable> AutocompletableHack<T> for &&PhantomData<T> {
    type Partial = T::Partial;

    fn extract_partial(self, value: &serde_json::Value) -> Result<T::Partial, SlashArgError> {
        <T as Autocompletable>::extract_partial(value)
    }

    fn into_json(self, value: T) -> serde_json::Value {
        value.into_json()
    }
}
