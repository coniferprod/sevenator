use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::str;

use dbg_hex::dbg_hex;

use sevenate::Ranged;
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
use sevenate::dx7::lfo::Lfo;
use sevenate::dx7::envelope::Envelope;
use sevenate::dx7::operator::{KeyboardLevelScaling, Operator};

trait ToXml {
    fn to_xml(&self) -> XMLElement;
    fn to_xml_named(&self, name: &str) -> XMLElement;
}

impl ToXml for Cartridge {
    fn to_xml(&self) -> XMLElement {
        self.to_xml_named("cartridge")
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut e = XMLElement::new(name);

        let mut voices_element = XMLElement::new("voices");

        for voice in &self.voices {
            voices_element.add_child(voice.to_xml()).unwrap();
        }

        e.add_child(voices_element);
        e
    }
}

impl ToXml for Voice {
    fn to_xml(&self) -> XMLElement {
        self.to_xml_named("voice")
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut e = XMLElement::new(name);

        e.add_attribute("name", &self.name.value());
        e.add_attribute("algorithm", &self.alg.value().to_string());
        e.add_attribute("transpose", &self.transpose.value().to_string());
        e.add_attribute("feedback", &self.feedback.value().to_string());
        e.add_attribute("oscillatorSync", &self.osc_sync.to_string());
        e.add_attribute("pitchModulationSensitivity", &self.pitch_mod_sens.value().to_string());

        e.add_child(self.peg.to_xml_named("peg")).unwrap();
        e.add_child(self.lfo.to_xml()).unwrap();

        let mut op_e = XMLElement::new("operators");
        for op in self.operators {
            op_e.add_child(op.to_xml()).unwrap();
        }
        e.add_child(op_e);

        e
    }
}

impl ToXml for Lfo {
    fn to_xml(&self) -> XMLElement {
        self.to_xml_named("lfo")
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut e = XMLElement::new(name);
    
        e.add_attribute("speed", &self.speed.value().to_string());
        e.add_attribute("delay", &self.delay.value().to_string());
        e.add_attribute("pmd", &self.pmd.value().to_string());
        e.add_attribute("amd", &self.amd.value().to_string());
        e.add_attribute("sync", &self.sync.to_string());
        e.add_attribute("wave", &self.waveform.to_string());
        //e.add_attribute("pms", &lfo.pms.value().to_string());
    
        e
    }    
}

impl ToXml for Envelope {
    fn to_xml(&self) -> XMLElement {
        unimplemented!();
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut e = XMLElement::new(name);

        let mut rates_element = XMLElement::new("rates");
        let mut rates_string = String::new();
        let mut count = 0;
        for r in self.rates.iter() {
            rates_string.push_str(&r.to_string());
            count += 1;
            if count < 4 {
                rates_string.push_str(" ");
            }
        }

        rates_element.add_text(rates_string).unwrap();
        e.add_child(rates_element);

        let mut levels_element = XMLElement::new("levels");
        let mut levels_string = String::new();
        let mut count = 0;
        for level in self.levels.iter() {
            levels_string.push_str(&level.to_string());
            count += 1;
            if count < 4 {
                levels_string.push_str(" ");
            }
        }
        levels_element.add_text(levels_string).unwrap();
        e.add_child(levels_element);

        e
    }
}

impl ToXml for Operator {
    fn to_xml(&self) -> XMLElement {
        self.to_xml_named("operator")
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut e = XMLElement::new(name);

        e.add_attribute("level", &self.output_level.value().to_string());
        e.add_attribute("mode", &self.mode.to_string());
        e.add_attribute("coarse", &self.coarse.value().to_string());
        e.add_attribute("fine", &self.fine.value().to_string());
        e.add_attribute("detune", &self.detune.value().to_string());
        e.add_attribute("amplitudeModulationSensitivity", &self.amp_mod_sens.value().to_string());
        e.add_attribute("keyVelocitySensitivity", &self.key_vel_sens.value().to_string());
        e.add_attribute("keyboardRateScaling", &self.kbd_rate_scaling.value().to_string());

        e.add_child(self.eg.to_xml_named("eg")).unwrap();
        e.add_child(self.kbd_level_scaling.to_xml()).unwrap();

        e
    }
}

impl ToXml for KeyboardLevelScaling {
    fn to_xml(&self) -> XMLElement {
        self.to_xml_named("keyboardLevelScaling")
    }

    fn to_xml_named(&self, name: &str) -> XMLElement {
        let mut elem = XMLElement::new(name);

        elem.add_attribute("breakpoint", &self.breakpoint.value().to_string());

        let mut depth_element = XMLElement::new("depth");
        depth_element.add_attribute("left", &self.left.depth.value().to_string());
        depth_element.add_attribute("right", &self.right.depth.value().to_string());
        elem.add_child(depth_element);

        let mut curve_element = XMLElement::new("curve");
        curve_element.add_attribute("left", &self.left.curve.to_string());
        curve_element.add_attribute("right", &self.right.curve.to_string());
        elem.add_child(curve_element);

        elem
    }
}

pub fn run_make_xml(input_path: &PathBuf, output_path: &PathBuf) {
    let Some(buffer) = read_file(&input_path) else {
        eprintln!("Unable to read from {}", input_path.display());
        return;
    };

    let Ok(Message::ManufacturerSpecific { manufacturer: _, payload }) 
            = Message::from_bytes(&buffer) else {
        eprintln!("Error in message");
        return;
    };

    let Ok(header) = Header::parse(&payload) else {
        eprintln!("Error parsing header");
        return;
    };

    println!("Header = {}", header);

    let data = &payload[Header::DATA_SIZE .. payload.len() - 1];
    //dbg_hex!(data);

    match header.format {
        Format::Voice => {
            eprintln!("Don't know how to make voice XML, only cartridge");
            return;
        },
        Format::Cartridge => {
            println!("data length = {}", data.len());

            let Ok(cartridge) = Cartridge::parse(&data) else {
                eprintln!("Error parsing cartridge data");
                return;
            };

            let mut xml = XMLBuilder::new()
                .version(XMLVersion::XML1_1)
                .encoding("UTF-8".into())
                .build();
            
            let cartridge_element = cartridge.to_xml();
            xml.set_root_element(cartridge_element);
            
            let mut writer: Vec<u8> = Vec::new();
            xml.generate(&mut writer).unwrap();
        
            let output = File::create(output_path);
            output
                .expect("to create output file")
                .write_all(&writer)
                .expect("to write XML data into the output file");
            
        }
    }    
}

use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};
use sevenate::dx7::voice::VoiceName;
use sevenate::dx7::{Algorithm, Transpose, Depth};

pub fn run_make_syx(input_path: &PathBuf, output_path: &PathBuf) {
    let file = match File::open(input_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to read from {}, error = {}", 
                input_path.display(), err);
            return;    
        }
    };

    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    // Creating a default cartridge also creates 32 default voices. 
    let mut cartridge: Cartridge = Default::default();
    let mut voice_index: usize = 0;  // index of next voice to save in the cartridge
    let mut voice: Voice = Default::default();
    let mut operator_index: usize = 0;  // index of operator to save in voice
    let mut operator: Operator = Operator::new();
    for element in parser {
        match element {
            Ok(XmlEvent::StartElement { name, attributes, namespace }) => {
                println!("start {}", name);

                match name.local_name.as_str() {
                    "cartridge" => {},
                    "voice" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "name" => {
                                    voice.name = VoiceName::from_string(attr.value);
                                },
                                "algorithm" => {
                                    voice.alg = Algorithm::new(attr.value.parse().expect("valid algorithm"));
                                },
                                "transpose" => {
                                    voice.transpose = Transpose::new(attr.value.parse().expect("valid transpose"));
                                },
                                "feedback" => {
                                    voice.feedback = Depth::new(attr.value.parse().expect("valid feedback"));
                                },
                                "oscillatorSync" => {
                                    voice.osc_sync = attr.value.parse().expect("valid boolean");
                                },
                                "pitchModulationSensitivity" => {
                                    voice.pitch_mod_sens = Depth::new(attr.value.parse().expect("valid PMS"));
                                }
                                _ => {}
                            }
                        }
                    },
                    "operator" => {

                    }
                    _ => {}
                };

            }
            Ok(XmlEvent::EndElement { name }) => {
                println!("end {}", name);

                match name.local_name.as_str() {
                    "cartridge" => {},
                    "voice" => {
                        cartridge.voices[voice_index] = voice.clone();
                        voice_index += 1;
                        operator_index = 0;  // voice added, reset operator count

                    },
                    "operator" => {
                        let mut ops = voice.operators;
                        ops[operator_index] = operator;
                        operator_index += 1;
                    },
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }

    // Print the results for debugging:
    for voice in cartridge.voices {
        println!("{} {}", voice.name.value(), voice.alg.value());
        for op in voice.operators {
            println!("{}", op);
        }
        println!();
    }
}
