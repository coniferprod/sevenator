use std::io::Error;

fn main() -> Result<(), Error> {
    env_logger::init();

    sevenator::run()
}
