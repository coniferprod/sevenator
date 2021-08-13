use std::fs::File;
use std::io::prelude::*;
use std::fmt;
use std::ops::RangeInclusive;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn, error, debug};
use rand::Rng;
use num;

// Helper types to keep the parameters in range and generate random values.

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RangeKind {
    OutputLevel,
    Rate,
    Level,
    Coarse,
    Fine,
}

// Rust ranges are not Copy because reasons (see https://github.com/rust-lang/rfcs/issues/2848),
// so let's use a wrapper:
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct RangeInclusiveWrapper {
    start: i16,
    end: i16,
}
// Everything fits in an i16, so that's why it's the base type.

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct RangedValue {
    kind: RangeKind,
    value: i16,
    range: RangeInclusiveWrapper,
}

impl Default for RangedValue {
    fn default() -> Self {
        RangedValue {
            kind: RangeKind::OutputLevel,
            value: 0,
            range: RangedValue::make_range(RangeKind::OutputLevel),
        }
    }
}

pub type IntRange = RangeInclusive<i16>;

impl RangedValue {
    // Private helper to get the range based on the kind.
    fn make_range(kind: RangeKind) -> RangeInclusiveWrapper {
        // It would have been so nice just to say `0..=99`...
        match kind {
            RangeKind::OutputLevel => RangeInclusiveWrapper { start: 0, end: 99 },
            RangeKind::Rate => RangeInclusiveWrapper { start: 0, end: 99 },
            RangeKind::Level => RangeInclusiveWrapper { start: 0, end: 99 },
            RangeKind::Coarse => RangeInclusiveWrapper { start: 0, end: 31 },
            RangeKind::Fine => RangeInclusiveWrapper { start: 0, end: 99 },
        }
    }

    /// Makes a new value from the given byte.
    pub fn from_byte(kind: RangeKind, initial_value: u8) -> RangedValue {
        let range = RangedValue::make_range(kind);

        // If this were a regular RangeInclusive, these would be calls to start() and end():
        let value = if range.start < 0 {  // need to adjust value
            initial_value as i16 - (range.end + 1) as i16
        }
        else {
            initial_value as i16
        };

        RangedValue { kind, range, value }
    }

    /// Makes a new ranged value from the given integer, clamping if necessary.
    pub fn from_int(kind: RangeKind, initial_value: i16) -> RangedValue {
        let range = RangedValue::make_range(kind);

        let value = if initial_value >= range.start && initial_value <= range.end {
            initial_value
        }
        else {
            num::clamp(initial_value, range.start, range.end)
        };

        RangedValue { kind, range, value }
    }

    /// Makes a new ranged value initialized to the start of the range.
    pub fn new_min(kind: RangeKind) -> RangedValue {
        let range = RangedValue::make_range(kind);
        RangedValue { kind, range, value: range.start }
    }

    /// Makes a new ranged value initialized to the end of the range.
    pub fn new_max(kind: RangeKind) -> RangedValue {
        let range = RangedValue::make_range(kind);
        RangedValue { kind, range, value: range.end }
    }

    /// Gets the range of this value.
    pub fn range(&self) -> IntRange {
        // Make a new normal range from our wrapper
        self.range.start..=self.range.end
    }

    /// Gets the range kind of this value.
    pub fn kind(&self) -> RangeKind {
        self.kind
    }

    /// Gets the current value.
    pub fn get(&self) -> i16 {
        self.value
    }

    /// Sets the current value, clamping it if necessary.
    pub fn set(&mut self, new_value: i16) {
        self.value = if new_value >= self.range.start && new_value <= self.range.end {
            new_value
        }
        else {
            num::clamp(new_value, self.range.start, self.range.end)
        }
    }

    /// Gets the value as a byte.
    pub fn as_byte(&self) -> u8 {
        if self.range.start < 0 {
            (self.value + self.range.end + 1) as u8
        }
        else {
            self.value as u8
        }
    }

    /// Default implementation for random value.
    pub fn random_value(&self) -> i16 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.range.start..=self.range.end)
    }

    /// Default implementation for random value from a subrange.
    pub fn random_value_restricted(&self, subrange: IntRange) -> i16 {
        assert!(subrange.start() >= &self.range.start && subrange.end() <= &self.range.end);

        let mut rng = rand::thread_rng();
        rng.gen_range(subrange)
    }
}

fn make_brass1() -> Voice {
    let op6 = Operator {
        eg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(49),
            rate2: EnvelopeGenerator::new_rate(99),
            rate3: EnvelopeGenerator::new_rate(28),
            rate4: EnvelopeGenerator::new_rate(68),
            level1: EnvelopeGenerator::new_level(98),
            level2: EnvelopeGenerator::new_level(98),
            level3: EnvelopeGenerator::new_level(91),
            level4: EnvelopeGenerator::new_level(0),
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60 - 21,
            left_depth: 54,
            right_depth: 50,
            left_curve: ScalingCurve {
                curve: CurveStyle::Exponential,
                positive: false
            },
            right_curve: ScalingCurve {
                curve: CurveStyle::Exponential,
                positive: false
            },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 82),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: 0,
    };

    let op5 = Operator {
        eg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(77),
            rate2: EnvelopeGenerator::new_rate(36),
            rate3: EnvelopeGenerator::new_rate(41),
            rate4: EnvelopeGenerator::new_rate(71),
            level1: EnvelopeGenerator::new_level(99),
            level2: EnvelopeGenerator::new_level(98),
            level3: EnvelopeGenerator::new_level(98),
            level4: EnvelopeGenerator::new_level(0),
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60 - 21, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 98),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: 1,
    };

    let op4 = Operator {
        eg: op5.eg.clone(),
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60 - 21, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 99),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: 0
    };

    let op3 = Operator {
        eg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(77),
            rate2: EnvelopeGenerator::new_rate(76),
            rate3: EnvelopeGenerator::new_rate(82),
            rate4: EnvelopeGenerator::new_rate(71),
            level1: EnvelopeGenerator::new_level(99),
            level2: EnvelopeGenerator::new_level(98),
            level3: EnvelopeGenerator::new_level(98),
            level4: EnvelopeGenerator::new_level(0),
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60 - 21, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 99),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: -2
    };

    let op2 = Operator {
        eg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(62),
            rate2: EnvelopeGenerator::new_rate(51),
            rate3: EnvelopeGenerator::new_rate(29),
            rate4: EnvelopeGenerator::new_rate(71),
            level1: EnvelopeGenerator::new_level(82),
            level2: EnvelopeGenerator::new_level(95),
            level3: EnvelopeGenerator::new_level(96),
            level4: EnvelopeGenerator::new_level(0),
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 48 - 21, left_depth: 0, right_depth: 7,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Exponential, positive: false },
        },
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 0,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 86),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 0),
        fine: RangedValue::from_int(RangeKind::Fine, 50),
        detune: 7
    };

    let op1 = Operator {
        eg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(72),
            rate2: EnvelopeGenerator::new_rate(76),
            rate3: EnvelopeGenerator::new_rate(99),
            rate4: EnvelopeGenerator::new_rate(71),
            level1: EnvelopeGenerator::new_level(99),
            level2: EnvelopeGenerator::new_level(88),
            level3: EnvelopeGenerator::new_level(96),
            level4: EnvelopeGenerator::new_level(0),
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60 - 21, left_depth: 0, right_depth: 14,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 0,
        output_level: RangedValue::from_int(RangeKind::OutputLevel, 98),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 0),
        fine: RangedValue::from_int(RangeKind::Fine, 50),
        detune: 7
    };

    Voice {
        op1: op1,
        op2: op2,
        op3: op3,
        op4: op4,
        op5: op5,
        op6: op6,
        peg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(84),
            rate2: EnvelopeGenerator::new_rate(95),
            rate3: EnvelopeGenerator::new_rate(95),
            rate4: EnvelopeGenerator::new_rate(60),
            level1: EnvelopeGenerator::new_level(50),
            level2: EnvelopeGenerator::new_level(50),
            level3: EnvelopeGenerator::new_level(50),
            level4: EnvelopeGenerator::new_level(50),
        },
        alg: 22,
        feedback: 7,
        osc_sync: true,
        lfo: Lfo {
            speed: RangedValue::from_int(RangeKind::Level, 37),
            delay: RangedValue::new_min(RangeKind::Level),
            pmd: RangedValue::from_int(RangeKind::Level, 5),
            amd: RangedValue::new_min(RangeKind::Level),
            sync: false, wave: LfoWaveform::Sine,
        },
        pitch_mod_sens: 3,
        transpose: 24,
        name: "BRASS   1 ".to_string(),
        op_flags: [true, true, true, true, true, true],
    }
}

// Makes an initialized voice. The defaults are as described in
// Howard Massey's "The Complete DX7", Appendix B.
fn make_init_voice() -> Voice {
    let init_eg = EnvelopeGenerator::new();

    // Break point = A-1 for all operators
    // Curve = -LIN for both curves, all operators
    // Depth = 0 for both curves, all operators
    let init_kbd_level_scaling = KeyboardLevelScaling {
        breakpoint: 0, left_depth: 0, right_depth: 0,
        left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
        right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
    };

    let init_op1 = Operator {
        eg: init_eg.clone(),
        kbd_level_scaling: init_kbd_level_scaling.clone(),
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 0,
        output_level: RangedValue::new_max(RangeKind::OutputLevel),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: 0
    };

    let init_op_rest = Operator {
        output_level: RangedValue::new_min(RangeKind::OutputLevel),
        ..init_op1
    };

    Voice {
        op1: init_op1.clone(),
        op2: init_op_rest.clone(),
        op3: init_op_rest.clone(),
        op4: init_op_rest.clone(),
        op5: init_op_rest.clone(),
        op6: init_op_rest.clone(),
        peg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(99),
            rate2: EnvelopeGenerator::new_rate(99),
            rate3: EnvelopeGenerator::new_rate(99),
            rate4: EnvelopeGenerator::new_rate(99),
            level1: EnvelopeGenerator::new_level(50),
            level2: EnvelopeGenerator::new_level(50),
            level3: EnvelopeGenerator::new_level(50),
            level4: EnvelopeGenerator::new_level(50),
        },
        alg: 1,
        feedback: 0,
        osc_sync: true, // osc key sync = on
        lfo: Lfo {
            speed: RangedValue::from_int(RangeKind::Level, 35),
            delay: RangedValue::new_min(RangeKind::Level),
            pmd: RangedValue::new_min(RangeKind::Level),
            amd: RangedValue::new_min(RangeKind::Level),
            sync: true,
            wave: LfoWaveform::Triangle,
        },
        pitch_mod_sens: 3,
        transpose: 24,
        name: "INIT VOICE".to_string(),
        op_flags: [true, true, true, true, true, true],  // all operators ON
    }
}

fn make_random_eg() -> EnvelopeGenerator {
    let rate = EnvelopeGenerator::new_rate(0);
    let level = EnvelopeGenerator::new_level(0);

    EnvelopeGenerator {
        rate1: EnvelopeGenerator::new_random_rate(),
        rate2: EnvelopeGenerator::new_random_rate(),
        rate3: EnvelopeGenerator::new_random_rate(),
        rate4: EnvelopeGenerator::new_random_rate(),
        level1: EnvelopeGenerator::new_random_level(),
        level2: EnvelopeGenerator::new_random_level(),
        level3: EnvelopeGenerator::new_random_level(),
        level4: EnvelopeGenerator::new_random_level(),
    }
}

fn make_random_operator() -> Operator {
    Operator {
        eg: make_random_eg(),
        kbd_level_scaling: KeyboardLevelScaling::new(),
        kbd_rate_scaling: 0,
        amp_mod_sens: 0,
        key_vel_sens: 0,
        output_level: Operator::new_random_output_level(),
        mode: OperatorMode::Ratio,
        coarse: RangedValue::from_int(RangeKind::Coarse, 1),
        fine: RangedValue::from_int(RangeKind::Fine, 0),
        detune: 0
    }
}

fn make_random_voice() -> Voice {
    Voice {
        op1: make_random_operator(),
        op2: make_random_operator(),
        op3: make_random_operator(),
        op4: make_random_operator(),
        op5: make_random_operator(),
        op6: make_random_operator(),
        peg: EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(99),
            rate2: EnvelopeGenerator::new_rate(99),
            rate3: EnvelopeGenerator::new_rate(99),
            rate4: EnvelopeGenerator::new_rate(99),
            level1: EnvelopeGenerator::new_level(50),
            level2: EnvelopeGenerator::new_level(50),
            level3: EnvelopeGenerator::new_level(50),
            level4: EnvelopeGenerator::new_level(50),
        },
        alg: 1,
        feedback: 0,
        osc_sync: true, // osc key sync = on
        lfo: Lfo::new_random(),
        pitch_mod_sens: 3,
        transpose: 24,
        name: "RNDM VOICE".to_string(),
        op_flags: [true, true, true, true, true, true],  // all operators ON
    }
}

const VOICE_COUNT: usize = 32;

fn make_init_cartridge() -> Vec<Voice> {
    vec![make_init_voice(); VOICE_COUNT]
}

fn make_random_cartridge() -> Vec<Voice> {
    vec![make_random_voice(); VOICE_COUNT]
}

pub fn run() -> std::io::Result<()> {
    // Get the default voice with `Voice::new()`.
    // The `make_init_voice()` function makes exactly the original init voice.
    // These should be more or less the same.
    //let cartridge = make_init_cartridge();

    // Make a cartridge full of random voices
    let cartridge = make_random_cartridge();

    let mut cartridge_data: Vec<u8> = Vec::new();

    for (index, voice) in cartridge.iter().enumerate() {
        let mut voice_data = voice.to_packed_bytes();
        debug!("Voice #{} packed data length = {} bytes", index, voice_data.len());
        cartridge_data.append(&mut voice_data);
    }

    // Compute the checksum before we add the SysEx header and terminator,
    // but don't add it yet -- only just before the terminator.
    let cartridge_checksum = voice_checksum(&cartridge_data);
    debug!("cartridge checksum = {:02X}h", cartridge_checksum);

    // Insert the System Exclusive header at the beginning of the vector:
    let header = vec![
        0xf0u8, // SysEx initiator
        0x43,   // Yamaha manufacturer ID
        0x00,   // MIDI channel 1
        0x09,   // format = 9 (32 voices)
        0x20,   // byte count MSB
        0x00,   // byte count LSB
    ];
    debug!("header length = {} bytes", header.len());
    // This may be a bit inefficient, but not too much.
    // The last byte of the header goes first to 0, then the others follow.
    for b in header.iter().rev() {
        cartridge_data.insert(0, *b);
    }

    // Now is the right time to apped the checksum
    cartridge_data.push(cartridge_checksum);

    // Add the System Exclusive message terminator:
    cartridge_data.push(0xf7u8);

    let now = SystemTime::now();
    let epoch_now = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let filename = format!("cartridge-{:?}.syx", epoch_now.as_secs());
    {
        let mut file = File::create(filename)?;
        file.write_all(&cartridge_data)?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct EnvelopeGenerator {
    rate1: RangedValue,
    rate2: RangedValue,
    rate3: RangedValue,
    rate4: RangedValue,
    level1: RangedValue,
    level2: RangedValue,
    level3: RangedValue,
    level4: RangedValue,
}

impl EnvelopeGenerator {
    /// Creates a new EG with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            rate1: EnvelopeGenerator::new_rate(99),
            rate2: EnvelopeGenerator::new_rate(99),
            rate3: EnvelopeGenerator::new_rate(99),
            rate4: EnvelopeGenerator::new_rate(99),
            level1: EnvelopeGenerator::new_level(99),
            level2: EnvelopeGenerator::new_level(99),
            level3: EnvelopeGenerator::new_level(99),
            level4: EnvelopeGenerator::new_level(0),
        }
    }

    /*
    From the Yamaha DX7 Operation Manual (p. 51):
    "You can simulate an ADSR if you set the envelope as follows:
    L1=99, L2=99, L4=0, and R2=99.
    With these settings, then R1 becomes Attack time, R3 is Decay
    time, L3 is Sustain level, and R4 is Release time."
    */
    pub fn adsr(attack: u8, decay: u8, sustain: u8, release: u8) -> Self {
        Self {
            rate1: RangedValue::from_byte(RangeKind::Rate, attack),
            rate2: RangedValue::new_max(RangeKind::Rate),
            rate3: RangedValue::from_byte(RangeKind::Rate, decay),
            rate4: RangedValue::from_byte(RangeKind::Rate, release),
            level1: RangedValue::new_max(RangeKind::Level),
            level2: RangedValue::new_max(RangeKind::Level),
            level3: RangedValue::from_byte(RangeKind::Level, sustain),
            level4: RangedValue::new_min(RangeKind::Level),
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            rate1: RangedValue::from_byte(RangeKind::Rate, data[0]),
            rate2: RangedValue::from_byte(RangeKind::Rate, data[1]),
            rate3: RangedValue::from_byte(RangeKind::Rate, data[2]),
            rate4: RangedValue::from_byte(RangeKind::Rate, data[3]),
            level1: RangedValue::from_byte(RangeKind::Rate, data[4]),
            level2: RangedValue::from_byte(RangeKind::Rate, data[5]),
            level3: RangedValue::from_byte(RangeKind::Rate, data[6]),
            level4: RangedValue::from_byte(RangeKind::Rate, data[7]),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.rate1.as_byte(), self.rate2.as_byte(), self.rate3.as_byte(), self.rate4.as_byte(),
            self.level1.as_byte(), self.level2.as_byte(), self.level3.as_byte(), self.level4.as_byte()
        ]
    }

    pub fn new_random() -> Self {
        Self {
            rate1: EnvelopeGenerator::new_random_rate(),
            rate2: EnvelopeGenerator::new_random_rate(),
            rate3: EnvelopeGenerator::new_random_rate(),
            rate4: EnvelopeGenerator::new_random_rate(),
            level1: EnvelopeGenerator::new_random_level(),
            level2: EnvelopeGenerator::new_random_level(),
            level3: EnvelopeGenerator::new_random_level(),
            level4: EnvelopeGenerator::new_random_level(),
        }
    }

    pub fn new_rate(value: i16) -> RangedValue {
        let kind = RangeKind::Rate;
        RangedValue::from_int(kind, value)
    }

    pub fn new_random_rate() -> RangedValue {
        let rate = EnvelopeGenerator::new_rate(0);
        RangedValue::from_int(RangeKind::Rate, rate.random_value())
    }

    pub fn new_level(value: i16) -> RangedValue {
        let kind = RangeKind::Level;
        RangedValue::from_int(kind, value)
    }

    pub fn new_random_level() -> RangedValue {
        let level = EnvelopeGenerator::new_level(0);
        RangedValue::from_int(RangeKind::Level, level.random_value())
    }
}

impl fmt::Display for EnvelopeGenerator {
    // The display trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R1={} L1={} R2={} L2={} R3={} L3={} R4={} L4={}",
            self.rate1.get(), self.level1.get(), self.rate2.get(), self.level2.get(),
            self.rate3.get(), self.level3.get(), self.rate4.get(), self.level4.get())
    }
}

#[derive(Debug, Copy, Clone)]
enum CurveStyle {
    Linear,
    Exponential
}

#[derive(Debug, Clone, Copy)]
struct ScalingCurve {
    curve: CurveStyle,
    positive: bool,  // true if positive, false if negative
}

impl ScalingCurve {
    pub fn lin_pos() -> Self {
        ScalingCurve { curve: CurveStyle::Linear, positive: true }
    }

    pub fn lin_neg() -> Self {
        ScalingCurve { curve: CurveStyle::Linear, positive: false }
    }

    pub fn exp_pos() -> Self {
        ScalingCurve { curve: CurveStyle::Exponential, positive: true }
    }

    pub fn exp_neg() -> Self {
        ScalingCurve { curve: CurveStyle::Exponential, positive: false }
    }

    pub fn to_bytes(&self) -> u8 {
        match self {
            ScalingCurve { curve: CurveStyle::Linear, positive: true } => 3,
            ScalingCurve { curve: CurveStyle::Linear, positive: false } => 0,
            ScalingCurve { curve: CurveStyle::Exponential, positive: true } => 2,
            ScalingCurve { curve: CurveStyle::Exponential, positive: false } => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct KeyboardLevelScaling {
    breakpoint: u8, // 0 ~ 99 (A-1 ~ C8)
    left_depth: u8,
    right_depth: u8,
    left_curve: ScalingCurve,  // 0 ~ 3
    right_curve: ScalingCurve, // 0 ~ 3
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
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            breakpoint: data[0],
            left_depth: data[1],
            right_depth: data[2],
            left_curve: match data[3] {
                0 => ScalingCurve::lin_neg(),
                1 => ScalingCurve::exp_neg(),
                2 => ScalingCurve::exp_pos(),
                3 => ScalingCurve::lin_pos(),
                _ => ScalingCurve::lin_pos(),
            },
            right_curve: match data[3] {
                0 => ScalingCurve::lin_neg(),
                1 => ScalingCurve::exp_neg(),
                2 => ScalingCurve::exp_pos(),
                3 => ScalingCurve::lin_pos(),
                _ => ScalingCurve::lin_pos(),
            },
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.breakpoint,
            self.left_depth,
            self.right_depth,
            self.left_curve.to_bytes(),
            self.right_curve.to_bytes(),
        ]
    }

    pub fn to_packed_bytes(&self) -> Vec<u8> {
        vec![
            self.breakpoint,
            self.left_depth,
            self.right_depth,
            self.left_curve.to_bytes() | (self.right_curve.to_bytes() << 2),
        ]
    }
}

#[derive(Debug, Copy, Clone)]
enum OperatorMode {
    Ratio,
    Fixed,
}

#[derive(Debug, Clone)]
struct Operator {
    eg: EnvelopeGenerator,
    kbd_level_scaling: KeyboardLevelScaling,
    kbd_rate_scaling: u8, // 0 ~ 7
    amp_mod_sens: u8,  // 0 ~ 3
    key_vel_sens: u8,  // 0 ~ 7
    output_level: RangedValue,
    mode: OperatorMode,
    coarse: RangedValue,  // 0 ~ 31
    fine: RangedValue,  // 0 ~ 99
    detune: i8,   // -7 ~ 7
}

impl Operator {
    /// Creates a new operator and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            eg: EnvelopeGenerator::new(),
            kbd_level_scaling: KeyboardLevelScaling::new(),
            kbd_rate_scaling: 0,
            amp_mod_sens: 0,
            key_vel_sens: 0,
            output_level: RangedValue::from_int(RangeKind::OutputLevel, 0),
            mode: OperatorMode::Ratio,
            coarse: RangedValue::from_int(RangeKind::Coarse, 1),
            fine: RangedValue::from_int(RangeKind::Fine, 0),  // TODO: voice init for fine is "1.00 for all operators", should this be 0 or 1?
            detune: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let eg_bytes = &data[0..8];
        let level_scaling_bytes = &data[8..13];
        let mode = match data[18] {
            0 => OperatorMode::Ratio,
            1 => OperatorMode::Fixed,
            _ => OperatorMode::Ratio
        };
        Self {
            eg: EnvelopeGenerator::from_bytes(eg_bytes.to_vec()),
            kbd_level_scaling: KeyboardLevelScaling::from_bytes(level_scaling_bytes.to_vec()),
            kbd_rate_scaling: data[14],
            amp_mod_sens: data[15],
            key_vel_sens: data[16],
            output_level: RangedValue::from_byte(RangeKind::OutputLevel, data[17]),
            mode: mode,
            coarse: RangedValue::from_byte(RangeKind::Coarse, data[19]),
            fine: RangedValue::from_byte(RangeKind::Fine, data[20]),
            detune: data[21] as i8,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.append(&mut self.eg.to_bytes());
        data.append(&mut self.kbd_level_scaling.to_bytes());
        data.push(self.kbd_rate_scaling);
        data.push(self.amp_mod_sens);
        data.push(self.key_vel_sens);
        data.push(self.output_level.as_byte());
        data.push(self.mode as u8);
        data.push(self.coarse.as_byte());
        data.push(self.fine.as_byte());
        data.push((self.detune + 7) as u8); // 0 = detune -7
        data
    }

    pub fn to_packed_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        let mut eg_data = self.eg.to_bytes(); // not packed!
        debug!("  EG: {} bytes, {:?}", eg_data.len(), eg_data);
        data.append(&mut eg_data);

        let mut kls_data = self.kbd_level_scaling.to_packed_bytes();
        debug!("  KLS: {} bytes, {:?}", kls_data.len(), kls_data);
        data.append(&mut kls_data);

        let byte12 = self.kbd_rate_scaling | (((self.detune + 7) as u8) << 3);
        debug!("  KBD RATE SCALING = {:?} DETUNE = {:?} b12: {:#08b}", self.kbd_rate_scaling, self.detune, byte12);
        data.push(byte12);

        let byte13 = self.amp_mod_sens | (self.key_vel_sens << 2);
        debug!("  b13: {:#08b}", byte12);
        data.push(byte13);

        let output_level = self.output_level.get();
        debug!("  OL:  {:#08b}", output_level);
        data.push(self.output_level.as_byte());

        let byte15 = self.mode as u8 | (self.coarse.as_byte() << 1);
        debug!("  b15: {:#08b}", byte15);
        data.push(byte15);

        let fine = self.fine.get();
        debug!("  FF:  {:#08b}", fine);
        data.push(self.fine.as_byte());

        data
    }

    pub fn new_random_output_level() -> RangedValue {
        let output_level = RangedValue::new_min(RangeKind::OutputLevel);
        RangedValue::from_int(RangeKind::OutputLevel, output_level.random_value())
    }
}

#[derive(Debug, Copy, Clone)]
enum LfoWaveform {
    Triangle,
    SawDown,
    SawUp,
    Square,
    Sine,
    SampleAndHold,
}

#[derive(Debug, Clone)]
struct Lfo {
    speed: RangedValue,  // 0 ~ 99
    delay: RangedValue,  // 0 ~ 99
    pmd: RangedValue,    // 0 ~ 99
    amd: RangedValue,    // 0 ~ 99
    sync: bool,
    wave: LfoWaveform,
}

impl Lfo {
    /// Makes a new LFO initialized with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            speed: RangedValue::from_int(RangeKind::Level, 35),
            delay: RangedValue::new_min(RangeKind::Level),
            pmd: RangedValue::new_min(RangeKind::Level),
            amd: RangedValue::new_min(RangeKind::Level),
            sync: true,
            wave: LfoWaveform::Triangle,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.speed.as_byte(),
            self.delay.as_byte(),
            self.pmd.as_byte(),
            self.amd.as_byte(),
            if self.sync { 1 } else { 0 },
            self.wave as u8,
        ]
    }

    pub fn to_packed_bytes(&self) -> Vec<u8> {
        vec![
            self.speed.as_byte(),
            self.delay.as_byte(),
            self.pmd.as_byte(),
            self.amd.as_byte(),
            (if self.sync { 1 } else { 0 }) | ((self.wave as u8) << 1),
        ]
    }

    pub fn new_random() -> Self {
        let level = RangedValue::new_min(RangeKind::Level);
        Self {
            speed: RangedValue::from_int(RangeKind::Level, level.random_value()),
            delay: RangedValue::from_int(RangeKind::Level, level.random_value()),
            pmd: RangedValue::from_int(RangeKind::Level, level.random_value()),
            amd: RangedValue::from_int(RangeKind::Level, level.random_value()),
            sync: true,
            wave: LfoWaveform::Triangle,
        }
    }
}

#[derive(Debug, Clone)]
struct Voice {
    op1: Operator,
    op2: Operator,
    op3: Operator,
    op4: Operator,
    op5: Operator,
    op6: Operator,
    peg: EnvelopeGenerator,  // pitch env
    alg: u8,  // 1...32
    feedback: u8,
    osc_sync: bool,
    lfo: Lfo,
    pitch_mod_sens: u8,
    transpose: u8,  // 12 = C2
    name: String,
    op_flags: [bool; 6],
}

impl Voice {
    /// Creates a new voice and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            op1: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            op2: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            op3: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            op4: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            op5: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            op6: Operator { output_level: RangedValue::new_min(RangeKind::OutputLevel), ..Operator::new() },
            peg: EnvelopeGenerator {
                level1: RangedValue::from_int(RangeKind::Level, 50),
                level2: RangedValue::from_int(RangeKind::Level, 50),
                level3: RangedValue::from_int(RangeKind::Level, 50),
                level4: RangedValue::from_int(RangeKind::Level, 50),
                ..EnvelopeGenerator::new()
            },
            alg: 1,
            feedback: 0,
            osc_sync: true,
            lfo: Lfo::new(),
            pitch_mod_sens: 3,
            transpose: 24,
            name: "INIT VOICE".to_string(),
            op_flags: [true, true, true, true, true, true],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.append(&mut self.op6.to_bytes());
        data.append(&mut self.op5.to_bytes());
        data.append(&mut self.op4.to_bytes());
        data.append(&mut self.op3.to_bytes());
        data.append(&mut self.op2.to_bytes());
        data.append(&mut self.op1.to_bytes());

        data.append(&mut self.peg.to_bytes());

        data.push(self.alg - 1);
        data.push(self.feedback);
        data.push(if self.osc_sync { 1 } else { 0 });
        data.append(&mut self.lfo.to_bytes());
        data.push(self.pitch_mod_sens);
        data.push(self.transpose);

        let padded_name = format!("{:<10}", self.name);
        data.append(&mut padded_name.into_bytes());

        let mut rev_flags = self.op_flags;
        rev_flags.reverse();
        let mut flags: u8 = 0;
        for (index, flag) in rev_flags.iter().enumerate() {
            if *flag {
                flags |= 1 << index
            }
        }
        data.push(flags);

        data
    }

    pub fn to_packed_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        let mut op6_data = self.op6.to_packed_bytes();
        debug!("OP6: {} bytes, {:?}", op6_data.len(), op6_data);
        data.append(&mut op6_data);

        let mut op5_data = self.op5.to_packed_bytes();
        debug!("OP5: {} bytes, {:?}", op5_data.len(), op5_data);
        data.append(&mut op5_data);

        let mut op4_data = self.op4.to_packed_bytes();
        debug!("OP4: {} bytes, {:?}", op4_data.len(), op4_data);
        data.append(&mut op4_data);

        let mut op3_data = self.op3.to_packed_bytes();
        debug!("OP3: {} bytes, {:?}", op3_data.len(), op3_data);
        data.append(&mut op3_data);

        let mut op2_data = self.op2.to_packed_bytes();
        debug!("OP2: {} bytes, {:?}", op2_data.len(), op2_data);
        data.append(&mut op2_data);

        let mut op1_data = self.op1.to_packed_bytes();
        debug!("OP1: {} bytes, {:?}", op1_data.len(), op1_data);
        data.append(&mut op1_data);

        let mut peg_data = self.peg.to_bytes(); // not packed!
        debug!("PEG: {} bytes, {:?}", peg_data.len(), peg_data);
        data.append(&mut peg_data);

        data.push(self.alg);
        debug!("ALG: {}", self.alg);

        let byte111 = self.feedback | ((if self.osc_sync { 1 } else { 0 }) << 3);
        data.push(byte111);
        debug!("  b111: {:#08b}", byte111);

        let mut lfo_data = self.lfo.to_packed_bytes();
        *lfo_data.last_mut().unwrap() |= self.pitch_mod_sens << 5;
        debug!("LFO: {} bytes, {:?}", lfo_data.len(), lfo_data);
        data.append(&mut lfo_data);

        data.push(self.transpose);
        debug!("  TRNSP: {:#02X}", self.transpose);

        let padded_name = format!("{:<10}", self.name);
        debug!("  NAME: '{}'", padded_name);
        data.append(&mut padded_name.into_bytes());

        data
    }
}

/*
impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R1={} L1={} R2={} L2={} R3={} L3={} R4={} L4={}",
            self.rate1.get(), self.level1.get(), self.rate2.get(), self.level2.get(),
            self.rate3.get(), self.level3.get(), self.rate4.get(), self.level4.get())
    }
}
*/

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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
        let eg = EnvelopeGenerator {
            rate1: EnvelopeGenerator::new_rate(64),
            rate2: EnvelopeGenerator::new_rate(64),
            rate3: EnvelopeGenerator::new_rate(64),
            rate4: EnvelopeGenerator::new_rate(64),
            level1: EnvelopeGenerator::new_level(32),
            level2: EnvelopeGenerator::new_level(32),
            level3: EnvelopeGenerator::new_level(32),
            level4: EnvelopeGenerator::new_level(32),
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
            eg: EnvelopeGenerator {
                rate1: EnvelopeGenerator::new_rate(49),
                rate2: EnvelopeGenerator::new_rate(99),
                rate3: EnvelopeGenerator::new_rate(28),
                rate4: EnvelopeGenerator::new_rate(68),
                level1: EnvelopeGenerator::new_level(98),
                level2: EnvelopeGenerator::new_level(98),
                level3: EnvelopeGenerator::new_level(91),
                level4: EnvelopeGenerator::new_level(0),
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 39,
                left_depth: 54,
                right_depth: 50,
                left_curve: ScalingCurve::exp_neg(),
                right_curve: ScalingCurve::exp_neg(),
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: RangedValue::from_int(RangeKind::OutputLevel, 82),
            mode: OperatorMode::Ratio,
            coarse: RangedValue::from_int(RangeKind::Coarse, 1),
            fine: RangedValue::from_int(RangeKind::Fine, 0),
            detune: 0
        };

        let data = op.to_packed_bytes();
        assert_eq!(data.len(), 17);
        assert_eq!(
            data,
            vec![49, 99, 28, 68, 98, 98, 91, 0, 39, 54, 50, 5, 60, 8, 82, 2, 0]
        );
    }

    #[test]
    fn test_lfo_to_packed_bytes() {
        let lfo = Lfo { speed: 37, delay: 0, pmd: 5, amd: 0, sync: false, wave: LfoWaveform::Sine };
        assert_eq!(
            lfo.to_packed_bytes(),
            vec![37, 0, 5, 0, 0b00001000]
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
            Some(offset) => println!("Vectors differ at offset {:?}", offset),
            None => println!("Vectors are the same")
        }

        assert_eq!(brass1_data, voice_data);
    }

    #[test]
    fn test_bulk_b116() {
        let lfo = Lfo { speed: 37, delay: 0, pmd: 5, amd: 0, sync: true, wave: LfoWaveform::Square };
        let pitch_mod_sens = 3u8;  // 0b011
        let sync = true;

        // Index of LFO square waveform is 3 = 0b011

        // |765|4321|0
        // |PMS|LFW |S

        let expected = 0b0_011_011_1;
        let mut actual = (if lfo.sync { 1 } else { 0 }) | ((lfo.wave as u8) << 1);
        actual |= pitch_mod_sens << 4;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bulk_b111() {
        let sync = true;
        let feedback = 7u8;
        let expected = 0x0fu8;
        let mut actual = feedback | ((if sync { 1 } else { 0 }) << 3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ranged_value_from_byte() {
        let level = RangedValue::from_byte(RangeKind::OutputLevel, 16u8);
        assert_eq!(level.get(), 16);
    }

    #[test]
    fn test_ranged_value_from_int() {
        let level = RangedValue::from_int(RangeKind::OutputLevel, 16);
        assert_eq!(level.get(), 16);
    }

    #[test]
    fn test_ranged_value_from_int_clamped() {
        let level = RangedValue::from_int(RangeKind::OutputLevel, 100);

        // The value should be clamped to the end of the range
        assert_eq!(level.get(), 99);
    }

    #[test]
    fn test_ranged_value_as_byte() {
        let level = RangedValue::from_int(RangeKind::OutputLevel, 16);
        assert_eq!(level.as_byte(), 16u8);
    }

    #[test]
    fn test_ranged_value_set() {
        let mut level = RangedValue::from_int(RangeKind::OutputLevel, 16);
        level.set(17);
        assert_eq!(level.get(), 17);
    }

    #[test]
    fn test_ranged_value_set_clamped() {
        let mut level = RangedValue::from_int(RangeKind::OutputLevel, 16);
        level.set(-100);  // deliberately out of the allowed range

        // The value should be clamped to the start of the range
        assert_eq!(level.get(), 0);
    }

    #[test]
    fn test_ranged_value_range() {
        let level = RangedValue::from_int(RangeKind::OutputLevel, 16);
        assert_eq!(level.range().start(), &0);
        assert_eq!(level.range().end(), &99);
    }

    #[test]
    fn test_ranged_value_min() {
        let value = RangedValue::new_min(RangeKind::OutputLevel);
        assert_eq!(value.get(), 0);
    }

    #[test]
    fn test_ranged_value_max() {
        let value = RangedValue::new_max(RangeKind::OutputLevel);
        assert_eq!(value.get(), 99);
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
