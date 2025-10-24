use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum SOLSQLError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Unauthorized")]
    Unauthorized,
}

impl From<SOLSQLError> for ProgramError {
    fn from(e: SOLSQLError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
