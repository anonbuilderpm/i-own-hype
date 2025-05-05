use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, utils};
use std::fs;
use std::path::PathBuf;

// Public inputs structure
#[derive(Deserialize, Serialize, Debug)]
struct PublicInputs {
    message_digest: String,
    merkle_root: String,
}

// Balance range result
#[derive(Debug, Serialize, Deserialize)]
struct BalanceRange {
    in_range: bool,
    min_balance: u64,
    max_balance: u64,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a previously generated proof
    Verify {
        /// Path to the binary proof file generated with the 'prove' command
        #[arg(short, long)]
        proof_file: PathBuf,
    },
}

fn main() {
    // Setup logging
    utils::setup_logger();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Verify { proof_file } => {
            println!("Verifying token ownership proof...");
            
            // Get the ELF file
            let elf_path = std::env::var("SP1_ELF_token-ownership-program")
                .expect("ELF path not found. Did you run 'cargo prove build' in the program directory?");
            let elf = fs::read(elf_path).expect("Failed to read ELF file");
            
            // Create a ProverClient
            let client = ProverClient::from_env();
            
            // Load the proof (binary format)
            let proof = SP1ProofWithPublicValues::load(proof_file).expect("Failed to load proof");
            
            // Setup the verification key
            let (_, vk) = client.setup(&elf);
            
            // Verify the proof
            client.verify(&proof, &vk).expect("Proof verification failed");
            
            // Read public outputs
            let mut public_values = proof.public_values.clone();
            let balance_range: BalanceRange = public_values.read();
            let committed_public_inputs: PublicInputs = public_values.read();
            
            println!("\n=== Proof Successfully Verified ===");
            if balance_range.in_range {
                println!("✓ Verified: Balance is between {} and {} HYPE", 
                    balance_range.min_balance as f64 / 100_000_000.0,
                    balance_range.max_balance as f64 / 100_000_000.0);
            } else {
                println!("✗ Balance is NOT within the required range");
            }
            println!("\nCommitted Public Values:");
            println!("- Message Digest: {}", committed_public_inputs.message_digest);
            println!("- Merkle Root: {}", committed_public_inputs.merkle_root);
        }
    }
} 