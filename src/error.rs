use custom_error::custom_error;

custom_error! {
/// All possible ways how caretaking may fail.
pub Error
    /// Error of the underlying watch mechanism
    Notify{
        /// Source of the error.
        source: notify::Error
    } = "Notify error: {source}",
    /// Error of the underlying watch mechanism
    PathWatch{
        /// The path being watched
        path: String,
        /// Source of the error.
        source: notify::Error
    } = "Notify error watching {path}: {source}",
    /// The provided glob path was not valid.
    Pattern{
        /// Source of the error.
        source: glob::PatternError
    } = "Invalid glob: {source}",
    /// Input / output error
    Io{
        /// Source of the error.
        source: std::io::Error
    } = "Input / output error: {source}",
    /// Channel receive error
    Receive{
        /// Source of the error.
        source: crossbeam_channel::RecvError
    } = "Channel receive error: {source}",
}
