use std::fs::File;
use std::io::prelude::*;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::convert::TryInto;
use log::{info, warn, error, debug};
use rand::Rng;
use num;
use bit::BitIndex;
use syxpack::{Message, Manufacturer, ManufacturerId};

type Byte = u8;
type ByteVector = Vec<u8>;

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

/// Base type for normal level (0...99)
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct UnsignedLevel(u8);

impl UnsignedLevel {
    fn range() -> Wrapper<u8> {
        Wrapper { start: 0, end: 99 }
    }

    pub fn new(value: u8) -> UnsignedLevel {
        let range = UnsignedLevel::range();
        UnsignedLevel(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for UnsignedLevel {
    fn from(value: u8) -> UnsignedLevel {
        UnsignedLevel::new(value)
    }
}

impl RandomValue for UnsignedLevel {
    type B = UnsignedLevel;
    type T = u8;

    fn random_value() -> Self::B {
        let mut rng = rand::thread_rng();
        let range = Self::B::range();
        Self::B::from(rng.gen_range(range.start..=range.end))
    }
}

// Semantic type aliases based on unsigned level:
type Level = UnsignedLevel;  // envelope level or operator output level
type Rate = UnsignedLevel; // envelope rate

/// Algorithm (1...32)
#[derive(Debug, Clone, Copy)]
pub struct Algorithm(u8);

impl Algorithm {
    fn range() -> Wrapper<u8> {
        Wrapper { start: 1, end: 32 }
    }

    pub fn new(value: u8) -> Algorithm {
        let range = Algorithm::range();
        Algorithm(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> u8 {
        (self.0 + 64) as u8
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Algorithm {
    fn from(value: u8) -> Algorithm {
        Algorithm::new(value)
    }
}

impl RandomValue for Algorithm {
    type B = Algorithm;
    type T = u8;

    fn random_value() -> Self::B {
        let mut rng = rand::thread_rng();
        let range = Self::B::range();
        Self::B::from(rng.gen_range(range.start..=range.end))
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}:\n{}", self.value(), ALGORITHM_DIAGRAMS[(self.value() as usize) - 1])
    }
}

/// Detune (-7...+7), represented in SysEx as 0...14.
#[derive(Debug, Clone, Copy)]
pub struct Detune(i8);

impl Detune {
    fn range() -> Wrapper<i8> {
        Wrapper { start: -7, end: 7 }
    }

    pub fn new(value: i8) -> Detune {
        let range = Detune::range();
        Detune(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> u8 {
        (self.0 + 7) as u8
    }

    pub fn value(&self) -> i8 {
        self.0
    }
}

impl From<u8> for Detune {
    fn from(value: u8) -> Detune {
        Detune::new(value as i8)
    }
}

impl From<i8> for Detune {
    fn from(value: i8) -> Detune {
        Detune::new(value)
    }
}

impl From<i32> for Detune {
    fn from(value: i32) -> Detune {
        Detune::new(value as i8)
    }
}

impl RandomValue for Detune {
    type B = Detune;
    type T = i32;

    fn random_value() -> Self::B {
        let mut rng = rand::thread_rng();
        let range = Self::B::range();
        Self::B::from(rng.gen_range(range.start..=range.end))
    }
}

/// Coarse (0...31).
#[derive(Debug, Clone, Copy)]
pub struct Coarse(u8);

impl Coarse {
    fn range() -> Wrapper<u8> {
        Wrapper { start: 0, end: 31 }
    }

    pub fn new(value: u8) -> Coarse {
        let range = Coarse::range();
        Coarse(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> u8 {
        self.0
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Coarse {
    fn from(value: u8) -> Coarse {
        Coarse::new(value)
    }
}

/// Depth (0...7) for sensitivity values.
#[derive(Debug, Clone, Copy)]
pub struct Depth(u8);

impl Depth {
    fn range() -> Wrapper<u8> {
        Wrapper { start: 0, end: 7 }
    }

    pub fn new(value: u8) -> Depth {
        let range = Depth::range();
        Depth(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> u8 {
        self.0
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Depth {
    fn from(value: u8) -> Depth {
        Depth::new(value)
    }
}

impl RandomValue for Depth {
    type B = Depth;
    type T = u8;

    fn random_value() -> Self::B {
        let mut rng = rand::thread_rng();
        let range = Self::B::range();
        Self::B::from(rng.gen_range(range.start..=range.end))
    }
}

// Makes a new voice based on the "BRASS1" settings in the DX7 manual.
fn make_brass1() -> Voice {
    let kbd_level_scaling = KeyboardLevelScaling {
        breakpoint: 60 - 21,
        left_depth: 0,
        right_depth: 0,
        left_curve: ScalingCurve::lin_pos(),
        right_curve: ScalingCurve::lin_pos(),
    };

    // Make one operator and then specify the differences to the others.
    let op = Operator {
        key_vel_sens: Depth::from(2),
        ..Operator::new()
    };

    let op6 = Operator {
        eg: Envelope::new_rate_level(Rates(49, 99, 28, 68), Levels(98, 98, 91, 0)),
        kbd_level_scaling: KeyboardLevelScaling {
            left_depth: 54,
            right_depth: 50,
            left_curve: ScalingCurve::exp_neg(),
            right_curve: ScalingCurve::exp_neg(),
            ..kbd_level_scaling
        },
        kbd_rate_scaling: Depth::from(4),
        output_level: Level::from(82),
        ..op
    };

    let op5 = Operator {
        eg: Envelope::new_rate_level(Rates(77, 36, 41, 71), Levels(99, 98, 98, 0)),
        kbd_level_scaling,
        output_level: Level::from(98),
        detune: Detune::from(1),
        ..op
    };

    let op4 = Operator {
        eg: op5.eg.clone(),
        kbd_level_scaling,
        output_level: Level::from(99),
        ..op
    };

    let op3 = Operator {
        eg: Envelope::new_rate_level(Rates(77, 76, 82, 71), Levels(99, 98, 98, 0)),
        kbd_level_scaling,
        output_level: Level::from(99),
        detune: Detune::from(-2),
        ..op
    };

    let op2 = Operator {
        eg: Envelope::new_rate_level(Rates(62, 51, 29, 71), Levels(82, 95, 96, 0)),
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 48 - 21,
            left_depth: 0,
            right_depth: 7,
            left_curve: ScalingCurve::lin_pos(),
            right_curve: ScalingCurve::exp_neg(),
        },
        key_vel_sens: Depth::from(0),
        output_level: Level::from(86),
        coarse: Coarse::from(0),
        detune: Detune::from(7),
        ..op
    };

    let op1 = Operator {
        eg: Envelope::new_rate_level(Rates(72, 76, 99, 71), Levels(99, 88, 96, 0)),
        kbd_level_scaling: KeyboardLevelScaling {
            right_depth: 14,
            ..kbd_level_scaling
        },
        key_vel_sens: Depth::from(0),
        output_level: Level::from(98),
        coarse: Coarse::from(0),
        detune: Detune::from(7),
        ..op
    };

    Voice {
        op1, op2, op3, op4, op5, op6,
        peg: Envelope::new_rate_level(Rates(84, 95, 95, 60), Levels(50, 50, 50, 50)),
        alg: Algorithm::from(22),
        feedback: Depth::from(7),
        osc_sync: true,
        lfo: Lfo {
            speed: Level::from(37),
            delay: Level::from(0),
            pmd: Level::from(5),
            amd: Level::from(0),
            sync: false,
            wave: LfoWaveform::Sine,
            pitch_mod_sens: Depth::from(3),
        },
        transpose: Transpose::from(0),
        name: "BRASS   1 ".to_string(),
        op_flags: [true, true, true, true, true, true],
    }
}

// Makes an initialized voice. The defaults are as described in
// Howard Massey's "The Complete DX7", Appendix B.
fn make_init_voice() -> Voice {
    let init_eg = Envelope::new();

    let init_op1 = Operator {
        eg: init_eg.clone(),
        kbd_level_scaling: KeyboardLevelScaling::new(),
        kbd_rate_scaling: Depth::from(0),
        amp_mod_sens: 0,
        key_vel_sens: Depth::from(0),
        output_level: Level::from(99),
        mode: OperatorMode::Ratio,
        coarse: Coarse::from(1),
        fine: Level::from(0),
        detune: Detune::from(0),
    };

    // Operators 2...6 are identical to operator 1 except they
    // have their output level set to zero.
    let init_op_rest = Operator {
        output_level: Level::from(0),
        ..init_op1
    };

    Voice {
        op1: init_op1.clone(),
        op2: init_op_rest.clone(),
        op3: init_op_rest.clone(),
        op4: init_op_rest.clone(),
        op5: init_op_rest.clone(),
        op6: init_op_rest.clone(),
        peg: Envelope::new_rate_level(Rates(99, 99, 99, 99), Levels(50, 50, 50, 50)),
        alg: Algorithm::from(1),
        feedback: Depth::from(0),
        osc_sync: true, // osc key sync = on
        lfo: Lfo {
            speed: Level::from(35),
            delay: Level::from(0),
            pmd: Level::from(0),
            amd: Level::from(0),
            sync: true,
            wave: LfoWaveform::Triangle,
            pitch_mod_sens: Depth::from(3),
        },
        transpose: Transpose::from(0),
        name: "INIT VOICE".to_string(),
        op_flags: [true, true, true, true, true, true],  // all operators ON
    }
}

// Makes a random envelope generator.
fn make_random_eg() -> Envelope {
    Envelope {
        rate1: Rate::random_value(),
        rate2: Rate::random_value(),
        rate3: Rate::random_value(),
        rate4: Rate::random_value(),
        level1: Level::random_value(),
        level2: Level::random_value(),
        level3: Level::random_value(),
        level4: Level::random_value(),
    }
}

// Makes a random operator.
fn make_random_operator() -> Operator {
    Operator {
        eg: make_random_eg(),
        kbd_level_scaling: KeyboardLevelScaling::new(),
        kbd_rate_scaling: Depth::from(0),
        amp_mod_sens: 0,
        key_vel_sens: Depth::from(0),
        output_level: Level::random_value(),
        mode: OperatorMode::Ratio,
        coarse: Coarse::from(1),
        fine: Level::from(0),
        detune: Detune::from(0),
    }
}

// Makes a random voice.
fn make_random_voice() -> Voice {
    Voice {
        op1: make_random_operator(),
        op2: make_random_operator(),
        op3: make_random_operator(),
        op4: make_random_operator(),
        op5: make_random_operator(),
        op6: make_random_operator(),
        peg: Envelope::new_rate_level(Rates(99, 99, 99, 99), Levels(50, 50, 50, 50)),
        alg: Algorithm::random_value(),
        feedback: Depth::from(0),
        osc_sync: true, // osc key sync = on
        lfo: Lfo::new_random(),
        transpose: Transpose::from(0),
        name: "RNDM VOICE".to_string(),
        op_flags: [true, true, true, true, true, true],  // all operators ON
    }
}

const VOICE_COUNT: usize = 32;

#[derive(Debug)]
struct Cartridge {
    voices: Vec<Voice>,
}

impl Default for Cartridge {
    fn default() -> Self {
        Cartridge {
            voices: vec![Default::default(); VOICE_COUNT],
        }
    }
}

impl SystemExclusiveData for Cartridge {
    fn from_bytes(data: ByteVector) -> Self {
        // Delegate to the packed bytes constructor,
        // since the cartridge data is always in packed format.
        warn!("from_bytes() called for Cartridge, delegating to from_packed_bytes()");
        Cartridge::from_packed_bytes(data)
    }

    fn from_packed_bytes(data: ByteVector) -> Self {
        let mut offset = 0;
        let mut voices = Vec::<Voice>::new();
        for _ in 0..VOICE_COUNT {
            voices.push(Voice::from_packed_bytes(data[offset..offset + 128].to_vec()));
            offset += 128;
        }
        Cartridge { voices }
    }

    fn to_bytes(&self) -> ByteVector {
        // Delegate to the to_packed_bytes() method,
        // since the cartridge data is always in packed format.
        warn!("to_bytes() called for Cartridge, delegating to to_packed_bytes()");
        self.to_packed_bytes()
    }

    fn to_packed_bytes(&self) -> ByteVector {
        let mut data: ByteVector = Vec::new();

        for (index, voice) in self.voices.iter().enumerate() {
            let voice_data = voice.to_packed_bytes();
            debug!("Voice #{} packed data length = {} bytes", index, voice_data.len());
            data.extend(voice_data);
        }

        data
    }

    fn data_size(&self) -> usize { 4096 }
}

// Makes a cartridge filled with random voices.
fn make_random_cartridge() -> Cartridge {
    let mut voices: Vec<Voice> = Vec::new();
    for i in 0..VOICE_COUNT {
        voices.push(make_random_voice());
    }
    Cartridge { voices }
}

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

    Message::new_manufacturer(
        Manufacturer::from_id(ManufacturerId::Standard(0x43)),
        payload)
    .to_bytes()
}

pub fn generate_random_voice(output_filename: String) -> std::io::Result<()> {
    let output = generate_voice(make_random_voice());

    {
        let mut file = File::create(output_filename)?;
        file.write_all(&output)?;
    }

    Ok(())
}

pub fn generate_init_voice(output_filename: String) -> std::io::Result<()> {
    let output = generate_voice(make_init_voice());

    {
        let mut file = File::create(output_filename)?;
        file.write_all(&output)?;
    }

    Ok(())
}

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

    let message = Message::new_manufacturer(
        Manufacturer::from_id(ManufacturerId::Standard(0x43)),
        payload);

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

/// Runs the cartridge generation routine.
pub fn run() -> std::io::Result<()> {
    // Show the ROM1 cartridge contents (just for testing):
    let rom1a_data: [u8; 4096] = include!("rom1asyx.in");
    let rom1a_cartridge = Cartridge::from_packed_bytes(rom1a_data.to_vec());
    for voice in rom1a_cartridge.voices.iter() {
        println!("{}", voice.name);
    }

    Ok(())

    // Get the default voice with `Voice::new()`.
    // The `make_init_voice()` function makes exactly the original init voice.
    // These should be more or less the same.
    //let cartridge: Cartridge = Default::default();
}

/// Parsing and generating MIDI System Exclusive data.
pub trait SystemExclusiveData {
    fn from_bytes(data: ByteVector) -> Self;
    fn from_packed_bytes(data: ByteVector) -> Self;
    fn to_bytes(&self) -> ByteVector;
    fn to_packed_bytes(&self) -> ByteVector { vec![] }
    fn data_size(&self) -> usize { 0 }
}

// Conveniences for initializing EGs.
pub struct Rates(u8, u8, u8, u8);
pub struct Levels(u8, u8, u8, u8);

/// Envelope generator.
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    pub rate1: Rate,
    pub rate2: Rate,
    pub rate3: Rate,
    pub rate4: Rate,
    pub level1: Level,
    pub level2: Level,
    pub level3: Level,
    pub level4: Level,
}

impl Envelope {
    /// Creates a new EG with the DX7 voice defaults.
    pub fn new() -> Self {
        Envelope::new_rate_level(Rates(99, 99, 99, 99), Levels(99, 99, 99, 0))
    }

    /// Makes a new EG with rates and levels.
    pub fn new_rate_level(rates: Rates, levels: Levels) -> Self {
        Self {
            rate1: Rate::from(rates.0),
            rate2: Rate::from(rates.1),
            rate3: Rate::from(rates.2),
            rate4: Rate::from(rates.3),
            level1: Level::from(levels.0),
            level2: Level::from(levels.1),
            level3: Level::from(levels.2),
            level4: Level::from(levels.3),
        }
    }

    /*
    From the Yamaha DX7 Operation Manual (p. 51):
    "You can simulate an ADSR if you set the envelope as follows:
    L1=99, L2=99, L4=0, and R2=99.
    With these settings, then R1 becomes Attack time, R3 is Decay
    time, L3 is Sustain level, and R4 is Release time."
    */

    /// Makes a new ADSR-style envelope.
    pub fn adsr(attack: u8, decay: u8, sustain: u8, release: u8) -> Self {
        Envelope::new_rate_level(
            Rates(attack, 99, decay, release),
            Levels(99, 99, sustain, 0)
        )
    }

    /// Makes a new EG with random rates and levels.
    pub fn new_random() -> Self {
        Self {
            rate1: Rate::random_value(),
            rate2: Rate::random_value(),
            rate3: Rate::random_value(),
            rate4: Rate::random_value(),
            level1: Level::random_value(),
            level2: Level::random_value(),
            level3: Level::random_value(),
            level4: Level::random_value(),
        }
    }
}

impl fmt::Display for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R1={} L1={} R2={} L2={} R3={} L3={} R4={} L4={}",
            self.rate1.value(), self.level1.value(), self.rate2.value(), self.level2.value(),
            self.rate3.value(), self.level3.value(), self.rate4.value(), self.level4.value())
    }
}

impl SystemExclusiveData for Envelope {
    /// Makes an envelope generator from relevant SysEx message bytes.
    fn from_bytes(data: ByteVector) -> Self {
        Envelope::new_rate_level(
            Rates(data[0], data[1], data[2], data[3]),
            Levels(data[4], data[5], data[6], data[7]),
        )
    }

    /// Makes an envelope generator from packed SysEx message bytes.
    fn from_packed_bytes(data: ByteVector) -> Self {
        Envelope::from_bytes(data)
    }

    fn to_packed_bytes(&self) -> ByteVector {
        self.to_bytes()
    }

    /// Gets the SysEx bytes of this EG.
    fn to_bytes(&self) -> ByteVector {
        vec![
            self.rate1.as_byte(), self.rate2.as_byte(), self.rate3.as_byte(), self.rate4.as_byte(),
            self.level1.as_byte(), self.level2.as_byte(), self.level3.as_byte(), self.level4.as_byte()
        ]
    }
}

/// Scaling curve style.
#[derive(Debug, Copy, Clone)]
pub enum CurveStyle {
    Linear,
    Exponential
}

impl fmt::Display for CurveStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CurveStyle::Linear => write!(f, "LIN"),
            CurveStyle::Exponential => write!(f, "EXP"),
        }
    }
}

/// Scaling curve settings.
#[derive(Debug, Clone, Copy)]
pub struct ScalingCurve {
    pub curve: CurveStyle,
    pub positive: bool,  // true if positive, false if negative
}

impl ScalingCurve {
    /// Makes a linear positive scaling curve.
    pub fn lin_pos() -> Self {
        ScalingCurve { curve: CurveStyle::Linear, positive: true }
    }

    /// Makes a linear negative scaling curve.
    pub fn lin_neg() -> Self {
        ScalingCurve { curve: CurveStyle::Linear, positive: false }
    }

    /// Makes an exponential positive scaling curve.
    pub fn exp_pos() -> Self {
        ScalingCurve { curve: CurveStyle::Exponential, positive: true }
    }

    /// Makes an exponential negative scaling curve.
    pub fn exp_neg() -> Self {
        ScalingCurve { curve: CurveStyle::Exponential, positive: false }
    }

    /// Makes a scaling curve from a System Exclusive data byte.
    pub fn from_byte(b: Byte) -> Self {
        match b {
            0 => ScalingCurve::lin_neg(),
            1 => ScalingCurve::exp_neg(),
            2 => ScalingCurve::exp_pos(),
            3 => ScalingCurve::lin_pos(),
            _ => ScalingCurve::lin_pos(),
        }
    }

    /// Gets the SysEx bytes for this scaling curve.
    pub fn to_bytes(&self) -> Byte {
        match self {
            ScalingCurve { curve: CurveStyle::Linear, positive: true } => 3,
            ScalingCurve { curve: CurveStyle::Linear, positive: false } => 0,
            ScalingCurve { curve: CurveStyle::Exponential, positive: true } => 2,
            ScalingCurve { curve: CurveStyle::Exponential, positive: false } => 1,
        }
    }
}

impl fmt::Display for ScalingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.curve, if self.positive { "+" } else { "-" })
    }
}

/// Keyboard level scaling.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardLevelScaling {
    pub breakpoint: u8, // 0 ~ 99 (A-1 ~ C8)
    pub left_depth: u8,
    pub right_depth: u8,
    pub left_curve: ScalingCurve,  // 0 ~ 3
    pub right_curve: ScalingCurve, // 0 ~ 3
}

/*
Usually MIDI note A0 (for Yamaha; A-1 for others) is 21.
The breakpoint is scaled to 0~99, so you need to subtract 21 to make it
zero-based. So, middle C (C3 for Yamaha) would be 60, but here it is 60 - 21 = 39.
*/

impl KeyboardLevelScaling {
    /// Creates new keyboard level scaling settings with DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            breakpoint: 60 - 21,  // Yamaha C3 is 60 - 21 = 39
            left_depth:  0,
            right_depth: 0,
            left_curve: ScalingCurve::lin_neg(),
            right_curve: ScalingCurve::lin_neg(),
        }
    }
}

impl SystemExclusiveData for KeyboardLevelScaling {
    /// Makes new keyboard level scaling settings from SysEx bytes.
    fn from_bytes(data: ByteVector) -> Self {
        Self {
            breakpoint: data[0],
            left_depth: data[1],
            right_depth: data[2],
            left_curve: ScalingCurve::from_byte(data[3]),
            right_curve: ScalingCurve::from_byte(data[4]),
        }
    }

    /// Makes new keyboard level scaling settings from packed SysEx bytes.
    fn from_packed_bytes(data: ByteVector) -> Self {
        Self {
            breakpoint: data[0],
            left_depth: data[1],
            right_depth: data[2],
            left_curve: ScalingCurve::from_byte(data[3] >> 4),
            right_curve: ScalingCurve::from_byte(data[3] & 0x0f),
        }
    }

    /// Gets the SysEx bytes representing this set of parameters.
    fn to_bytes(&self) -> ByteVector {
        vec![
            self.breakpoint,
            self.left_depth,
            self.right_depth,
            self.left_curve.to_bytes(),
            self.right_curve.to_bytes(),
        ]
    }

    /// Gets the packed SysEx bytes representing this set of parameters.
    fn to_packed_bytes(&self) -> ByteVector {
        vec![
            self.breakpoint,
            self.left_depth,
            self.right_depth,
            self.left_curve.to_bytes() | (self.right_curve.to_bytes() << 2),
        ]
    }
}

impl fmt::Display for KeyboardLevelScaling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "breakpoint = {}, left depth = {}, right depth = {}, left curve = {}, right curve = {}",
            self.breakpoint, self.left_depth, self.right_depth, self.left_curve, self.right_curve)
    }
}

/// Operator mode.
#[derive(Debug, Copy, Clone)]
pub enum OperatorMode {
    Ratio,
    Fixed,
}

/// Operator.
#[derive(Debug, Clone, Copy)]
pub struct Operator {
    pub eg: Envelope,
    pub kbd_level_scaling: KeyboardLevelScaling,
    pub kbd_rate_scaling: Depth, // 0 ~ 7
    pub amp_mod_sens: u8,  // 0 ~ 3
    pub key_vel_sens: Depth,  // 0 ~ 7
    pub output_level: Level,
    pub mode: OperatorMode,
    pub coarse: Coarse,  // 0 ~ 31
    pub fine: Level,  // 0 ~ 99
    pub detune: Detune,   // -7 ~ 7
}

impl Operator {
    /// Creates a new operator and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            eg: Envelope::new(),
            kbd_level_scaling: KeyboardLevelScaling::new(),
            kbd_rate_scaling: Depth::from(0),
            amp_mod_sens: 0,
            key_vel_sens: Depth::from(0),
            output_level: Level::from(0),
            mode: OperatorMode::Ratio,
            coarse: Coarse::from(1),
            fine: Level::from(0),  // TODO: voice init for fine is "1.00 for all operators", should this be 0 or 1?
            detune: Detune::from(0),
        }
    }
}

impl SystemExclusiveData for Operator {
    /// Makes a new operator from SysEx bytes.
    fn from_bytes(data: ByteVector) -> Self {
        let eg_bytes = &data[0..8];
        let level_scaling_bytes = &data[8..13];
        Self {
            eg: Envelope::from_bytes(eg_bytes.to_vec()),
            kbd_level_scaling: KeyboardLevelScaling::from_bytes(level_scaling_bytes.to_vec()),
            kbd_rate_scaling: Depth::from(data[13]),
            amp_mod_sens: data[14],
            key_vel_sens: Depth::from(data[15]),
            output_level: Level::from(data[16]),
            mode: if data[17] == 0b1 { OperatorMode::Fixed } else { OperatorMode::Ratio },
            coarse: Coarse::from(data[18]),
            fine: Level::from(data[19]),
            detune: Detune::from(data[20]),
        }
    }

    /// Makes a new operator from packed SysEx bytes.
    fn from_packed_bytes(data: ByteVector) -> Self {
        Operator {
            eg: Envelope::from_packed_bytes(data[0..8].to_vec()),
            kbd_level_scaling: KeyboardLevelScaling::from_packed_bytes(data[8..12].to_vec()),
            kbd_rate_scaling: Depth::from(data[12].bit_range(0..3)),
            amp_mod_sens: data[13].bit_range(0..2),
            key_vel_sens: Depth::from(data[13].bit_range(2..5)),
            output_level: Level::from(data[14]),
            mode: if data[15].bit(0) { OperatorMode::Fixed } else { OperatorMode::Ratio },
            coarse: Coarse::from(data[15].bit_range(1..6)),
            fine: Level::from(data[16]),
            detune: Detune::from(data[12].bit_range(3..7)),
        }
    }

    /// Gets the SysEx bytes representing the operator.
    fn to_bytes(&self) -> ByteVector {
        let mut data: ByteVector = Vec::new();
        data.extend(self.eg.to_bytes());
        data.extend(self.kbd_level_scaling.to_bytes());
        data.push(self.kbd_rate_scaling.as_byte());
        data.push(self.amp_mod_sens);
        data.push(self.key_vel_sens.as_byte());
        data.push(self.output_level.as_byte());
        data.push(self.mode as u8);
        data.push(self.coarse.as_byte());
        data.push(self.fine.as_byte());
        data.push(self.detune.as_byte()); // 0 = detune -7
        assert_eq!(data.len(), 21);
        data
    }

    /// Gets the packed SysEx bytes representing the operator.
    fn to_packed_bytes(&self) -> ByteVector {
        let mut data: ByteVector = Vec::new();

        let eg_data = self.eg.to_packed_bytes();
        debug!("  EG: {} bytes, {:?}", eg_data.len(), eg_data);
        data.extend(eg_data);

        let kls_data = self.kbd_level_scaling.to_packed_bytes();
        debug!("  KLS: {} bytes, {:?}", kls_data.len(), kls_data);
        data.extend(kls_data);

        let detune = self.detune.as_byte();
        let byte12 = self.kbd_rate_scaling.as_byte() | (detune << 3);
        debug!("  KBD RATE SCALING = {:?} DETUNE = {:?} b12: {:#08b}", self.kbd_rate_scaling, self.detune, byte12);
        data.push(byte12);

        let byte13 = self.amp_mod_sens | (self.key_vel_sens.as_byte() << 2);
        debug!("  b13: {:#08b}", byte12);
        data.push(byte13);

        let output_level = self.output_level.value();
        debug!("  OL:  {:#08b}", output_level);
        data.push(self.output_level.as_byte());

        let byte15 = self.mode as u8 | (self.coarse.as_byte() << 1);
        debug!("  b15: {:#08b}", byte15);
        data.push(byte15);

        let fine = self.fine.value();
        debug!("  FF:  {:#08b}", fine);
        data.push(self.fine.as_byte());

        data
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"EG: {}
Kbd level scaling: {}, Kbd rate scaling: {}
Amp mod sens = {}, Key vel sens = {}
Level = {}, Mode = {:?}
Coarse = {}, Fine = {}, Detune = {}
",
            self.eg, self.kbd_level_scaling, self.kbd_rate_scaling.value(),
            self.amp_mod_sens, self.key_vel_sens.value(), self.output_level.value(), self.mode,
            self.coarse.value(), self.fine.value(), self.detune.value())
    }
}

/// LFO waveform.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum LfoWaveform {
    Triangle,
    SawDown,
    SawUp,
    Square,
    Sine,
    SampleAndHold,
}

/// LFO.
#[derive(Debug, Clone, Copy)]
pub struct Lfo {
    pub speed: Level,  // 0 ~ 99
    pub delay: Level,  // 0 ~ 99
    pub pmd: Level,    // 0 ~ 99
    pub amd: Level,    // 0 ~ 99
    pub sync: bool,
    pub wave: LfoWaveform,
    pub pitch_mod_sens: Depth,  // 0 ~ 7
}

impl Lfo {
    /// Makes a new LFO initialized with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            speed: Level::from(35),
            delay: Level::from(0),
            pmd: Level::from(0),
            amd: Level::from(0),
            sync: true,
            wave: LfoWaveform::Triangle,
            pitch_mod_sens: Depth::from(0),
        }
    }

    /// Makes a new LFO with random settings.
    pub fn new_random() -> Self {
        Self {
            speed: Level::random_value(),
            delay: Level::random_value(),
            pmd: Level::random_value(),
            amd: Level::random_value(),
            sync: true,
            wave: LfoWaveform::Triangle,
            pitch_mod_sens: Depth::random_value(),
        }
    }
}

impl SystemExclusiveData for Lfo {
    fn from_bytes(data: ByteVector) -> Self {
        Lfo {
            speed: Level::from(data[0]),
            delay: Level::from(data[1]),
            pmd: Level::from(data[2]),
            amd: Level::from(data[3]),
            sync: if data[4] == 1u8 { true } else { false },
            wave: match data[5] {
                0 => LfoWaveform::Triangle,
                1 => LfoWaveform::SawDown,
                2 => LfoWaveform::SawUp,
                3 => LfoWaveform::Square,
                4 => LfoWaveform::Sine,
                5 => LfoWaveform::SampleAndHold,
                _ => {
                    warn!("LFO waveform out of range: {}, setting to TRI", data[5]);
                    LfoWaveform::Triangle
                }
            },
            pitch_mod_sens: Depth::from(data[6]),
        }
    }

    fn from_packed_bytes(data: ByteVector) -> Self {
        Lfo {
            speed: Level::from(data[0]),
            delay: Level::from(data[1]),
            pmd: Level::from(data[2]),
            amd: Level::from(data[3]),
            sync: if data[4].bit(0) { true } else { false },
            wave: match data[4].bit_range(1..4) {
                0 => LfoWaveform::Triangle,
                1 => LfoWaveform::SawDown,
                2 => LfoWaveform::SawUp,
                3 => LfoWaveform::Square,
                4 => LfoWaveform::Sine,
                5 => LfoWaveform::SampleAndHold,
                _ => {
                    warn!("LFO waveform out of range: {}, setting to TRI", data[4]);
                    LfoWaveform::Triangle
                }
            },
            pitch_mod_sens: Depth::from(data[4].bit_range(4..7)),
        }
    }

    fn to_bytes(&self) -> ByteVector {
        vec![
            self.speed.as_byte(),
            self.delay.as_byte(),
            self.pmd.as_byte(),
            self.amd.as_byte(),
            if self.sync { 1 } else { 0 },
            self.wave as u8,
            self.pitch_mod_sens.as_byte(),
        ]
    }

    fn to_packed_bytes(&self) -> ByteVector {
        let mut b116: Byte = if self.sync { 1 } else { 0 };
        b116.set_bit_range(1..4, self.wave as Byte);
        b116.set_bit_range(4..7, self.pitch_mod_sens.as_byte());

        vec![
            self.speed.as_byte(),
            self.delay.as_byte(),
            self.pmd.as_byte(),
            self.amd.as_byte(),
            b116,
        ]
    }
}

impl fmt::Display for Lfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "speed = {}, delay = {}, PMD = {}, AMD = {}, sync = {}, wave = {:?}, pitch mod sens = {}",
            self.speed.value(), self.delay.value(), self.pmd.value(), self.amd.value(),
            self.sync, self.wave, self.pitch_mod_sens.value())
    }
}

/// Key transpose in octaves (-2...2).
#[derive(Debug, Clone, Copy)]
pub struct Transpose(i8);

impl Transpose {
    fn range() -> Wrapper<i8> {
        Wrapper { start: -2, end: 2 }
    }

    pub fn new(value: i8) -> Transpose {
        let range = Transpose::range();
        Transpose(num::clamp(value, range.start, range.end))
    }

    pub fn as_byte(&self) -> Byte {
        // Convert to the range 0...48
        (self.0 + 2) as u8 * 12
    }

    pub fn value(&self) -> i8 {
        self.0
    }
}

impl From<i8> for Transpose {
    fn from(value: i8) -> Transpose {
        Transpose::new(value)
    }
}

impl Transpose {
    /// Makes a key transpose from a System Exclusive data byte.
    pub fn from_byte(b: Byte) -> Self {
        // SysEx value is 0...48, corresponding to four octaves (with 12 semitones each):
        // 0 = -2
        let semitones: i8 = b as i8 - 24;  // bring to range -24...24
        Transpose::new(semitones / 12)
    }
}

/// Voice.
#[derive(Debug, Clone)]
pub struct Voice {
    pub op1: Operator,
    pub op2: Operator,
    pub op3: Operator,
    pub op4: Operator,
    pub op5: Operator,
    pub op6: Operator,
    pub peg: Envelope,  // pitch env
    pub alg: Algorithm,  // 1...32
    pub feedback: Depth,
    pub osc_sync: bool,
    pub lfo: Lfo,
    pub transpose: Transpose,  // number of octaves to transpose (-2...+2) (12 = C2 (value is 0~48 in SysEx))
    pub name: String,
    pub op_flags: [bool; 6],
}

impl Voice {
    /// Creates a new voice and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            op1: Operator { output_level: Level::from(0), ..Operator::new() },
            op2: Operator { output_level: Level::from(0), ..Operator::new() },
            op3: Operator { output_level: Level::from(0), ..Operator::new() },
            op4: Operator { output_level: Level::from(0), ..Operator::new() },
            op5: Operator { output_level: Level::from(0), ..Operator::new() },
            op6: Operator { output_level: Level::from(0), ..Operator::new() },
            peg: Envelope {
                level1: Level::from(50),
                level2: Level::from(50),
                level3: Level::from(50),
                level4: Level::from(50),
                ..Envelope::new()
            },
            alg: Algorithm::from(1),
            feedback: Depth::from(0),
            osc_sync: true,
            lfo: Lfo::new(),
            transpose: Transpose::from(0),
            name: "INIT VOICE".to_string(),
            op_flags: [true, true, true, true, true, true],
        }
    }
}

impl Default for Voice {
    fn default() -> Self {
        Voice::new()
    }
}

impl SystemExclusiveData for Voice {
    fn from_bytes(data: ByteVector) -> Self {
        Voice {
            op6: Operator::from_bytes(data[0..22].to_vec()),
            op5: Operator::from_bytes(data[21..43].to_vec()),
            op4: Operator::from_bytes(data[42..64].to_vec()),
            op3: Operator::from_bytes(data[63..85].to_vec()),
            op2: Operator::from_bytes(data[84..106].to_vec()),
            op1: Operator::from_bytes(data[105..127].to_vec()),
            peg: Envelope::from_bytes(data[126..134].to_vec()),
            alg: Algorithm::from(data[134]),
            feedback: Depth::from(data[135]),
            osc_sync: if data[136] == 1 { true } else { false },
            lfo: Lfo::from_bytes(data[137..144].to_vec()),
            transpose: Transpose::from_byte(data[144]),
            name: String::from_utf8(data[145..155].to_vec()).unwrap(),
            /*op_flags: [data[155].bit(5), data[155].bit(4), data[155].bit(3), data[155].bit(2), data[155].bit(1), data[155].bit(0),]*/
            op_flags: [true, true, true, true, true, true],
        }
    }

    fn from_packed_bytes(data: ByteVector) -> Self {
        Voice {
            op6: Operator::from_packed_bytes(data[0..17].to_vec()),
            op5: Operator::from_packed_bytes(data[17..34].to_vec()),
            op4: Operator::from_packed_bytes(data[34..51].to_vec()),
            op3: Operator::from_packed_bytes(data[51..68].to_vec()),
            op2: Operator::from_packed_bytes(data[68..85].to_vec()),
            op1: Operator::from_packed_bytes(data[85..102].to_vec()),
            peg: Envelope::from_packed_bytes(data[102..110].to_vec()),
            alg: Algorithm::from(data[110]),
            feedback: Depth::from(data[111].bit_range(0..5)),
            osc_sync: if data[111].bit(3) { true } else { false },
            lfo: Lfo::from_packed_bytes(data[112..117].to_vec()),
            transpose: Transpose::from_byte(data[117]),
            name: String::from_utf8(data[118..128].to_vec()).unwrap(),
            op_flags: [true; 6],
        }
    }

    fn to_bytes(&self) -> ByteVector {
        let mut data: ByteVector = Vec::new();

        data.extend(self.op6.to_bytes());
        data.extend(self.op5.to_bytes());
        data.extend(self.op4.to_bytes());
        data.extend(self.op3.to_bytes());
        data.extend(self.op2.to_bytes());
        data.extend(self.op1.to_bytes());

        data.extend(self.peg.to_bytes());

        data.push((self.alg.value() - 1).try_into().unwrap());  // adjust alg# for SysEx
        data.push(self.feedback.as_byte());
        data.push(if self.osc_sync { 1 } else { 0 });
        data.extend(self.lfo.to_bytes());
        data.push(self.transpose.as_byte());

        let padded_name = format!("{:<10}", self.name);
        data.extend(padded_name.into_bytes());

        // "The OPERATOR ON/OFF parameter is not stored with the
        // voice, and is only transmitted or received while editing a voice.
        // So it only shows up in parameter change SYS-EX's."
        // Source: dx7sysexformat.txt
        /*
        let mut rev_flags = self.op_flags;
        rev_flags.reverse();
        let mut flags: u8 = 0;
        for (index, flag) in rev_flags.iter().enumerate() {
            if *flag {
                flags |= 1 << index
            }
        }
        data.push(flags);
        */

        assert_eq!(data.len(), 155);

        data
    }

    fn to_packed_bytes(&self) -> ByteVector {
        let mut data: ByteVector = Vec::new();

        let op6_data = self.op6.to_packed_bytes();
        debug!("OP6: {} bytes, {:?}", op6_data.len(), op6_data);
        data.extend(op6_data);

        let op5_data = self.op5.to_packed_bytes();
        debug!("OP5: {} bytes, {:?}", op5_data.len(), op5_data);
        data.extend(op5_data);

        let op4_data = self.op4.to_packed_bytes();
        debug!("OP4: {} bytes, {:?}", op4_data.len(), op4_data);
        data.extend(op4_data);

        let op3_data = self.op3.to_packed_bytes();
        debug!("OP3: {} bytes, {:?}", op3_data.len(), op3_data);
        data.extend(op3_data);

        let op2_data = self.op2.to_packed_bytes();
        debug!("OP2: {} bytes, {:?}", op2_data.len(), op2_data);
        data.extend(op2_data);

        let op1_data = self.op1.to_packed_bytes();
        debug!("OP1: {} bytes, {:?}", op1_data.len(), op1_data);
        data.extend(op1_data);

        let peg_data = self.peg.to_bytes(); // not packed!
        debug!("PEG: {} bytes, {:?}", peg_data.len(), peg_data);
        data.extend(peg_data);

        let algorithm = self.alg.value();
        data.push((algorithm - 1).try_into().unwrap());  // bring alg to range 0...31
        debug!("ALG: {}", algorithm);

        let byte111 = self.feedback.as_byte() | ((if self.osc_sync { 1 } else { 0 }) << 3);
        data.push(byte111);
        debug!("  b111: {:#08b}", byte111);

        // Inject the pitch mod sensitivity value to the last LFO byte
        let lfo_data = self.lfo.to_packed_bytes();
        debug!("LFO: {} bytes, {:?}", lfo_data.len(), lfo_data);
        data.extend(lfo_data);

        data.push(self.transpose.as_byte());
        debug!("  TRNSP: {:#02X}", self.transpose.value());

        let padded_name = format!("{:<10}", self.name);
        debug!("  NAME: '{}'", padded_name);
        data.extend(padded_name.into_bytes());

        data
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "==========
{}
==========
OP1: {}
OP2: {}
OP3: {}
OP4: {}
OP5: {}
OP6: {}
PEG: {}
ALG: {}, feedback = {}, osc sync = {}
LFO: {}
Transpose: {}
",
            self.name, self.op1, self.op2, self.op3, self.op4, self.op5, self.op6, self.peg,
            self.alg, self.feedback.value(), self.osc_sync, self.lfo, self.transpose.value())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_bit_range() {
        let b: u8 = 0b00110000;

        // If this succeeds, the range upper bound is not included,
        // i.e. 4..6 means bits 4 and 5.
        assert_eq!(b.bit_range(4..6), 0b11);
    }

    #[test]
    fn test_checksum() {
        // Yamaha DX7 original ROM1A sound bank (data only, no SysEx header/terminator
        // or checksum.)
        let rom1a_data: [u8; 4096] = include!("rom1asyx.in");

        // The checksum is 0x33
        let rom1a_data_checksum = voice_checksum(&rom1a_data.to_vec());
        assert_eq!(0x33, rom1a_data_checksum);
        //debug!("ROM1A data checksum = {:X}h", rom1a_data_checksum);
    }

    #[test]
    fn test_eg_to_bytes() {
        let eg = Envelope {
            rate1: Rate::from(64),
            rate2: Rate::from(64),
            rate3: Rate::from(64),
            rate4: Rate::from(64),
            level1: Level::from(32),
            level2: Level::from(32),
            level3: Level::from(32),
            level4: Level::from(32),
        };
        assert_eq!(eg.to_bytes(), vec![64u8, 64, 64, 64, 32, 32, 32, 32]);
    }

    #[test]
    fn test_scaling_curve_exp_pos_to_bytes() {
        assert_eq!(ScalingCurve::exp_pos().to_bytes(), 2);
    }

    #[test]
    fn test_scaling_curve_exp_neg_to_bytes() {
        assert_eq!(ScalingCurve::exp_neg().to_bytes(), 1);
    }

    #[test]
    fn test_scaling_curve_lin_pos_to_bytes() {
        assert_eq!(ScalingCurve::lin_pos().to_bytes(), 3);
    }

    #[test]
    fn test_scaling_curve_lin_neg_to_bytes() {
        assert_eq!(ScalingCurve::lin_neg().to_bytes(), 0);
    }

    #[test]
    fn test_kbd_level_scaling_to_packed_bytes() {
        // From ROM1A: BRASS 1
        let ks = KeyboardLevelScaling {
            breakpoint: 60 - 21,
            left_depth: 54,
            right_depth: 50,
            left_curve: ScalingCurve::exp_neg(),
            right_curve: ScalingCurve::exp_neg(),
        };

        assert_eq!(
            ks.to_packed_bytes(),
            vec![39, 54, 50, 5]
        )
    }

    #[test]
    fn test_op_to_packed_bytes() {
        let op = Operator {
            eg: Envelope {
                rate1: Rate::from(49),
                rate2: Rate::from(99),
                rate3: Rate::from(28),
                rate4: Rate::from(68),
                level1: Level::from(98),
                level2: Level::from(98),
                level3: Level::from(91),
                level4: Level::from(0),
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 39,
                left_depth: 54,
                right_depth: 50,
                left_curve: ScalingCurve::exp_neg(),
                right_curve: ScalingCurve::exp_neg(),
            },
            kbd_rate_scaling: Depth::from(4),
            amp_mod_sens: 0,
            key_vel_sens: Depth::from(2),
            output_level: Level::from(82),
            mode: OperatorMode::Ratio,
            coarse: Coarse::from(1),
            fine: Level::from(0),
            detune: Detune::from(0),
        };

        let data = op.to_packed_bytes();

        let expected_data = vec![0x31u8, 0x63, 0x1c, 0x44, 0x62, 0x62, 0x5b, 0x00, 0x27, 0x36, 0x32, 0x05, 0x3c, 0x08, 0x52, 0x02, 0x00];

        let diff_offset = first_different_offset(&expected_data, &data);
        match diff_offset {
            Some(offset) => {
                println!("Vectors differ at offset {:?}", offset);
                println!("Expected = {}, actual = {}", expected_data[offset], data[offset]);
            },
            None => println!("Vectors are the same")
        }

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_lfo_to_packed_bytes() {
        let lfo = Lfo {
            speed: Level::from(37),
            delay: Level::from(0),
            pmd: Level::from(5),
            amd: Level::from(0),
            sync: false,
            wave: LfoWaveform::Sine,
            pitch_mod_sens: Depth::from(3),
        };

        assert_eq!(
            lfo.to_packed_bytes(),
            vec![37, 0, 5, 0, 0x38]
        );
    }

    #[test]
    fn test_voice_packed_length() {
        let brass1 = make_brass1();
        assert_eq!(brass1.to_packed_bytes().len(), 128);
    }

    // Finds the first offset where the two vectors differ.
    // Returns None if no differences are found, or if the vectors
    // are different lengths, Some<usize> with the offset otherwise.
    fn first_different_offset(v1: &[u8], v2: &[u8]) -> Option<usize> {
        if v1.len() != v2.len() {
            return None;
        }

        let mut offset = 0;
        for i in 0..v1.len() {
            if v1[i] != v2[i] {
                return Some(offset);
            }
            else {
                offset += 1;
            }
        }

        None
    }

    #[test]
    fn test_voice_to_packed_bytes() {
        let rom1a_data: [u8; 4096] = include!("rom1asyx.in");

        // The first voice in ROM1A ("BRASS 1") is the first 128 bytes
        let voice_data = &rom1a_data[..128];
        let brass1 = make_brass1();
        let brass1_data = brass1.to_packed_bytes();

        let diff_offset = first_different_offset(voice_data, &brass1_data);
        match diff_offset {
            Some(offset) => {
                println!("Vectors differ at offset {:?}", offset);
                println!("Expected = {}, actual = {}", voice_data[offset], brass1_data[offset]);
            },
            None => println!("Vectors are the same")
        }

        assert_eq!(voice_data, brass1_data);
    }

    #[test]
    fn test_operator_from_bytes() {
        let data = vec![
            0x03u8, 0x47, 0x00, 0x03, 0x00, 0x07, 0x63, 0x23,  // rate and level
            0x63, 0x57, 0x63, 0x63, 0x63,  // kbd level scaling
            0x00, 0x00, 0x00,
            0x11,  // output level
            0x00,   // osc mode
            0x00, 0x00, 0x00, // coarse, fine, detune
        ];
        assert_eq!(data.len(), 21);
        let operator = Operator::from_bytes(data);
        let coarse = operator.coarse;
        assert_eq!(coarse.value(), 0);
    }

    #[test]
    fn test_voice_from_bytes() {
        let data: [u8; 155] = [
            0x63, 0x63, 0x63, 0x63,
            0x63, 0x63, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x57, 0x00, 0x0b, 0x00, 0x07, 0x63, 0x27, 0x63,
            0x63, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x07, 0x41, 0x00, 0x00, 0x00, 0x07, 0x63, 0x27,
            0x63, 0x63, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00,
            0x00, 0x00, 0x00, 0x05, 0x58, 0x00, 0x08, 0x00, 0x07, 0x63,
            0x20, 0x63, 0x57, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x03, 0x47, 0x00, 0x03, 0x00, 0x07,
            0x63, 0x23, 0x63, 0x57, 0x63, 0x63, 0x63, 0x00, 0x00, 0x00,
            0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00,
            0x07, 0x63, 0x43, 0x1e, 0x57, 0x63, 0x5f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x63, 0x00, 0x00,
            0x00, 0x07, 0x63, 0x63, 0x63, 0x63, 0x32, 0x32, 0x32, 0x32,
            0x0f, 0x05, 0x01, 0x23, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x18, 0x47, 0x45, 0x54, 0x20, 0x46, 0x55, 0x4e, 0x4b, 0x59,
            0x20,
        ];

        let voice = Voice::from_bytes(data.to_vec());
        assert_eq!(voice.name, "GET FUNKY ");
    }

    #[test]
    fn test_bulk_b111() {
        let sync = true;
        let feedback = 7u8;
        let expected = 0x0fu8;
        let actual = feedback | ((if sync { 1 } else { 0 }) << 3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bulk_b116() {
        let sync = false;
        let wave = LfoWaveform::Sine;
        let pms = 3u8;
        let mut actual: Byte = if sync { 1u8 } else { 0u8 };
        actual.set_bit_range(1..4, wave as Byte);
        actual.set_bit_range(4..7, pms);
        assert_eq!(actual, 0x38);
    }

    #[test]
    fn test_unsigned_level() {
        let level = UnsignedLevel::from(42);
        assert_eq!(level, UnsignedLevel::from(42));
    }

    #[test]
    fn test_unsigned_level_clamped() {
        let level = UnsignedLevel::from(192);  // too big for range
        assert_eq!(level, UnsignedLevel::from(99));  // should be clamped to top of range
    }

    #[test]
    fn test_unsigned_level_default() {
        let level = UnsignedLevel::default();  // based on u8
        assert_eq!(level, UnsignedLevel::from(0));  // so should be the u8 Default
    }

    #[test]
    fn test_transpose_from_byte() {
        let transpose_zero = Transpose::from_byte(24);
        assert_eq!(transpose_zero.value(), 0);
        let transpose_minus_one = Transpose::from_byte(12);
        assert_eq!(transpose_minus_one.value(), -1);
    }

    #[test]
    fn test_transpose_as_byte() {
        let transpose_plus_one = Transpose::from(1);
        assert_eq!(transpose_plus_one.as_byte(), 36);
    }
}

fn voice_checksum(data: &Vec<u8>) -> u8 {
    let mut sum: u32 = 0;
    for b in data {
        sum += *b as u32;
    }

    let mut checksum: u8 = (sum & 0xff) as u8;
    checksum = !checksum;
    checksum &= 0x7f;
    checksum += 1;
    checksum
}

pub static ALGORITHM_DIAGRAMS: [&str; 32] = include!("algorithms.in");
