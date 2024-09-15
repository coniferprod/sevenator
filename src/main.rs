use std::env;
use std::io::Error;
use std::path::Path;

use syxpack::read_file;

mod dx7;

pub type Byte = u8;
pub type ByteVector = Vec<u8>;

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

    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args[1..]);
    println!("args.len() = {}, config = {:?}", args.len(), config);

    println!("command = {}", config.command);
    println!("filename = {:?}", config.filename);

    match config.command.as_str() {
        "list" => {
            match read_file(Path::new(&config.filename.expect("no filename"))) {
                Some(filedata) => {
                    dx7::list_cartridge(&filedata);
                },
                None => {
                    eprintln!("Error reading file");
                }
            }
        },
        "dump" => {
            match read_file(Path::new(&config.filename.expect("no filename"))) {
                Some(filedata) => {
                    dx7::dump_cartridge(&filedata);
                },
                None => {
                    eprintln!("Error reading file");
                }
            }
        },
        /*
        "generate" => match config.target.as_str() {
            "randomvoice" => {
                dx7::generate_random_voice(config.filename.unwrap())
            },
            "cartridge" => {
                dx7::generate_cartridge(config.filename.unwrap())
            },
            "initvoice" => {
                dx7::generate_init_voice(config.filename.unwrap())
            },
            "example" => {
                dx7::generate_example_voice(config.filename.unwrap())
            },
            _ => {
                eprintln!("Unknown target: {}", config.target);
                Ok(())
            }
        },
         */
        _ => {
            eprintln!("Unknown command: {}", config.command);
        }
    }
}
