use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Attributes {
    pub(crate) unallocated: u8,
    pub(crate) strength: u8,
    pub(crate) agility: u8,
    pub(crate) constitution: u8
}