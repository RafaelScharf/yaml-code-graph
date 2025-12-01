// crates/ycg_cli/src/main.rs
use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use ycg_core::{LevelOfDetail, YcgConfig, run_scip_conversion};

#[derive(Parser)]
#[command(author, version, about = "YAML Code Graph Transcoder")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate SCIP index automatically by detecting project language
    Index {
        /// Project directory to index (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,

        /// Output path for SCIP index file
        #[arg(short, long, default_value = "index.scip")]
        output: PathBuf,
    },

    /// Generate YAML graph from existing SCIP index
    Generate {
        /// Caminho para o arquivo de índice SCIP (Input)
        #[arg(short, long)]
        input: PathBuf,

        /// Caminho para o arquivo YAML de saída (Output)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Caminho raiz do projeto fonte
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Nível de Detalhe (0=Low, 1=Medium, 2=High)
        #[arg(short, long, default_value_t = 1)]
        lod: u8,

        /// Ativa modo compacto (Lista de Adjacência)
        #[arg(short, long, default_value_t = false)]
        compact: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { directory, output } => handle_index_command(directory, output),
        Commands::Generate {
            input,
            output,
            root,
            lod,
            compact,
        } => handle_generate_command(input, output, root, lod, compact),
    }
}

fn handle_index_command(directory: PathBuf, output: PathBuf) -> Result<()> {
    println!("--- YCG: Detecting project language in {:?} ---", directory);

    let language = detect_project_language(&directory)?;
    println!("Detected language: {}", language);

    match language.as_str() {
        "rust" => run_rust_indexer(&directory, &output),
        "typescript" | "javascript" => run_typescript_indexer(&directory, &output),
        _ => Err(anyhow!("Unsupported language: {}", language)),
    }
}

fn detect_project_language(directory: &PathBuf) -> Result<String> {
    // Check for Rust project
    if directory.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }

    // Check for TypeScript/JavaScript project
    if directory.join("package.json").exists() || directory.join("tsconfig.json").exists() {
        return Ok("typescript".to_string());
    }

    Err(anyhow!(
        "Could not detect project language. No Cargo.toml, package.json, or tsconfig.json found in {:?}",
        directory
    ))
}

fn run_rust_indexer(directory: &PathBuf, output: &PathBuf) -> Result<()> {
    println!("--- Running Rust SCIP indexer (rust-analyzer) ---");

    // Check if rust-analyzer is available
    let check = Command::new("rust-analyzer").arg("--version").output();

    if check.is_err() {
        return Err(anyhow!(
            "rust-analyzer not found in PATH.\n\
             Please install it with: rustup component add rust-analyzer\n\
             Or visit: https://rust-analyzer.github.io/manual.html#installation"
        ));
    }

    // Convert output path to absolute if it's relative
    let absolute_output = if output.is_absolute() {
        output.clone()
    } else {
        std::env::current_dir()?.join(output)
    };

    // Run rust-analyzer with SCIP export
    let status = Command::new("rust-analyzer")
        .arg("scip-export")
        .arg("--output")
        .arg(&absolute_output)
        .current_dir(directory)
        .status()
        .context("Failed to execute rust-analyzer")?;

    if !status.success() {
        return Err(anyhow!(
            "rust-analyzer scip-export failed with exit code: {:?}",
            status.code()
        ));
    }

    println!("✓ SCIP index generated successfully: {:?}", absolute_output);
    Ok(())
}

fn run_typescript_indexer(directory: &PathBuf, output: &PathBuf) -> Result<()> {
    println!("--- Running TypeScript SCIP indexer (scip-typescript) ---");

    // Try npx first (preferred for local installs)
    let check_npx = Command::new("npx").arg("--version").output();

    let use_npx = check_npx.is_ok();

    if !use_npx {
        // Check if scip-typescript is globally installed
        let check_global = Command::new("scip-typescript").arg("--version").output();

        if check_global.is_err() {
            return Err(anyhow!(
                "scip-typescript not found.\n\
                 Please install it with: npm install -g @sourcegraph/scip-typescript\n\
                 Or ensure npx is available for local execution."
            ));
        }
    }

    // Convert output path to absolute if it's relative
    let absolute_output = if output.is_absolute() {
        output.clone()
    } else {
        std::env::current_dir()?.join(output)
    };

    // Run scip-typescript
    let mut cmd = if use_npx {
        let mut c = Command::new("npx");
        c.arg("@sourcegraph/scip-typescript");
        c
    } else {
        Command::new("scip-typescript")
    };

    let status = cmd
        .arg("index")
        .arg("--output")
        .arg(&absolute_output)
        .current_dir(directory)
        .status()
        .context("Failed to execute scip-typescript")?;

    if !status.success() {
        return Err(anyhow!(
            "scip-typescript failed with exit code: {:?}",
            status.code()
        ));
    }

    println!("✓ SCIP index generated successfully: {:?}", output);
    Ok(())
}

fn handle_generate_command(
    input: PathBuf,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    lod: u8,
    compact: bool,
) -> Result<()> {
    let lod = match lod {
        0 => LevelOfDetail::Low,
        1 => LevelOfDetail::Medium,
        _ => LevelOfDetail::High,
    };

    // Define a raiz do projeto automaticamente se não informada
    let project_root = match root {
        Some(p) => p,
        None => input
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    };

    let config = YcgConfig {
        lod,
        project_root: project_root.clone(),
        compact,
    };

    println!("--- YCG: Processando {:?} ---", input);

    let yaml_output = run_scip_conversion(&input, config)?;

    match output {
        Some(path) => {
            std::fs::write(&path, yaml_output)?;
            println!("Sucesso! Grafo salvo em: {:?}", path);
        }
        None => {
            println!("\n--- OUTPUT ---\n");
            println!("{}", yaml_output);
        }
    }

    Ok(())
}
