// crates/ycg_core/build.rs
use std::io::Result;

fn main() -> Result<()> {
    // Avisa o Cargo para recompilar se o arquivo .proto mudar
    println!("cargo:rerun-if-changed=../../proto/scip.proto");

    // Configura o prost para compilar o .proto
    let mut config = prost_build::Config::new();

    // Isso garante que os campos opcionais usem Option<T> do Rust
    config.protoc_arg("--experimental_allow_proto3_optional");

    config.compile_protos(
        &["../../proto/scip.proto"], // Arquivo entrada
        &["../../proto/"],           // Diretório raiz de includes
    )?;

    Ok(())
}
