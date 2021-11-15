use std::env;
use std::io::Error;

mod dx7;

struct Config {
    command: String,
    filename: Option<String>,
    voice_number: Option<u32>,
}

fn parse_config(args: &[String]) -> Config {
    match args.len() {
        4 => Config {
            command: args[1].clone(),
            filename: Some(args[2].clone()),
            voice_number: Some(args[3].parse::<u32>().unwrap()),
        },
        3 => Config {
            command: args[1].clone(),
            filename: Some(args[2].clone()),
            voice_number: None,
        },
        2 => Config {
            command: args[1].clone(),
            filename: None,
            voice_number: None,
        },
        _ => Config {
            command: "None".to_string(),
            filename: None,
            voice_number: None,
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args);

    //println!("command = {}", config.command);
    //println!("filename = {}", config.filename);

    //let opt = Opt::from_args();
    //println!("{:?}", opt);

    match config.command.as_str() {
        "dump" => dx7::dump(config.filename.unwrap(), match config.voice_number { None => 0, Some(n) => n }),
        "generate" => dx7::generate(config.filename.unwrap()),
        _ => {
            eprintln!("Unknown command: {}", config.command);
            Ok(())
        }
    }
}
