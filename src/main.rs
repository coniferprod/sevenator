use std::env;
use std::io::Error;

mod dx7;

pub type Byte = u8;
pub type ByteVector = Vec<u8>;

//
// Experiment a little with the newtype pattern.
// A newtype is a special case of a tuple struct,
// with just one field.
//

// Simple private wrapper for an inclusive range of Ord types.
// We need this because Rust ranges are not Copy
// (see https://github.com/rust-lang/rfcs/issues/2848).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Wrapper<T> where T: Ord {
    start: T,
    end: T,
}

pub trait RandomValue {
    type B;  // semantic type
    type T;  // primitive value type
    fn random_value() -> Self::B;
}

/// Parsing and generating MIDI System Exclusive data.
pub trait SystemExclusiveData {
    fn from_bytes(data: ByteVector) -> Self;
    fn from_packed_bytes(data: ByteVector) -> Self;
    fn to_bytes(&self) -> ByteVector;
    fn to_packed_bytes(&self) -> ByteVector { vec![] }
    fn data_size(&self) -> usize { 0 }
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

fn main() -> Result<(), Error> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args[1..]);
    println!("args.len() = {}, config = {:?}", args.len(), config);

    //println!("command = {}", config.command);
    //println!("filename = {}", config.filename);

    match config.command.as_str() {
        "dump" => dx7::dump(config.filename.unwrap(), match config.voice_number { None => 0, Some(n) => n }),
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
        _ => {
            eprintln!("Unknown command: {}", config.command);
            Ok(())
        }
    }
}
