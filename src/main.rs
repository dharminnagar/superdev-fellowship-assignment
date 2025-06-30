use axum::{
    http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use std::{net::SocketAddr, str::FromStr};
use serde::{Deserialize, Serialize};
use serde_json::{self};

#[tokio::main]
async fn main() {
    // Define your app with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(token_create));

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

async fn token_create(Json(payload): Json<CreateNewToken>) -> Response {
    let CreateNewToken { mint_authority, mint, decimals } = payload;
    
    // Parse public keys from base58 strings
    let mint_pubkey = match Pubkey::from_str(&mint) {
        Ok(key) => key,
        Err(_) => {
            let error_response = ErrorResponse {
                success: false,
                error: "Invalid mint public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    
    let mint_authority_pubkey = match Pubkey::from_str(&mint_authority) {
        Ok(key) => key,
        Err(_) => {
            let error_response = ErrorResponse {
                success: false,
                error: "Invalid mint authority public key format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    
    // Create the initialize mint instruction - FIXED: use spl_instruction instead of instruction
    let init_mint_instruction = match spl_instruction::initialize_mint(
        &spl_token::id(),      // SPL Token program ID
        &mint_pubkey,          // Mint account
        &mint_authority_pubkey, // Mint authority
        None,                  // Freeze authority (None = no freeze authority)
        decimals,              // Decimals
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            let error_response = ErrorResponse {
                success: false,
                error: format!("Failed to create mint instruction: {}", e),
            };
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    
    // Convert accounts to our response format
    let accounts: Vec<AccountInfo> = init_mint_instruction
        .accounts
        .iter()
        .map(|account| AccountInfo {
            pubkey: account.pubkey.to_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        })
        .collect();
    
    // Encode instruction data as base64
    let instruction_data = base64::encode(&init_mint_instruction.data);
    
    let response = SuccessResponse {
        success: true,
        data: TokenCreateData {
            program_id: init_mint_instruction.program_id.to_string(),
            accounts,
            instruction_data,
        },
    };
    
    (StatusCode::OK, Json(response)).into_response()
}

async fn sign_transaction(Json(payload): Json<SignTxRequest>) -> impl IntoResponse {
    let SignTxRequest { message, secret } = payload;

    // Implement the logic to sign the transaction here.
    let keypair = solana_sdk::signer::keypair::Keypair::from_base58_string(&secret);
    let message = solana_sdk::message::Message::(&message)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": format!("Invalid message format: {}", e)
        }))))?;

    let signature = keypair.sign_message(&message);

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Transaction signed successfully"
    })))
}

async fn send_sol(payload: SendSolRequest) -> Result<SendSolResponse, String> {
    // Validate inputs
    if payload.lamports == 0 {
        return Err("Amount must be greater than 0".to_string());
    }

    let from_pubkey =
        parse_pubkey(&payload.from).map_err(|e| format!("Invalid from address: {}", e))?;

    let to_pubkey = parse_pubkey(&payload.to).map_err(|e| format!("Invalid to address: {}", e))?;

    // Create transfer instruction
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, payload.lamports);

    let accounts = vec![from_pubkey.to_string(), to_pubkey.to_string()];

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(response)
}
// async fn get_recent_blockhash() -> solana_sdk::hash::Hash {
//     let client = Client::new();
//     let solana_request = json!({
//         "jsonrpc": "2.0",
//         "id": 1,
//         "method": "getRecentBlockhash",
//         "params": []
//     });

//     let response = client
//         .post("https://api.mainnet-beta.solana.com")
//         .json(&solana_request)
//         .send().await.unwrap().json::<serde_json::Value>().await.unwrap();
// }

#[derive(Serialize, Deserialize, Debug)]
struct CreateTokenRequest {
    mintAuthority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct SendSOLRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SignTxRequest {
    message: String,
    secret: String,
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Serialize)]
struct SendSolResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}