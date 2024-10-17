use rand::Rng;

use sevenate::dx7::lfo::Lfo;
use syxpack::Message;
use sevenate::Ranged;
use sevenate::dx7::{
    Depth,
    Algorithm,
    Transpose
};
use sevenate::dx7::cartridge::{
    Cartridge,
    VOICE_COUNT
};
use sevenate::dx7::voice::{
    Voice,
    VoiceName,
    OPERATOR_COUNT
};
use sevenate::dx7::operator::{
    Operator,
};
use sevenate::dx7::envelope::Envelope;
use sevenate::dx7::sysex::{
    SystemExclusiveData,
    Header
};

pub mod randomizer;

// Makes a cartridge filled with random voices.
pub fn make_random_cartridge() -> Cartridge {
    let mut voices: Vec<Voice> = Vec::new();
    for i in 0..VOICE_COUNT {
        voices.push(make_random_voice());
    }
    Cartridge { voices }
}

pub fn make_random_voice() -> Voice {
    let mut voice = Voice::new();

    for i in 0..OPERATOR_COUNT {
        voice.operators[i] = Operator::random();
    }

    voice.peg = Envelope::random();
    voice.alg = Algorithm::random();
    voice.feedback = Depth::random();

    let mut rng = rand::thread_rng();
    voice.osc_sync = rng.gen();

    voice.lfo = Lfo::random();
    voice.pitch_mod_sens = Depth::random();
    voice.transpose = Transpose::random();
    voice.name = VoiceName::random();

    voice
}

pub fn list_cartridge(filedata: &[u8]) {
    match Message::from_bytes(&filedata) {
        Ok(Message::ManufacturerSpecific { manufacturer: _, payload }) => {
            println!("message payload length = {}", payload.len());
            let cartridge_data = &payload[Header::DATA_SIZE..];
            println!("cartridge data length = {}", cartridge_data.len());
            let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();

            for voice in cartridge.voices.iter() {
                println!("'{}'", voice.name.value());
            }
        },
        _ => {
            eprintln!("invalid SysEx message");
        }
    }
}

pub fn dump_cartridge(filedata: &[u8]) {
    match Message::from_bytes(&filedata) {
        Ok(Message::ManufacturerSpecific { manufacturer: _, payload }) => {
            println!("message payload length = {}", payload.len());
            let cartridge_data = &payload[Header::DATA_SIZE..];
            println!("cartridge data length = {}", cartridge_data.len());
            let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();

            for (index, voice) in cartridge.voices.iter().enumerate() {
                println!("VOICE {}: {}\n", index + 1, voice);
            }
        },
        _ => {
            eprintln!("invalid SysEx message");
        }
    }
}
