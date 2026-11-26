// These config values are plain JSON strings. Keep the wire names and legacy
// aliases in one place for settings, relay discovery and existing CLI callers.
macro_rules! string_enum {
    ($(#[$attr:meta])* pub enum $name:ident { $( $(#[$variant_attr:meta])* $variant:ident => $wire:literal $(| $alias:literal)* ),+ $(,)? }) => {
        $(#[$attr])*
        pub enum $name {
            $( $(#[$variant_attr])* $variant ),+
        }

        impl $name {
            fn from_wire(value: &str) -> Option<Self> {
                match value { $( $wire $(| $alias)* => Some(Self::$variant), )+ _ => None }
            }
        }

        impl nextjson::NsonSchema for $name {
            const SCHEMA: nextjson::TypeSchema = nextjson::TypeSchema::Str;
        }
        impl nextjson::NsonSerialize for $name {
            fn nextencode<E: nextjson::FormatEncoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
                encoder.write_str(match self { $( Self::$variant => $wire, )+ })
            }
        }
        impl<'de> nextjson::NsonDeserialize<'de> for $name {
            fn nextdecode_into<D: nextjson::FormatDecoder<'de>>(decoder: &mut D, slot: &mut nextjson::DecodeSlot<Self>) -> Result<(), D::Error> {
                let value = decoder.string()?;
                let parsed = Self::from_wire(&value).ok_or_else(|| nextjson::Error::custom(concat!("invalid ", stringify!($name))))?;
                slot.write(parsed);
                Ok(())
            }
        }
    };
}

pub mod agents;
mod anthropic_auth;
mod bedrock;
pub mod catalog;
pub mod config;
pub mod gateway;
pub mod json;
mod media;
pub mod memory;
mod model_name;
mod opencode;
pub mod peer;
pub mod project_instructions;
pub mod service;
mod translate;
mod upstream;
mod vertex;
pub mod webchat;

pub const APP_ID: &str = "com.blue.crabforge";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
