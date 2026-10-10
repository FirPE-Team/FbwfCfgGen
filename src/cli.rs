use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "FbwfCfgGen",
    version,
    about = "Generate an FBWF file-based write filter configuration file",
    arg_required_else_help = true,
    disable_version_flag = true
)]
pub struct Cli {
    /// Cache capacity in MB; must be a multiple of 16
    #[arg(value_name = "CAPACITY_MB")]
    pub(crate) capacity_mb: u32,

    /// Output file path
    #[arg(value_name = "OUTPUT_PATH")]
    pub(crate) output: PathBuf,

    /// Overwrite an existing output file
    #[arg(short = 'f', long)]
    pub(crate) force: bool,
}
