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
    run_dump
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
    }
}

#[derive(Debug)]
struct Config {
    command: String,
    target: String,
    filename: Option<String>,
    voice_number: Option<u32>,
}

fn parse_config(args: &[String]) -> Config {
    match args.len() {
        4 => Config {
            command: args[0].clone(),
            target: args[1].clone(),
            filename: Some(args[2].clone()),
            voice_number: Some(args[3].parse::<u32>().unwrap()),
        },
        3 => Config {
            command: args[0].clone(),
            target: args[1].clone(),
            filename: Some(args[2].clone()),
            voice_number: None,
        },
        2 => Config {
            command: args[0].clone(),
            target: args[1].clone(),
            filename: None,
            voice_number: None,
        },
        _ => Config {
            command: "None".to_string(),
            target: "None".to_string(),
            filename: None,
            voice_number: None,
        }
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
        }
    }

    /*
    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args[1..]);
    println!("args.len() = {}, config = {:?}", args.len(), config);

    println!("command = {}", config.command);
    println!("filename = {:?}", config.filename);

    let now = SystemTime::now();
    let epoch_now = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let mut output_path = PathBuf::new();
    output_path.push(format!("{}-{:?}.syx", config.target, epoch_now.as_secs()));

    let mut input_path = PathBuf::new();
    input_path.push(config.filename.expect("should have a filename"));

    let yamaha = Manufacturer::Standard(0x42);

    match config.command.as_str() {
        "list" => {
            match read_file(&input_path) {
                Some(filedata) => {
                    dx7::list_cartridge(&filedata);
                },
                None => {
                    eprintln!("Error reading file");
                }
            }
        },
        "dump" => {
            match read_file(&input_path) {
                Some(filedata) => {
                    dx7::dump_cartridge(&filedata);
                },
                None => {
                    eprintln!("Error reading file");
                }
            }
        },
        "generate" => match config.target.as_str() {
            "randomvoice" => {
                let voice = dx7::make_random_voice();
                let header = Header {
                    sub_status: 0x00, // voice/cartridge
                    channel: MIDIChannel::new(1),
                    format: Format::Voice,
                    byte_count: 155,
                };
                let mut payload = voice.to_bytes();
                let sum = checksum(&payload);
                payload.extend(header.to_bytes());
                payload.push(sum);
                let message = Message::ManufacturerSpecific {
                    manufacturer: yamaha,
                    payload,
                };

                if let Err(err) = write_file(&output_path, &message.to_bytes()) {
                    eprintln!("Error writing file: {:?}", err);
                }
            },
            "randomcartridge" => {
                let cartridge = dx7::make_random_cartridge();
                let mut payload = cartridge.to_bytes();
                let sum = checksum(&payload);
                println!("cartridge payload length = {} bytes", payload.len());
                let header = Header {
                    sub_status: 0x00, // voice/cartridge
                    channel: MIDIChannel::new(1),
                    format: Format::Cartridge,
                    byte_count: 4096,
                };
                payload.extend(header.to_bytes());
                payload.push(sum);
                let message = Message::ManufacturerSpecific {
                    manufacturer: yamaha,
                    payload,
                };

                if let Err(err) = write_file(&output_path, &message.to_bytes()) {
                    eprintln!("Error writing file: {:?}", err);
                }
            },
            /*
            "initvoice" => {
                dx7::generate_init_voice(config.filename.unwrap())
            },
            "example" => {
                dx7::generate_example_voice(config.filename.unwrap())
            },
             */
            _ => {
                eprintln!("Unknown target: {}", config.target);
            }
        },
        "extract" => {
            match read_file(&input_path) {
                Some(filedata) => {
                    dx7::extract_voices(&filedata, &input_path);
                },
                None => {
                    eprintln!("Error reading file");
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", config.command);
        }
    }
     */
}

fn read_file(name: &PathBuf) -> Option<Vec<u8>> {
    match fs::File::open(&name) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            match f.read_to_end(&mut buffer) {
                Ok(_) => Some(buffer),
                Err(_) => None
            }
        },
        Err(_) => {
            eprintln!("Unable to open file {}", &name.display());
            None
        }
    }
}

fn write_file(path: &PathBuf, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    //let mut f = fs::File::create(&name).expect("create file");
    fs::write(path, data)?;
    Ok(())
}
