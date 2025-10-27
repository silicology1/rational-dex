use anchor_lang::prelude::*;

// Custom error codes for program
#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Math overflow")]
    Overflow,

    #[msg("Insufficient LP tokens")]
    InsufficientLP,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Commit phase is closed")]
    CommitClosed,

    #[msg("Reveal phase not started")]
    RevealNotStarted,

    #[msg("Reveal phase already closed")]
    RevealClosed,

    #[msg("Vote already revealed")]
    AlreadyRevealed,

    #[msg("Wrong voting stage")]
    WrongStage,

    #[msg("Hash mismatch during reveal")]
    HashMismatch,

    #[msg("No revealed votes found")]
    NoRevealedVotes,

    #[msg("Reveal not finished")]
    RevealNotFinished,

    #[msg("Vote round already finalized")]
    AlreadyFinalized,

    #[msg("Vote round not finalized yet")]
    NotFinalized,

    #[msg("Unauthorized voter")]
    Unauthorized,

    #[msg("No stake available")]
    NoStake,

    #[msg("Vote round mismatch")]
    VoteRoundMismatch,

    #[msg("Failed to deserialize VoteAccount")]
    VoteAccountDeserialize,

    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}

#[error_code]
pub enum DexError {
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Price not set")]
    PriceNotSet,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient offer")]
    InsufficientOffer,
}
