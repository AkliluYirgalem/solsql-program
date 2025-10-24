use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{DataFields, TableMetadata};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum SOLSQLInstruction {
    CreateTable(TableMetadata),
    InsertIntoTable(DataFields),
    UpdateTable(DataFields),
    DeleteRow,
}

impl SOLSQLInstruction {
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        SOLSQLInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}
