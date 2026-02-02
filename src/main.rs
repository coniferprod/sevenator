use std::path::PathBuf;

use clap::{
    Parser,
    Subcommand
};

pub mod cmd;
pub mod dx7;
pub mod tx802;

use crate::cmd::{
    run_list,
    run_extract,
    run_dump,
    run_make_xml,
    run_make_syx,
    run_repl,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List the voices in a cartridge file
    List {
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Extract the voices in a cartridge file to separate voice files
    Extract {
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Dump a System Exclusive file
    Dump {
        #[arg(short, long)]
        file: PathBuf,

        #[arg(short, long)]
        number: Option<u8>,
    },

    /// Make XML file from System Exclusive
    MakeXml {
        #[arg(short, long)]
        input_file: PathBuf,

        #[arg(short, long)]
        output_file: PathBuf,
    },

    /// Make System Exclusive file from XML
    MakeSyx {
        #[arg(short, long)]
        input_file: PathBuf,

        #[arg(short, long)]
        output_file: PathBuf,
    },

    /// Start a REPL for commands
    Repl,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::List { file } => {
            let path = PathBuf::from(file);
            run_list(&path);
        },
        Commands::Extract { file } => {
            let path = PathBuf::from(file);
            run_extract(&path);
        },
        Commands::Dump { file, number } => {
            let path = PathBuf::from(file);
            run_dump(&path, number);
        },
        Commands::MakeXml { input_file, output_file } => {
            let input_path = PathBuf::from(input_file);
            let output_path = PathBuf::from(output_file);
            run_make_xml(&input_path, &output_path);
        },
        Commands::MakeSyx { input_file, output_file } => {
            let input_path = PathBuf::from(input_file);
            let output_path = PathBuf::from(output_file);
            run_make_syx(&input_path, &output_path);
        },
        Commands::Repl => {
            run_repl().unwrap();
        },
    }
}
