// crates/ycg_cli/src/main.rs
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use ycg_core::{run_scip_conversion, LevelOfDetail, YcgConfig};

#[derive(Parser)]
#[command(author, version, about = "YAML Code Graph Transcoder")]
struct Args {
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    let lod = match args.lod {
        0 => LevelOfDetail::Low,
        1 => LevelOfDetail::Medium,
        _ => LevelOfDetail::High,
    };

    // Define a raiz do projeto automaticamente se não informada
    let project_root = match args.root {
        Some(p) => p,
        None => args
            .input
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    };

    // CORREÇÃO AQUI: Passando o campo 'compact' que estava faltando
    let config = YcgConfig {
        lod,
        project_root: project_root.clone(),
        compact: args.compact,
    };

    println!("--- YCG: Processando {:?} ---", args.input);

    let yaml_output = run_scip_conversion(&args.input, config)?;

    match args.output {
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
