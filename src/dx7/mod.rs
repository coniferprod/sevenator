use rand::Rng;

use sevenate::dx7::lfo::Lfo;
use syxpack::Message;
use sevenate::Ranged;
use sevenate::dx7::{
    Depth,
    Algorithm,
    Transpose,
    Level,
    Detune,
    Coarse,
    Sensitivity
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
    KeyboardLevelScaling,
    Operator,
    Key,
    Scaling,
    ScalingCurve,
    OperatorMode
};
use sevenate::dx7::envelope::{
    Envelope,
    Rate
};
use sevenate::dx7::lfo::LfoWaveform;
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
