use axum::{
    routing::{post},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use bitcoin::Address;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use risc0_zkvm::ProverOpts;

// Shared application state
#[derive(Clone)]
struct AppState {
    prover: Arc<Mutex<ProverOpts>>,
}

#[derive(Serialize, Deserialize)]
struct ProofRequest {
    address: String,
    signature: String,
    threshold_amount: u64,
}

#[derive(Serialize)]
struct ProofResponse {
    proof: Vec<u8>,
    public_output: Vec<u8>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn generate_proof(
    State(state): State<AppState>,
    Json(req): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify the Bitcoin signature
    verify_bitcoin_signature(&req.address, &req.signature)
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() })
        ))?;

    // Fetch UTXOs for the address
    let utxos = fetch_utxos(&req.address)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    // Create the prover input
    let input = BalanceInput {
        address: req.address,
        threshold_amount: req.threshold_amount,
        utxo_set: utxos,
    };

    // Generate the proof using RISC0
    let prover = state.prover.lock().await;
    let proof_result = prover
        .with_input(&input)
        .prove()
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;

    Ok(Json(ProofResponse {
        proof: proof_result.proof,
        public_output: proof_result.public_output,
    }))
}

async fn verify_proof(
    State(state): State<AppState>,
    Json(proof): Json<Vec<u8>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    verify_risc0_proof(&proof)
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() })
        ))?;

    Ok(StatusCode::OK)
}

// Helper functions
async fn fetch_utxos(address: &str) -> Result<Vec<UTXO>, Box<dyn std::error::Error>> {
    // Implement UTXO fetching logic here
    // This could use bitcoincore-rpc or a third-party API
    todo!("Implement UTXO fetching")
}

fn verify_bitcoin_signature(address: &str, signature: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bitcoin signature verification
    todo!("Implement signature verification")
}

fn verify_risc0_proof(proof: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Implement RISC0 proof verification
    todo!("Implement proof verification")
}

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create application state
    let state = AppState {
        prover: Arc::new(Mutex::new(ProverOpts::new())),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    // Build the router
    let app = Router::new()
        .route("/generate-proof", post(generate_proof))
        .route("/verify-proof", post(verify_proof))
        .layer(cors)
        .with_state(state);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
