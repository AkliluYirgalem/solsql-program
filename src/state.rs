use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TableMetadata {
    // who can write to the table
    pub authority: Pubkey,
    // name of the table --> authority + table_name = table's PDA
    pub table_name: String,
    // how many columns does the table have
    pub num_of_columns: u8,
    // the last available row id
    pub last_available_row_id: u32,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RowMetadata {
    //the row id
    pub row_id: u32,
    //a vector of pubkey for each data field, order matter
    pub data_field_addresses: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DataFields {
    //there is an address that points to to its belonging row,
    //but it doesnt show-up here because we dont have to pass too many ix data
    //represing the actual data's as String
    pub flatten_data_fields: Vec<String>,
}
