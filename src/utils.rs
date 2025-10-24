use solana_program::{
    account_info::AccountInfo,
    hash::{hashv, Hash},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_system_interface::instruction;

use crate::ID as PROGRAM_ID;

pub fn get_hashed_seed(seeds: &[&[u8]]) -> Hash {
    hashv(seeds)
}

fn rent_exempt_lamports(account_size: usize) -> Result<u64, ProgramError> {
    let rent = Rent::get()?;
    Ok(rent.minimum_balance(account_size))
}

pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    pda_account: &AccountInfo<'a>,
    seeds: &[&[u8]],
    space: usize,
) -> Result<(), ProgramError> {
    let pda_lamports = **pda_account.lamports.borrow();
    let pda_owner = *pda_account.owner;

    if pda_lamports > 0 {
        if pda_owner != PROGRAM_ID {
            msg!("PDA already exists but is not owned by this program");
            return Err(ProgramError::IllegalOwner);
        }
        msg!("PDA already exists, skipping creation");
        return Ok(());
    }
    let lamports = rent_exempt_lamports(space)?;
    invoke_signed(
        &instruction::create_account(
            payer.key,
            pda_account.key,
            lamports,
            space as u64,
            &PROGRAM_ID,
        ),
        &[payer.clone(), pda_account.clone()],
        &[seeds],
    )?;
    Ok(())
}

pub fn write_to_account(data: &[u8], account: &AccountInfo) -> Result<(), ProgramError> {
    let mut account_data = account.try_borrow_mut_data()?;
    if data.len() > account_data.len() {
        return Err(ProgramError::AccountDataTooSmall);
    }

    account_data[..data.len()].copy_from_slice(data);
    Ok(())
}
