use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::str;

use syxpack::{
    Message,
    Manufacturer
};

use sevenate::dx7::voice::Voice;
use sevenate::dx7::cartridge::Cartridge;

use sevenate::dx7::sysex::{
    Format,
    Header,
    SystemExclusiveData,
    checksum
};

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

pub fn run_list(path: &PathBuf) {
    if let Some(buffer) = read_file(&path) {
        match Message::from_bytes(&buffer) {
            Ok(Message::ManufacturerSpecific { manufacturer, payload }) => {
                println!("Manufacturer = {}", manufacturer);
                println!("Payload = {} bytes", payload.len());

                match Header::parse(&payload) {
                    Ok(header) => {
                        println!("{}", header);

                        let data = &payload[4..payload.len() - 1];
                        println!("data length = {}", data.len());

                        match header.format {
                            Format::Voice => {
                                let name_data = &data[145..155];
                                let name = str::from_utf8(&name_data).expect("invalid UTF-8");
                                println!("{}", name);
                            },
                            Format::Cartridge => {
                                // For a cartridge, pick out the data for each
                                // of the 32 voices and extract the name.
                                // The voice data is packed into chunks of 128 bytes,
                                // and the name is in the last 10 bytes.
                                let mut voice_number = 1;
                                for voice_data in data.chunks(128) {
                                    let name_data = &voice_data[118..128];
                                    let name = str::from_utf8(&name_data).expect("invalid UTF-8");
                                    println!("{:2} {}", voice_number, name);
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
            Err(err) => {
                eprintln!("Error in message: {:?}", err);
            },
            _ => {
                eprintln!("Not a manufacturer-specific System Exclusive message");
            }
        }
    }
}

pub fn run_extract(path: &PathBuf) {
    if let Some(buffer) = read_file(&path) {
        match Message::from_bytes(&buffer) {
            Ok(Message::ManufacturerSpecific { manufacturer, payload }) => {
                println!("Manufacturer = {}", manufacturer);
                println!("Payload = {} bytes", payload.len());

                match Header::parse(&payload) {
                    Ok(mut header) => {
                        println!("{}", header);

                        let data = &payload[4..payload.len() - 1];
                        println!("data length = {}", data.len());
                        match header.format {
                            Format::Voice => {
                                println!("Not extracting an individual voice.")
                            },
                            Format::Cartridge => {
                                // For a cartridge, pick out the data for each
                                // of the 32 voices. Then unpack the voice data
                                // and write out a new file.
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
                                                Ok(_) => {

                                                },
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
            Err(err) => {
                eprintln!("Error in message: {:?}", err);
            },
            _ => {
                eprintln!("Not a manufacturer-specific System Exclusive message");
            }
        }
    }
}

/// Dumps the contents of the file. It is assumed to be either a single voice,
/// or a cartridge of 32 voices, based on the format byte at offset 3.
/// Voice number is 1...32 for cartridges, ignored for single voices.
pub fn run_dump(path: &PathBuf, number: &Option<u8>) {
    if let Some(buffer) = read_file(&path) {
        match Message::from_bytes(&buffer) {
            Ok(Message::ManufacturerSpecific { manufacturer, payload }) => {
                println!("Manufacturer = {}", manufacturer);
                println!("Payload = {} bytes", payload.len());

                match Header::parse(&payload) {
                    Ok(header) => {
                        println!("{}", header);

                        let data = &payload[4..payload.len() - 1];
                        println!("data length = {}", data.len());
                        match header.format {
                            Format::Voice => {
                                match Voice::parse(&payload) {
                                    Ok(voice) => {
                                        println!("{}", voice);
                                    },
                                    Err(e) => {
                                        eprintln!("{}", e);
                                    }

                                }
                            },
                            Format::Cartridge => {
                                match Cartridge::parse(&payload) {
                                    Ok(cartridge) => {
                                        if let Some(n) = number {
                                            println!("{}", cartridge.voices[(*n as usize) - 1]);
                                        }
                                        else {
                                            for voice in cartridge.voices.iter() {
                                                println!("{}", voice);
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("{}", e);
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            },
            Err(err) => {
                eprintln!("Error in message: {:?}", err);
            },
            _ => {
                eprintln!("Not a manufacturer-specific System Exclusive message");
            }
        }
    }
}

use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub fn run_make_xml(input_path: &PathBuf, output_path: &PathBuf) {
    let mut xml = XMLBuilder::new()
    .version(XMLVersion::XML1_1)
    .encoding("UTF-8".into())
    .build();

    let mut cartridge_element = XMLElement::new("cartridge");
    //cartridge.add_attribute("rooms", "2");

    let cartridge: Cartridge = Default::default();

    for voice in cartridge.voices {
        let mut voice_element = XMLElement::new("voice");
        //voice_element.add_attribute("number", &i.to_string());
        //voice_element.add_text(format!("This is room number {}", i)).unwrap();
        cartridge_element.add_child(voice_element).unwrap();
    }

    xml.set_root_element(cartridge_element);

    let mut writer: Vec<u8> = Vec::new();
    xml.generate(&mut writer).unwrap();

    let output = File::create(output_path);
    output.expect("to create output file").write_all(&writer).expect("to write XML data into the output file");

}
