use std::fs;
use std::env;
use std::path::{
    Path,
    PathBuf
};
use std::io::{
    Read,
    Error
};
use std::time::{
    SystemTime,
    UNIX_EPOCH
};

use clap::{
    Parser,
    Subcommand
};

use sevenate::Ranged;
use sevenate::dx7::sysex::{
    SystemExclusiveData,
    Header,
    Format,
    MIDIChannel,
    checksum
};

use syxpack::{
    Message,
    Manufacturer
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
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long)]
        file: PathBuf,
    },

    Extract {
        #[arg(short, long)]
        file: PathBuf,
    },

    Dump {
        #[arg(short, long)]
        file: PathBuf,

        #[arg(short, long)]
        number: Option<u8>,
    },

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
    }
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
        }
    }
}


fn write_file(path: &PathBuf, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    //let mut f = fs::File::create(&name).expect("create file");
    fs::write(path, data)?;
    Ok(())
}
