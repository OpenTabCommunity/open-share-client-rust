use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

use openshare_core::{ClientConfig, Identity, Manifest, Client};
use storage::{LocalStorage, Storage};

#[derive(Parser, Debug)]
#[command(name = "openshare", version, about = "OpenShare P2P File Transfer")]
struct Cli {
    /// Set log level: error,warn,info,debug,trace
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    /// Data directory for storage
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new device identity
    Init {
        /// Device name/identifier
        #[arg(long)]
        device_id: String,

        /// Account identifier (will be hashed for discovery)
        #[arg(long)]
        account: String,
    },

    /// Show device information
    Info,

    /// Announce this device on the local network
    Announce {
        /// Network interface to use
        #[arg(long)]
        interface: String,

        /// Port to listen on
        #[arg(long, default_value_t = 9876)]
        port: u16,

        /// Keep announcing (0 = forever)
        #[arg(long, default_value_t = 0)]
        ttl: u64,
    },

    /// Discover devices on the local network
    Discover {
        /// Network interface to use
        #[arg(long)]
        interface: String,

        /// Discovery timeout in seconds
        #[arg(long, default_value_t = 5)]
        timeout: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create a manifest from a file
    CreateManifest {
        /// File to create manifest for
        #[arg(long)]
        file: PathBuf,

        /// Output manifest file
        #[arg(long)]
        output: PathBuf,
    },

    /// Verify a manifest signature
    VerifyManifest {
        /// Manifest file to verify
        #[arg(long)]
        manifest: PathBuf,
    },

    /// Send a file to a peer
    Send {
        /// File to send
        #[arg(long)]
        file: PathBuf,

        /// Peer address (host:port)
        #[arg(long)]
        peer: String,
    },

    /// Listen for incoming transfers
    Listen {
        /// Port to listen on
        #[arg(long, default_value_t = 9876)]
        port: u16,

        /// Output directory for received files
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::new(&cli.log_level))
        .with_target(false)
        .init();

    // Determine data directory
    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".openshare")
    });

    let identity_path = data_dir.join("identity.key");

    match cli.cmd {
        Commands::Init { device_id, account } => {
            std::fs::create_dir_all(&data_dir)?;

            let identity = Identity::generate_and_store(&identity_path)?;

            // Create config with account hash
            let account_hash = compute_account_hash(&account);
            let mut cfg = ClientConfig::default();
            cfg.data_dir = data_dir.clone();
            cfg.device_id = device_id.clone();
            cfg.account_hash = account_hash.clone();

            cfg.ensure_data_dir()?;

            // Save config
            let cfg_path = data_dir.join("config.json");
            let cfg_json = serde_json::to_string_pretty(&cfg)?;
            std::fs::write(cfg_path, cfg_json)?;

            println!("✓ Device initialized");
            println!("  Device ID: {}", device_id);
            println!("  Account: {}", account);
            println!("  Fingerprint: {}", identity.fingerprint());
            println!("  Full fingerprint: {}", identity.full_fingerprint());
            println!("  Data directory: {}", data_dir.display());
        }

        Commands::Info => {
            if !identity_path.exists() {
                anyhow::bail!("Device not initialized. Run 'openshare init' first.");
            }

            let identity = Identity::load(&identity_path)?;
            let cfg = load_config(&data_dir)?;

            println!("Device Information:");
            println!("  Device ID: {}", cfg.device_id);
            println!("  Account hash: {}", cfg.account_hash);
            println!("  Fingerprint: {}", identity.fingerprint());
            println!("  Full fingerprint: {}", identity.full_fingerprint());
            println!("  Data directory: {}", data_dir.display());
            println!("  Listen port: {}", cfg.listen_port);
        }

        Commands::Announce { interface, port, ttl } => {
            let identity = Identity::load(&identity_path)
                .context("Device not initialized. Run 'openshare init' first.")?;
            let cfg = load_config(&data_dir)?;

            announce_device(&cfg, &identity, &interface, port, ttl).await?;
        }

        Commands::Discover { interface, timeout, json } => {
            let cfg = load_config(&data_dir)?;
            discover_devices(&cfg, &interface, timeout, json).await?;
        }

        Commands::CreateManifest { file, output } => {
            let identity = Identity::load(&identity_path)
                .context("Device not initialized. Run 'openshare init' first.")?;
            let cfg = load_config(&data_dir)?;

            let mut manifest = Manifest::from_file(
                file.to_str().unwrap(),
                cfg.chunk_size
            )?;

            manifest.sign(&identity)?;

            let manifest_json = serde_json::to_string_pretty(&manifest)?;
            std::fs::write(&output, manifest_json)?;

            println!("✓ Manifest created: {}", output.display());
            println!("  {}", manifest.summary());
        }

        Commands::VerifyManifest { manifest: manifest_path } => {
            let manifest_json = std::fs::read_to_string(&manifest_path)?;
            let manifest: Manifest = serde_json::from_str(&manifest_json)?;

            match manifest.verify() {
                Ok(_) => println!("✓ Manifest signature is valid"),
                Err(e) => {
                    println!("✗ Manifest signature is invalid: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Send { file, peer } => {
            let identity = Identity::load(&identity_path)
                .context("Device not initialized. Run 'openshare init' first.")?;
            let cfg = load_config(&data_dir)?;
            let storage = LocalStorage::new(data_dir.clone())?;

            send_file(&identity, &cfg, &storage, &file, &peer).await?;
        }

        Commands::Listen { port, output } => {
            let identity = Identity::load(&identity_path)
                .context("Device not initialized. Run 'openshare init' first.")?;
            let mut cfg = load_config(&data_dir)?;
            cfg.listen_port = port;
            let storage = LocalStorage::new(data_dir.clone())?;

            let output_dir = output.unwrap_or_else(|| std::env::current_dir().unwrap());

            listen_for_transfers(&identity, &cfg, &storage, &output_dir).await?;
        }
    }

    Ok(())
}

fn compute_account_hash(account: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(account.as_bytes());
    hex::encode(&hasher.finalize()[..8]) // Use first 8 bytes for compact hash
}

fn load_config(data_dir: &PathBuf) -> Result<ClientConfig> {
    let cfg_path = data_dir.join("config.json");
    if !cfg_path.exists() {
        anyhow::bail!("Device not initialized. Run 'openshare init' first.");
    }

    let cfg_json = std::fs::read_to_string(cfg_path)?;
    let cfg: ClientConfig = serde_json::from_str(&cfg_json)?;
    Ok(cfg)
}

async fn announce_device(
    cfg: &ClientConfig,
    identity: &Identity,
    interface: &str,
    port: u16,
    ttl: u64,
) -> Result<()> {
    use mdns_core::{announce::Announcer, model::{ServiceAnnouncement, TxtRecord}, net::list_interface_ips_result};

    let interface_ips = list_interface_ips_result()?;
    let ip = interface_ips
        .iter()
        .find(|item| item.name == interface)
        .ok_or_else(|| anyhow::anyhow!("No matching interface found: {}", interface))?
        .ip;

    let txt = vec![
        ("acct_hash".to_string(), cfg.account_hash.clone()),
        ("dev_id".to_string(), cfg.device_id.clone()),
        ("fp".to_string(), identity.fingerprint()),
    ];

    let ann = ServiceAnnouncement {
        service_type: cfg.service_type.clone(),
        instance_name: cfg.device_id.clone(),
        host_name: format!("{}.local.", cfg.device_id),
        ip_addr: ip.to_string(),
        port,
        txt: Some(TxtRecord(txt)),
    };

    let announcer = Announcer::register(ann)?;
    tracing::info!("Announcing: {}", announcer.fullname());
    println!("✓ Announcing device on {}:{}", ip, port);
    println!("  Service: {}", announcer.fullname());

    if ttl == 0 {
        println!("  Press Ctrl+C to stop");
        std::thread::park();
    } else {
        tokio::time::sleep(Duration::from_secs(ttl)).await;
    }

    Ok(())
}

async fn discover_devices(
    cfg: &ClientConfig,
    interface: &str,
    timeout: u64,
    json: bool,
) -> Result<()> {
    use mdns_core::{discover::browse_blocking, net::list_interface_ips_result};

    let interface_ips = list_interface_ips_result()?;
    interface_ips
        .iter()
        .find(|item| item.name == interface)
        .ok_or_else(|| anyhow::anyhow!("No matching interface found: {}", interface))?;

    let results = browse_blocking(&cfg.service_type, Duration::from_secs(timeout), interface)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Discovered {} device(s):", results.len());
        for svc in results {
            println!("\n  {} @ {}:{}", svc.instance_name, svc.host_name, svc.port);
            println!("    Addresses:");
            for addr in &svc.addresses {
                println!("      - {}", addr);
            }
            println!("    TXT records:");
            for (k, v) in &svc.txt {
                println!("      {} = {}", k, v);
            }
        }
    }

    Ok(())
}

async fn send_file(
    identity: &Identity,
    cfg: &ClientConfig,
    storage: &LocalStorage,
    file: &PathBuf,
    peer: &str,
) -> Result<()> {
    use tokio::net::TcpStream;

    println!("Preparing to send: {}", file.display());

    // Create manifest
    let mut manifest = Manifest::from_file(file.to_str().unwrap(), cfg.chunk_size)?;
    manifest.sign(identity)?;
    println!("  {}", manifest.summary());

    // Store chunks locally first
    println!("Chunking file...");
    let mut f = std::fs::File::open(file)?;
    use std::io::Read;
    let mut buf = vec![0u8; cfg.chunk_size];
    let mut stored_chunks = 0;

    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        storage.put_chunk(&buf[..n]).await?;
        stored_chunks += 1;
    }
    println!("  Stored {} chunks locally", stored_chunks);

    // Connect to peer
    println!("Connecting to {}...", peer);
    let stream = TcpStream::connect(peer).await
        .context("Failed to connect to peer")?;
    println!("✓ Connected");

    // Create client and send
    let client = Client::new(identity.clone(), storage.clone(), cfg.clone());
    client.send_manifest_over(stream, manifest).await?;

    println!("✓ File sent successfully");
    Ok(())
}

async fn listen_for_transfers(
    identity: &Identity,
    cfg: &ClientConfig,
    storage: &LocalStorage,
    output_dir: &PathBuf,
) -> Result<()> {
    use tokio::net::TcpListener;

    let addr = format!("0.0.0.0:{}", cfg.listen_port);
    let listener = TcpListener::bind(&addr).await?;

    println!("✓ Listening on {}", addr);
    println!("  Output directory: {}", output_dir.display());
    println!("  Press Ctrl+C to stop");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        println!("\n← Incoming connection from {}", peer_addr);

        let identity = identity.clone();
        let cfg = cfg.clone();
        let storage = storage.clone();
        let output_dir = output_dir.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_transfer(identity, cfg, storage, stream, output_dir).await {
                tracing::error!("Transfer failed: {}", e);
                println!("✗ Transfer failed: {}", e);
            }
        });
    }
}

async fn handle_transfer(
    identity: Identity,
    cfg: ClientConfig,
    storage: LocalStorage,
    stream: tokio::net::TcpStream,
    output_dir: PathBuf,
) -> Result<()> {
    let client = Client::new(identity, storage.clone(), cfg.clone());

    println!("  Receiving manifest...");
    let manifest = client.accept_and_receive(stream).await?;

    println!("  {}", manifest.summary());

    // Verify manifest signature
    manifest.verify().context("Invalid manifest signature")?;
    println!("  ✓ Signature verified");

    // Reconstruct file from chunks
    let output_path = output_dir.join(&manifest.filename);
    println!("  Writing to: {}", output_path.display());

    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    let mut outfile = File::create(&output_path).await?;

    for (i, chunk_hash) in manifest.chunk_hashes.iter().enumerate() {
        if let Some(chunk_data) = storage.get_chunk(chunk_hash).await? {
            outfile.write_all(&chunk_data).await?;
            if (i + 1) % 10 == 0 || i + 1 == manifest.chunk_hashes.len() {
                println!("    Progress: {}/{} chunks", i + 1, manifest.chunk_hashes.len());
            }
        } else {
            anyhow::bail!("Missing chunk: {}", chunk_hash);
        }
    }

    outfile.flush().await?;
    println!("✓ File received: {}", output_path.display());

    Ok(())
}