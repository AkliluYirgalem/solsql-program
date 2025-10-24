use crate::instruction::SOLSQLInstruction;
use crate::state::RowMetadata;
use crate::state::TableMetadata;
use crate::utils::{create_pda_account, get_hashed_seed, write_to_account};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_system_interface::instruction;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction_data = SOLSQLInstruction::unpack(instruction_data)?;
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let passed_table_pda = next_account_info(accounts_iter)?;
    let passed_fee_vault_pda = next_account_info(accounts_iter)?;
    let _passed_system_account = next_account_info(accounts_iter)?;

    match instruction_data {
        SOLSQLInstruction::CreateTable(table_metadata) => {
            let table_seed = get_hashed_seed(&[
                table_metadata.authority.as_ref(),
                table_metadata.table_name.as_bytes(),
            ])
            .to_bytes();

            let (expected_table_pda, bump) =
                Pubkey::find_program_address(&[&table_seed], &program_id);
            assert_eq!(
                expected_table_pda, *passed_table_pda.key,
                "Invalid Table Address."
            );
            if **passed_table_pda.lamports.borrow() > 0 {
                return Ok(());
            }

            let table_data = borsh::to_vec(&table_metadata)?;
            create_pda_account(
                payer,
                passed_table_pda,
                &[&table_seed, &[bump]],
                table_data.len(),
            )?;

            write_to_account(&table_data, passed_table_pda)?;
            msg!("Table Created At: {}", passed_table_pda.key);
        }
        SOLSQLInstruction::InsertIntoTable(data_field) => {
            let mut raw_table_data = passed_table_pda.try_borrow_mut_data()?;
            let mut table_metadata = TableMetadata::try_from_slice(&raw_table_data)?;

            let num_of_expected_accounts = 5 + table_metadata.num_of_columns;
            let num_of_passed_accounts = accounts.len() as u8;
            assert_eq!(
                num_of_expected_accounts, num_of_passed_accounts,
                "Incorrect Accounts."
            );

            if !payer.is_signer || table_metadata.authority != *payer.key {
                msg!("Authority must sign the transaction.");
                return Err(ProgramError::IncorrectAuthority);
            }

            let row_seed = get_hashed_seed(&[
                table_metadata.authority.as_ref(),
                table_metadata.table_name.as_bytes(),
                &table_metadata.last_available_row_id.to_le_bytes(),
            ])
            .to_bytes();

            let passed_row_pda = next_account_info(accounts_iter)?;
            let (expected_row_pda, row_bump) =
                Pubkey::find_program_address(&[&row_seed], &program_id);
            assert_eq!(
                expected_row_pda, *passed_row_pda.key,
                "Invalid Row Address."
            );

            let mut row_metadata = RowMetadata {
                row_id: table_metadata.last_available_row_id,
                data_field_addresses: Vec::new(),
            };
            for col in 0..table_metadata.num_of_columns {
                let data_field_seed = get_hashed_seed(&[
                    table_metadata.authority.as_ref(),
                    table_metadata.table_name.as_bytes(),
                    &table_metadata.last_available_row_id.to_le_bytes(),
                    &(col + 1).to_le_bytes(),
                    &data_field.flatten_data_fields[col as usize].as_bytes(),
                ])
                .to_bytes();

                let passed_data_field_pda = next_account_info(accounts_iter)?;
                let (expected_data_field_pda, bump) =
                    Pubkey::find_program_address(&[&data_field_seed], &program_id);

                assert_eq!(
                    expected_data_field_pda, *passed_data_field_pda.key,
                    "Invalid Data Field Address."
                );

                create_pda_account(
                    payer,
                    passed_data_field_pda,
                    &[&data_field_seed, &[bump]],
                    size_of::<Pubkey>() + data_field.flatten_data_fields[col as usize].len(),
                )?;
                let mut row_id_plus_data = Vec::new();
                row_id_plus_data.extend_from_slice(passed_row_pda.key.as_ref());
                row_id_plus_data
                    .extend_from_slice(data_field.flatten_data_fields[col as usize].as_bytes());

                write_to_account(&row_id_plus_data, passed_data_field_pda)?;

                row_metadata
                    .data_field_addresses
                    .push(*passed_data_field_pda.key);
            }

            let row_data = borsh::to_vec(&row_metadata)?;
            create_pda_account(
                payer,
                passed_row_pda,
                &[&row_seed, &[row_bump]],
                row_data.len(),
            )?;
            write_to_account(&row_data, passed_row_pda)?;

            table_metadata.last_available_row_id += 1;
            let serialized = borsh::to_vec(&table_metadata)?;
            raw_table_data[..serialized.len()].copy_from_slice(&serialized);
        }
        SOLSQLInstruction::UpdateTable(update_fields) => {
            let raw_table_data = passed_table_pda.try_borrow_data()?;
            let table_metadata = TableMetadata::try_from_slice(&raw_table_data)?;

            let expected_fee_vault_seed = get_hashed_seed(&["fee_vault".as_bytes()]).to_bytes();
            let (expected_fee_vault_pda, vault_bump) =
                Pubkey::find_program_address(&[&expected_fee_vault_seed], &program_id);

            assert_eq!(
                expected_fee_vault_pda, *passed_fee_vault_pda.key,
                "Invalid Fee Vault Address"
            );
            create_pda_account(
                payer,
                passed_fee_vault_pda,
                &[&expected_fee_vault_seed, &[vault_bump]],
                0,
            )?;

            if !payer.is_signer || table_metadata.authority != *payer.key {
                msg!("Authority must sign the transaction.");
                return Err(ProgramError::IncorrectAuthority);
            }

            let passed_row_pda = next_account_info(accounts_iter)?;

            let mut raw_row_data = passed_row_pda.try_borrow_mut_data()?;
            let mut passed_row = RowMetadata::try_from_slice(&raw_row_data)?;

            let remaining: Vec<&AccountInfo> = accounts_iter.collect();
            let mut new_value: usize = 0;
            for data_field_pda in remaining.chunks(2) {
                let old_data_pda = data_field_pda[0];
                let passed_new_data_pda = data_field_pda[1];

                let mut col_num: Option<u8> = None;
                for (i, pk) in passed_row.data_field_addresses.iter_mut().enumerate() {
                    if *pk == *old_data_pda.key {
                        *pk = *passed_new_data_pda.key;
                        col_num = Some(i as u8);
                        break;
                    }
                }

                let new_data_field_seed = get_hashed_seed(&[
                    table_metadata.authority.as_ref(),
                    table_metadata.table_name.as_bytes(),
                    &passed_row.row_id.to_le_bytes(),
                    &(col_num.unwrap() + 1).to_le_bytes(),
                    &update_fields.flatten_data_fields[new_value].as_bytes(),
                ])
                .to_bytes();

                let (expected_new_data_pda, new_bump) =
                    Pubkey::find_program_address(&[&new_data_field_seed], &program_id);
                assert_eq!(
                    expected_new_data_pda, *passed_new_data_pda.key,
                    "Mismatch New PDA"
                );

                let mut row_id_plus_data = Vec::new();
                row_id_plus_data.extend_from_slice(passed_row_pda.key.as_ref());
                row_id_plus_data
                    .extend_from_slice(update_fields.flatten_data_fields[new_value].as_bytes());

                create_pda_account(
                    payer,
                    passed_new_data_pda,
                    &[&new_data_field_seed, &[new_bump]],
                    row_id_plus_data.len(),
                )?;

                write_to_account(&row_id_plus_data, passed_new_data_pda)?;
                new_value += 1;

                //taking fee
                let old_data_pda_total_lamports = **old_data_pda.lamports.borrow();
                let fee = old_data_pda_total_lamports / 10;
                if old_data_pda.key != passed_new_data_pda.key {
                    let remainder = old_data_pda_total_lamports - fee;
                    **passed_fee_vault_pda.lamports.borrow_mut() += fee;
                    **payer.lamports.borrow_mut() += remainder;
                    **old_data_pda.lamports.borrow_mut() = 0;
                } else {
                    invoke(
                        &instruction::transfer(payer.key, passed_fee_vault_pda.key, fee),
                        &[
                            payer.clone(),
                            passed_fee_vault_pda.clone(),
                            _passed_system_account.clone(),
                        ],
                    )?;
                }
                //taking fee
            }

            let serialized = borsh::to_vec(&passed_row)?;
            raw_row_data[..serialized.len()].copy_from_slice(&serialized);
        }
        SOLSQLInstruction::DeleteRow => {
            let raw_table_data = passed_table_pda.try_borrow_data()?;
            let table_metadata = TableMetadata::try_from_slice(&raw_table_data)?;

            if !payer.is_signer || table_metadata.authority != *payer.key {
                msg!("Authority must sign the transaction.");
                return Err(ProgramError::IncorrectAuthority);
            }

            let expected_fee_vault_seed = get_hashed_seed(&["fee_vault".as_bytes()]).to_bytes();
            let (expected_fee_vault_pda, vault_bump) =
                Pubkey::find_program_address(&[&expected_fee_vault_seed], &program_id);

            assert_eq!(
                expected_fee_vault_pda, *passed_fee_vault_pda.key,
                "Invalid Fee Vault Address"
            );
            create_pda_account(
                payer,
                passed_fee_vault_pda,
                &[&expected_fee_vault_seed, &[vault_bump]],
                0,
            )?;

            let remaining: Vec<&AccountInfo> = accounts_iter.collect();
            for data_field_pda in remaining {
                let total_lamports = **data_field_pda.lamports.borrow();
                let fee = total_lamports / 10; // 10% fee
                let remainder = total_lamports - fee; // 90% to payer

                **passed_fee_vault_pda.lamports.borrow_mut() += fee;
                **payer.lamports.borrow_mut() += remainder;
                **data_field_pda.lamports.borrow_mut() = 0;
            }
        }
    }

    Ok(())
}
