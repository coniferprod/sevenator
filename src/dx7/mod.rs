use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use rand::Rng;

use sevenate::dx7::lfo::Lfo;
use syxpack::{
    Message,
    Manufacturer
};
use sevenate::Ranged;
use sevenate::dx7::{
    Algorithm, Coarse, Depth, Detune, Level, Sensitivity, Transpose
};
use sevenate::dx7::cartridge::{
    Cartridge,
    VOICE_COUNT
};
use sevenate::dx7::voice::{
    Voice,
    VoiceName,
    OPERATOR_COUNT,
};
use sevenate::dx7::operator::{
    KeyboardLevelScaling,
    Operator,
    OperatorMode
};
use sevenate::dx7::envelope::{
    Envelope,
    Rate
};
use sevenate::dx7::lfo::LfoWaveform;
use sevenate::dx7::sysex::{
    SystemExclusiveData,
    Header,
    Format,
    checksum
};

pub mod randomizer;

// Makes a cartridge filled with random voices.
pub fn make_random_cartridge() -> Cartridge {
    let mut voices: Vec<Voice> = Vec::new();
    for _ in 0..VOICE_COUNT {
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

    let mut rng = rand::rng();
    voice.osc_sync = rng.random();

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
            let cartridge = Cartridge::parse(&cartridge_data).unwrap();

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
            let cartridge = Cartridge::parse(&cartridge_data).unwrap();

            for (index, voice) in cartridge.voices.iter().enumerate() {
                println!("VOICE {}: {}\n", index + 1, voice);
            }
        },
        _ => {
            eprintln!("invalid SysEx message");
        }
    }
}

pub fn extract_voices(filedata: &[u8], path: &PathBuf) {
    match Message::from_bytes(&filedata) {
        Ok(Message::ManufacturerSpecific { manufacturer: _, payload }) => {
            println!("message payload length = {}", payload.len());

            match Header::parse(&payload) {
                Ok(mut header) => {
                    println!("{}", header);

                    let data = &payload[4..payload.len() - 1];
                    println!("data length = {}", data.len());

                    match header.format {
                        Format::Voice => {
                            eprintln!("Not extracting an individual voice");
                        },
                        Format::Cartridge => {
                            // For a cartridge, pick out the data for each
                            // of the 32 voices. Then unpack the voice data
                            // and write it out to a new file.
                            let mut voice_number = 1;
                            let stem = path.file_stem().unwrap().to_str().unwrap();
                            for packed_voice_data in data.chunks(128) {
                                let voice_data = Voice::unpack(packed_voice_data);
                                let mut payload = Vec::<u8>::new();

                                // Change the format and byte count in the header,
                                // then add it to the file data. Use the original channel.
                                header.format = Format::Voice;
                                header.byte_count = 155;
                                payload.extend(header.to_bytes());
                                payload.extend(&voice_data);

                                payload.push(checksum(&voice_data.clone()));

                                let message = Message::ManufacturerSpecific {
                                    manufacturer: Manufacturer::Standard(0x43),
                                    payload
                                };

                                let filename = format!("{}-{:02}.syx", stem, voice_number);
                                match File::create(filename) {
                                    Ok(mut file) => {
                                        match file.write_all(&message.to_bytes()) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                eprintln!("Error writing file: {}", e);
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error creating file: {}", e);
                                    }
                                }

                                voice_number += 1;
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("Error in message: {:?}", e);
        },
        _ => {
            eprintln!("Not a manufacturer-specific System Exclusive message");
        }
    }
}

//
// Utilities for creating voices and cartridges
//

/// Makes an initialized voice. The defaults are as described in
/// Howard Massey's "The Complete DX7", Appendix B.
pub fn make_init_voice() -> Voice {
    let init_eg = Envelope::new();

    let init_op1 = Operator {
        eg: init_eg.clone(),
        kbd_level_scaling: KeyboardLevelScaling::new(),
        kbd_rate_scaling: Depth::new(0),
        amp_mod_sens: Sensitivity::new(0),
        key_vel_sens: Depth::new(0),
        output_level: Level::new(99),
        mode: OperatorMode::Ratio,
        coarse: Coarse::new(1),
        fine: Level::new(0),
        detune: Detune::default(),
    };

    // Operators 2...6 are identical to operator 1 except they
    // have their output level set to zero.
    let init_op_rest = Operator {
        output_level: Level::new(0),
        ..init_op1
    };

    Voice {
        operators: [
            init_op1.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
        ],
        peg: Envelope::new_rate_level(
            [Rate::new(99), Rate::new(99), Rate::new(99), Rate::new(99)],
            [Level::new(50), Level::new(50), Level::new(50), Level::new(50)]),
        alg: Algorithm::new(1),
        feedback: Depth::new(0),
        osc_sync: true, // osc key sync = on
        lfo: Lfo {
            speed: Level::new(35),
            delay: Level::new(0),
            pmd: Level::new(0),
            amd: Level::new(0),
            sync: true,
            waveform: LfoWaveform::Triangle,
        },
        pitch_mod_sens: Depth::new(3),
        transpose: Transpose::new(0),
        name: VoiceName::new("INIT VOICE"),
    }
}
