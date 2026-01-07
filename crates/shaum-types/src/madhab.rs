//! Madhab and DaudStrategy enums.

use serde::{Serialize, Deserialize};

/// Sunni schools of jurisprudence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Madhab {
    Shafi,
    Hanafi,
    Maliki,
    Hanbali,
}

impl Default for Madhab {
    fn default() -> Self { Self::Shafi }
}

/// Strategy for Daud fasting on Haram days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DaudStrategy {
    /// Skip turn, lose the fast.
    Skip,
    /// Postpone to next permissible day.
    Postpone,
}

impl Default for DaudStrategy {
    fn default() -> Self { Self::Skip }
}
