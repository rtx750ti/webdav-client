use serde::{Deserialize, Serialize};

pub trait FormatXml {
    fn format_to_friendly<'de, T>() -> T
    where
        T: Deserialize<'de> + Serialize;
}
