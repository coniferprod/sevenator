use std::env;
use std::io::Error;
use std::path::PathBuf;

mod dx7;

struct Config {
    command: String,
    filename: String,
}

fn parse_config(args: &[String]) -> Config {
    let command = args[1].clone();
    let filename = if args.len() >= 3 {
        args[2].clone()
    } else {
        String::from("")
    };

    Config { command, filename }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args);

    println!("command = {}", config.command);
    println!("filename = {}", config.filename);

    //let opt = Opt::from_args();
    //println!("{:?}", opt);

    match config.command.as_str() {
        "dump" => dx7::dump(config.filename),
        "generate" => dx7::generate(config.filename),
        _ => {
            eprintln!("Unknown command: {}", config.command);
            Ok(())
        }
    }
}
