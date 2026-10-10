use std::fs;
use std::path::{Path};

use anyhow::{Context, Result, bail};
use clap::Parser;
use crate::cli::Cli;

#[cfg(test)]
mod test;
mod cli;

const CONFIG_SIZE: usize = 0x1000;
const MIN_CAPACITY_MB: u32 = 128;
const MAX_CAPACITY_MB: u32 = 131_072;
const VOLUME_PATH: &str = r"\??\Volume{d9b257fc-684e-4dcb-ab79-03cfa2f6b750}";

fn validate_capacity(capacity_mb: u32) -> Result<()> {
    if !(MIN_CAPACITY_MB..=MAX_CAPACITY_MB).contains(&capacity_mb) {
        bail!(
            "Capacity must be between {MIN_CAPACITY_MB} MB and {MAX_CAPACITY_MB} MB; got {capacity_mb} MB"
        );
    }
    if !capacity_mb.is_multiple_of(16) {
        bail!("Capacity must be a multiple of 16 MB; got {capacity_mb} MB");
    }
    Ok(())
}

fn build_config(capacity_mb: u32) -> Result<Vec<u8>> {
    validate_capacity(capacity_mb)?;

    let units = u16::try_from(capacity_mb / 16)
        .expect("validated capacity always fits in a 16-bit unit count");
    let mut buffer = vec![0u8; CONFIG_SIZE];

    buffer[0x01..=0x02].copy_from_slice(&[0xF9, 0x23]);
    buffer[0x04] = 0x02;
    buffer[0x08] = 0x01;
    buffer[0x10] = 0x02;
    buffer[0x1B..=0x1C].copy_from_slice(&units.to_le_bytes());
    buffer[0x20] = 0x01;
    buffer[0x28] = 0x02;
    buffer[0x00] = 0x24u8.wrapping_sub(buffer[0x1C]);
    buffer[0x03] = 0xF8u8.wrapping_sub(buffer[0x1B]);

    let mut path_offset = 0x2C;
    for code_unit in VOLUME_PATH.encode_utf16().chain(std::iter::once(0)) {
        let encoded = code_unit.to_le_bytes();
        buffer[path_offset..path_offset + 2].copy_from_slice(&encoded);
        path_offset += 2;
    }

    Ok(buffer)
}

fn write_config(output: &Path, config: &[u8], force: bool) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!(
            "Output directory does not exist or is not a directory: {}",
            parent.display()
        );
    }

    if output.exists() && !force {
        bail!(
            "Output file already exists; use --force to overwrite: {}",
            output.display()
        );
    }

    fs::write(output, config)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    let config = build_config(cli.capacity_mb)?;
    let existed = cli.output.exists();
    write_config(&cli.output, &config, cli.force)?;

    if existed {
        println!(
            "Rebuilt {} MB configuration: {}",
            cli.capacity_mb,
            cli.output.display()
        );
    } else {
        println!(
            "Created {} MB configuration: {}",
            cli.capacity_mb,
            cli.output.display()
        );
    }
    Ok(())
}

fn main() -> Result<()> {
    run(Cli::parse())
}
