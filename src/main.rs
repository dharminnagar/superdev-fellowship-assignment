pub mod types;

use axum::{
    http::StatusCode, response::{IntoResponse}, routing::{get, post}, Json, Router
};
use base64::{engine::general_purpose, Engine};
use solana_keypair::keypair_from_seed;
use solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer, system_instruction::transfer};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{initialize_mint2, mint_to_checked, transfer_checked};
use spl_token::ID as TOKEN_PROGRAM_ID;

use std::{net::SocketAddr, str::FromStr};
use serde_json::{self, json};

use crate::types::{CreateTokenRequest, SendSOLRequest, SendTokenRequest, SignMsgRequest, TokenCreateErrorResponse, TokenCreateSuccessResponse, TokenData, TokenMintRequest, VerifyMsgRequest};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(token_create))
        .route("/token/mint", post(token_mint))
        .route("/message/sign", post(sign_msg))
        .route("/message/verify", post(verify_msg))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "gm Dharmin!"
}

async fn generate_keypair() -> impl IntoResponse {
    let keypair = solana_sdk::signature::Keypair::new();
    let pub_key = keypair.pubkey();
    let secret_key = keypair.to_base58_string();

    if secret_key.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": "Failed to generate keypair"
            })),
        );
    } else {
        return (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "data": {
                "pubkey": pub_key.to_string(),
                "secret": secret_key
            }
        })));
    }
}

async fn token_create(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    let CreateTokenRequest { mintAuthority, mint, decimals } = payload;

    let mint_pubkey = match Pubkey::from_str(&mint) {
        Ok(key) => key,
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: "Invalid mint public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    
    let mint_authority_pubkey = match Pubkey::from_str(&mintAuthority) {
        Ok(key) => key,
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: "Invalid mint authority public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    
    let initialize_mint_ix = initialize_mint2(
        &TOKEN_PROGRAM_ID,
        &mint_pubkey,
        &mint_authority_pubkey,
        Some(&mint_authority_pubkey),
        decimals,
    );

    match initialize_mint_ix {
        Ok(ix) => {
            let response = TokenCreateSuccessResponse {
                success: true,
                data: TokenData {
                    program_id: ix.program_id.to_string(),
                    accounts: ix.accounts,
                    instruction_data: general_purpose::STANDARD.encode(&ix.data),
                },
            };

            return (StatusCode::OK, Json(response)).into_response()
        },
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: String::from("Failed to create mint instruction"),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    }
    
    
}

async fn token_mint(Json(payload): Json<TokenMintRequest>) -> impl IntoResponse {
    let TokenMintRequest { mint, destination, authority, amount } = payload;

    let mint_pubkey = match Pubkey::from_str(&mint) {
        Ok(key) => key,
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: "Invalid mint public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };

    let destination_pubkey = match Pubkey::from_str(&destination) {
        Ok(key) => key,
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: "Invalid destination public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };

    let authority_pubkey = match Pubkey::from_str(&authority) {
        Ok(key) => key,
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: "Invalid authority public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };

    let associated_token_account =
        get_associated_token_address(&destination_pubkey, &mint_pubkey);

    let mint_to_ix = mint_to_checked(
        &TOKEN_PROGRAM_ID,
        &mint_pubkey,
        &associated_token_account,
        &authority_pubkey,
        &[&authority_pubkey],
        amount,
        6,
    );

    match mint_to_ix {
        Ok(ix) => {
            let response = TokenCreateSuccessResponse {
                success: true,
                data: TokenData {
                    program_id: TOKEN_PROGRAM_ID.to_string(),
                    accounts: ix.accounts,
                    instruction_data: general_purpose::STANDARD.encode(&ix.data),
                },
            };

            return (StatusCode::OK, Json(response)).into_response()
        }
        Err(_) => {
            let error_response = TokenCreateErrorResponse {
                success: false,
                error: String::from("Failed to create mint instruction"),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    }
}

async fn sign_msg(Json(payload): Json<SignMsgRequest>) -> impl IntoResponse {
    let SignMsgRequest { message, secret } = payload;

    if message.is_empty() || secret.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": "Missing required fields"
        }))).into_response();
    }

    let secret_bytes = general_purpose::STANDARD.decode(secret).unwrap();
    let keypair = keypair_from_seed(&secret_bytes).unwrap();

    let signature = keypair.sign_message(message.as_bytes());

    let response = serde_json::json!({
        "success": true,
        "data": {
            "signature": signature.to_string(),
            "public_key": keypair.pubkey().to_string(),
            "message": message
        }
    });

    (StatusCode::OK, Json(response)).into_response()
}

async fn verify_msg(Json(payload): Json<VerifyMsgRequest>) -> impl IntoResponse {
    let VerifyMsgRequest { message, signature, pubkey } = payload;

    let public_key = Pubkey::from_str(&pubkey).unwrap();

    let signature = Signature::from_str(&signature).unwrap();

    let is_valid_signature = signature.verify(&public_key.to_bytes(), message.as_bytes());

    if !is_valid_signature {
        let error_response = json!({
            "success": false,
            "error": "Invalid signature"
        });
        
        return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
    }
    
    let response = json!({
        "success": true,
        "data": {
            "signature": signature.to_string(),
            "public_key": &pubkey,
            "message": message
        }
    });

    return (StatusCode::OK, Json(response)).into_response()

}

async fn send_sol(Json(payload): Json<SendSOLRequest>) -> impl IntoResponse {
    let SendSOLRequest { from, to, lamports } = payload;

    let from_pubkey = match Pubkey::from_str(&from) {
        Ok(key) => key,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid from public key format"
            }))).into_response();
        }
    };

    let to_pubkey = match Pubkey::from_str(&to) {
        Ok(key) => key,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid to public key format"
            }))).into_response();
        }
    };

    let transfer_ix = transfer(
        &from_pubkey,
        &to_pubkey,
        lamports,
    );

    let response = json!({
        "success": true,
        "data": {
            "program_id": transfer_ix.program_id.to_string(),
            "accounts": transfer_ix.accounts,
            "instruction_data": general_purpose::STANDARD.encode(&transfer_ix.data),
        }
    });

    (StatusCode::OK, Json(response)).into_response()
}

async fn send_token(Json(payload): Json<SendTokenRequest>) -> impl IntoResponse {
    let SendTokenRequest { destination, mint, owner, amount } = payload;

    let destination_pubkey = match Pubkey::from_str(&destination) {
        Ok(key) => key,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid destination public key format"
            }))).into_response();
        }
    };

    let mint_pubkey = match Pubkey::from_str(&mint) {
        Ok(key) => key,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid mint public key format"
            }))).into_response();
        }
    };

    let owner_pubkey = match Pubkey::from_str(&owner) {
        Ok(key) => key,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid owner public key format"
            }))).into_response();
        }
    };

    let destination_token_account =
        get_associated_token_address(&destination_pubkey, &mint_pubkey);
    let sender_token_account =
        get_associated_token_address(&owner_pubkey, &mint_pubkey);

    let transfer_ix = transfer_checked(
        &TOKEN_PROGRAM_ID,
        &sender_token_account,
        &mint_pubkey,
        &destination_token_account,
        &owner_pubkey,
        &[&owner_pubkey],
        amount,
        6, // Assuming 6 decimals for the token
    );
    match transfer_ix {
        Ok(ix) => {
            let response = json!({
                "success": true,
                "data": {
                    "program_id": ix.program_id.to_string(),
                    "accounts": ix.accounts,
                    "instruction_data": general_purpose::STANDARD.encode(&ix.data),
                }
            });
            return (StatusCode::OK, Json(response)).into_response();
        },
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": String::from("Failed to create transfer instruction: ")
            }))).into_response();
        }
    };
}
