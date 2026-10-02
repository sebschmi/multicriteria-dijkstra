#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Backtracking is not supported by the prune list.")]
    BacktrackingNotSupported,
}
