use clap::{Parser, Subcommand};
use anyhow::Result;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_client::rpc_client::RpcClient;

#[derive(Parser)]
#[command(name = "facilitator-cli")]
#[command(about = "Admin CLI for x402 Facilitator", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new keypair for fee payer
    GenerateKey {
        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Check configuration from .env file
    CheckConfig {
        /// Path to .env file
        #[arg(short, long, default_value = ".env")]
        env_file: String,
    },
    
    /// Test RPC connection
    TestRpc {
        /// RPC URL to test
        #[arg(short, long)]
        url: String,
    },
    
    /// Get account balance
    GetBalance {
        /// Public key (base58)
        pubkey: String,
        
        /// RPC URL
        #[arg(short, long, default_value = "https://api.devnet.solana.com")]
        rpc: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKey { output } => {
            println!("🔑 Generating new keypair...\n");
            
            let keypair = Keypair::new();
            let pubkey = keypair.pubkey();
            let privkey_base58 = bs58::encode(keypair.to_bytes()).into_string();
            
            println!("✅ Generated new keypair:");
            println!("   Public Key:  {}", pubkey);
            println!("   Private Key: {}...", &privkey_base58[..32]);
            println!();
            
            if let Some(path) = output {
                std::fs::write(&path, keypair.to_bytes())?;
                println!("💾 Keypair saved to: {}", path);
            } else {
                println!("💡 To save to file, use: --output <path>");
                println!();
                println!("📝 Add to your .env file:");
                println!("   FEE_PAYER_PRIVATE_KEY={}", privkey_base58);
            }
        }
        
        Commands::CheckConfig { env_file } => {
            println!("🔍 Checking configuration from {}...\n", env_file);
            
            // Load env file
            match dotenvy::from_filename(&env_file) {
                Ok(_) => println!("✅ Environment file loaded"),
                Err(e) => {
                    println!("❌ Failed to load {}: {}", env_file, e);
                    return Ok(());
                }
            }
            
            println!();
            
            // Check each required env var
            let vars = vec![
                ("SOLANA_RPC_URL", true),
                ("FEE_PAYER_PRIVATE_KEY", true),
                ("NETWORK", false),
                ("PORT", false),
                ("RUST_LOG", false),
            ];
            
            let mut all_valid = true;
            
            for (var, required) in vars {
                match std::env::var(var) {
                    Ok(value) => {
                        let display = if var == "FEE_PAYER_PRIVATE_KEY" {
                            format!("{}...", &value[..16])
                        } else {
                            value
                        };
                        println!("✅ {:<25} = {}", var, display);
                    }
                    Err(_) => {
                        if required {
                            println!("❌ {:<25} = NOT SET (required)", var);
                            all_valid = false;
                        } else {
                            println!("⚠️  {:<25} = NOT SET (using default)", var);
                        }
                    }
                }
            }
            
            println!();
            if all_valid {
                println!("✅ Configuration is valid!");
            } else {
                println!("❌ Configuration has errors");
            }
        }
        
        Commands::TestRpc { url } => {
            println!("🔍 Testing RPC connection to {}...\n", url);
            
            let client = RpcClient::new(url.clone());
            
            // Test health
            print!("   Health check... ");
            match client.get_health() {
                Ok(_) => println!("✅ healthy"),
                Err(e) => {
                    println!("❌ unhealthy: {}", e);
                    return Ok(());
                }
            }
            
            // Get version
            print!("   Version check... ");
            match client.get_version() {
                Ok(version) => {
                    println!("✅");
                    println!("   Solana version: {}", version.solana_core);
                }
                Err(e) => {
                    println!("❌ {}", e);
                }
            }
            
            // Get slot
            print!("   Latest slot... ");
            match client.get_slot() {
                Ok(slot) => println!("✅ {}", slot),
                Err(e) => println!("❌ {}", e),
            }
            
            println!("\n✅ RPC connection is working!");
        }
        
        Commands::GetBalance { pubkey, rpc } => {
            println!("💰 Checking balance for {}...\n", pubkey);
            
            // Parse pubkey
            let pubkey = match solana_sdk::pubkey::Pubkey::try_from(pubkey.as_str()) {
                Ok(pk) => pk,
                Err(e) => {
                    println!("❌ Invalid public key: {}", e);
                    return Ok(());
                }
            };
            
            let client = RpcClient::new(rpc);
            
            match client.get_balance(&pubkey) {
                Ok(balance) => {
                    let sol = balance as f64 / 1_000_000_000.0;
                    println!("✅ Balance: {} lamports ({:.9} SOL)", balance, sol);
                }
                Err(e) => {
                    println!("❌ Failed to get balance: {}", e);
                }
            }
        }
    }

    Ok(())
}

