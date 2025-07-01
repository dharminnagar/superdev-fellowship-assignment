use serde::{Deserialize, Serialize};
use solana_sdk::instruction::AccountMeta;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenRequest {
    pub mintAuthority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub program_id: String,
    pub accounts: Vec<AccountMeta>,
    pub instruction_data: String
}

#[derive(Serialize, Deserialize)]
pub struct TokenCreateSuccessResponse {
    pub success: bool,
    pub data: TokenData,
}

#[derive(Serialize, Deserialize)]
pub struct TokenCreateErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenMintRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignMsgRequest {
    pub message: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyMsgRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendSOLRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}