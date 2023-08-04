use thiserror::Error;

// TODO: Branch out the errors more <01-08-23>
// TODO: When finished zotero storage model, add more specifics in the Display implementations <01-08-23>

#[derive(Error, Debug)]
pub enum ZoteroError {
    #[error("required parameter `{required}` not passed")]
    ParamNotPassed { required: String },

    #[error("`{call}` is not a valid API call")]
    InvalidAPICall { call: String },

    #[error("`{unsupported:?}` are not supported in the API call `{call}`")]
    UnsupportedParams {
        call: String,
        unsupported: Vec<String>,
    },

    #[error("user `{user}` is not authorized to perform this action")]
    NotAuthorized { user: String },

    #[error("too many items passed to a write method")]
    TooManyItems,

    #[error("user ID or user key not provided")]
    MissingCredentials,

    #[error("create/update items w/invalid fields `{invalid:?}`")]
    InvalidItemFields { invalid: Vec<String> },

    #[error("resource `{resource}` not found")]
    ResourceNotFound { resource: String },

    #[error("HTTP error: `{0}`")]
    HTTPError(u32),

    #[error("library is locked")]
    Conflict, // 409

    #[error("X-Zotero-Write-Token has already been submitted")]
    PreConditionFailed, // 412

    #[error("upload would exceed the storage quota of library owner")]
    RequestEntityTooLarge, // 413

    #[error("If-Match or If-None-Match was not provided")]
    PreConditionRequired, // 428

    #[error("too many unfinished uploads")]
    TooManyRequests, // 429

    // TODO: Specify to does not exist and cannot be opened <01-08-23>
    #[error("file `{file}` does not exist")]
    FileDoesNotExist { file: String },

    #[error("backoff period for new requests exceeds 32s")]
    TooManyRetries,

    // TODO: Specify to type of error or error code (non-HTTP) <01-08-23>
    #[error("connection dropped")]
    UploadError,
}
