//! Error types for the ATS SDK.

use thiserror::Error;

use crate::model::MAX_CREATORS;

/// All errors that can occur in the ATS SDK.
#[derive(Debug, Error)]
pub enum AtsError {
    /// The work title is empty.
    #[error("title cannot be empty")]
    EmptyTitle,

    /// No creators were provided.
    #[error("no creators provided")]
    NoCreators,

    /// More than [`MAX_CREATORS`] creators were provided.
    #[error("too many creators: {count} (max {MAX_CREATORS})")]
    TooManyCreators {
        /// Number of creators provided.
        count: usize,
    },

    /// A creator's `full_name` field is empty.
    #[error("creator[{index}]: full_name cannot be empty")]
    CreatorEmptyName {
        /// Index of the offending creator.
        index: usize,
    },

    /// A creator's `email` field is empty.
    #[error("creator[{index}]: email cannot be empty")]
    CreatorEmptyEmail {
        /// Index of the offending creator.
        index: usize,
    },

    /// A creator has no roles assigned.
    #[error("creator[{index}]: at least one role is required")]
    CreatorNoRoles {
        /// Index of the offending creator.
        index: usize,
    },

    /// A creator's IPI code is invalid (wrong length or non-digit characters).
    #[error("creator[{index}]: invalid IPI — {reason}")]
    CreatorInvalidIpi {
        /// Index of the offending creator.
        index: usize,
        /// Description of the validation failure.
        reason: String,
    },

    /// A creator's ISNI code is invalid (wrong length or invalid characters).
    #[error("creator[{index}]: invalid ISNI — {reason}")]
    CreatorInvalidIsni {
        /// Index of the offending creator.
        index: usize,
        /// Description of the validation failure.
        reason: String,
    },

    /// The creator index is out of bounds for the input's creator list.
    #[error("creator index {index} out of bounds (total creators: {total})")]
    CreatorIndexOutOfBounds {
        /// Requested index.
        index: usize,
        /// Actual number of creators.
        total: usize,
    },

    /// The media data is empty (zero bytes).
    #[error("media data cannot be empty")]
    EmptyMedia,
}
