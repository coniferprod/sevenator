use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use syxpack::{Message, Manufacturer, read_file};
use crate::{Byte, ByteVector};
use sevenate::dx7::cartridge::Cartridge;
use sevenate::dx7::voice::Voice;
use sevenate::dx7::sysex::{
    SystemExclusiveData,
    Header
};

// Makes a cartridge filled with random voices.
/*
fn make_random_cartridge() -> Cartridge {
    let mut voices: Vec<Voice> = Vec::new();
    for i in 0..VOICE_COUNT {
        voices.push(make_random_voice());
    }
    Cartridge { voices }
}
 */

 /*
fn generate_voice(voice: Voice) -> ByteVector {
    let voice_data = voice.to_bytes();
    let checksum = voice_checksum(&voice_data);

    let mut payload = vec![
        0x00,   // MIDI channel 1
        0x00,   // format = 0 (1 voice)
        0x01,   // byte count MSB
        0x1B,   // byte count LSB
    ];
    payload.extend(voice_data);
    payload.push(checksum);

    Message::ManufacturerSpecific {
        manufacturer: Manufacturer::Standard(0x43),
        payload
    }.to_bytes()
}
 */

 /*
pub fn generate_random_voice(output_filename: String) -> std::io::Result<()> {
    let output = generate_voice(make_random_voice());

    {
        let mut file = File::create(output_filename)?;
        file.write_all(&output)?;
    }

    Ok(())
}
 */

 /*
pub fn generate_init_voice(output_filename: String) -> std::io::Result<()> {
    let output = generate_voice(make_init_voice());

    {
        let mut file = File::create(output_filename)?;
        file.write_all(&output)?;
    }

    Ok(())
}
 */

 /*
pub fn generate_example_voice(output_filename: String) -> std::io::Result<()> {
    let output = generate_voice(make_example("1.1"));

    {
        let mut file = File::create(output_filename)?;
        file.write_all(&output)?;
    }

    Ok(())
}
 */

 /*
pub fn generate_cartridge(output_filename: String) -> std::io::Result<()> {
    // Make a cartridge full of random voices
    let cartridge = make_random_cartridge();
    let cartridge_data = cartridge.to_packed_bytes();
    let cartridge_checksum = voice_checksum(&cartridge_data);
    //debug!("cartridge checksum = {:02X}h", cartridge_checksum);

    let mut payload = vec![
        0x00,   // MIDI channel 1
        0x09,   // format = 9 (32 voices)
        0x20,   // byte count MSB
        0x00,   // byte count LSB
    ];
    payload.extend(cartridge_data);
    payload.push(cartridge_checksum);

    let message = Message::ManufacturerSpecific {
        manufacturer: Manufacturer::Standard(0x43),
        payload
    };

    if output_filename == "" {
        let now = SystemTime::now();
        let epoch_now = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let filename = format!("cartridge-{:?}.syx", epoch_now.as_secs());
        {
            let mut file = File::create(filename)?;
            file.write_all(&message.to_bytes())?;
        }
    }
    else {
        let mut file = File::create(output_filename)?;
        file.write_all(&message.to_bytes())?;
    }

    Ok(())
}
 */

pub fn list_cartridge(filedata: &[u8]) {
    match Message::from_bytes(&filedata) {
        Ok(Message::ManufacturerSpecific { manufacturer: _, payload }) => {
            println!("message payload length = {}", payload.len());
            let cartridge_data = &payload[Header::DATA_SIZE..];
            println!("cartridge data length = {}", cartridge_data.len());
            let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();

            for voice in cartridge.voices.iter() {
                println!("{}", voice.name);
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

            for voice in cartridge.voices.iter() {
                println!("{}", voice.name);
            }
        },
        _ => {
            eprintln!("invalid SysEx message");
        }
    }
}

/*
/// Dumps the contents of the file. It is assumed to be either a single voice,
/// or a cartridge of 32 voices, based on the format byte at offset 3.
/// Voice number is 1...32 for cartridges, ignored for single voices.
pub fn dump(input_filename: String, voice_number: u32) -> std::io::Result<()> {
    let mut file = File::open(input_filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data[3] == 0x00 {  // single voice
        let voice = Voice::from_bytes(data[6..].to_vec());
        println!("{}", voice);
    }
    else if data[3] == 0x09 { // cartridge of 32 voices
        let cartridge = Cartridge::from_packed_bytes(data[6..].to_vec());
        if voice_number == 0 {
            for voice in cartridge.voices.iter() {
                println!("{}", voice);
            }
        }
        else {
            let voice_number = voice_number - 1;
            println!("{}", cartridge.voices[voice_number as usize]);
        }
    }

    Ok(())
}
 */
