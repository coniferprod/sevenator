use std::fs::File;
use std::io::prelude::*;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn, error, debug};

fn make_brass1() -> Voice {
    let op6 = Operator {
        eg: EnvelopeGenerator {
            rate1: 49,
            rate2: 99,
            rate3: 28,
            rate4: 68,
            level1: 99,
            level2: 98,
            level3: 91,
            level4: 0,
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60,
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
        output_level: 82,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0, detune: 0
    };

    let op5 = Operator {
        eg: EnvelopeGenerator {
            rate1: 79, rate2: 36, rate3: 41, rate4: 71,
            level1: 99, level2: 98, level3: 98, level4: 0
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: 98,
        mode: OperatorMode::Ratio,
        coarse: 1, fine: 0, detune: 0
    };

    let op4 = Operator {
        eg: EnvelopeGenerator {
            rate1: 79, rate2: 36, rate3: 41, rate4: 71,
            level1: 99, level2: 98, level3: 98, level4: 0
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: 99,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0,
        detune: 0
    };

    let op3 = Operator {
        eg: EnvelopeGenerator {
            rate1: 79, rate2: 36, rate3: 41, rate4: 71,
            level1: 99, level2: 98, level3: 98, level4: 0
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: 99,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0,
        detune: 0
    };

    let op2 = Operator {
        eg: EnvelopeGenerator {
            rate1: 79, rate2: 36, rate3: 41, rate4: 71,
            level1: 99, level2: 98, level3: 98, level4: 0
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: 99,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0,
        detune: 0
    };

    let op1 = Operator {
        eg: EnvelopeGenerator {
            rate1: 79, rate2: 36, rate3: 41, rate4: 71,
            level1: 99, level2: 98, level3: 98, level4: 0
        },
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: 60, left_depth: 0, right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
        },
        kbd_rate_scaling: 4,
        amp_mod_sens: 0,
        key_vel_sens: 2,
        output_level: 99,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0,
        detune: 0
    };

    Voice {
        op1: op1,
        op2: op2,
        op3: op3,
        op4: op4,
        op5: op5,
        op6: op6,
        peg: EnvelopeGenerator {
            rate1: 84, rate2: 95, rate3: 95, rate4: 60,
            level1: 50, level2: 50, level3: 50, level4: 50
        },
        alg: 21,  // algorithm 22 - 1
        feedback: 7,
        osc_sync: false,
        lfo: LFO { speed: 37, delay: 0, pmd: 5, amd: 0, sync: false, wave: LFOWaveform::Triangle },
        pitch_mod_sens: 3,
        transpose: 60,
        name: "BRASS 1".to_string(),
        op_flags: [true, true, true, true, true, true],
    }
}

// Makes an initialized voice. The defaults are as described in
// Howard Massey's "The Complete DX7", Appendix B.
fn make_init_voice() -> Voice {
    let init_eg = EnvelopeGenerator {
        rate1: 99, rate2: 99, rate3: 99, rate4: 99,
        level1: 99, level2: 99, level3: 99, level4: 0,
    };

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
        output_level: 99,
        mode: OperatorMode::Ratio,
        coarse: 1,
        fine: 0,
        detune: 0
    };

    let init_op_rest = Operator {
        output_level: 0,
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
            rate1: 99, rate2: 99, rate3: 99, rate4: 99,
            level1: 50, level2: 50, level3: 50, level4: 50,
        },
        alg: 0, // algorithm #1
        feedback: 0,
        osc_sync: true, // osc key sync = on
        lfo: LFO {
            speed: 35,
            delay: 0,
            pmd: 0,
            amd: 0,
            sync: true,
            wave: LFOWaveform::Triangle,
        },
        pitch_mod_sens: 3,
        transpose: 60, // Middle C = C3 (Yamaha style, others call this C4)
        name: "INIT VOICE".to_string(),
        op_flags: [true, true, true, true, true, true],  // all operators ON
    }
}

pub fn run() -> std::io::Result<()> {
    //let brass1 = make_brass1();

    // Get the default voice with `Voice::new()`.
    // The `make_init_voice()` function makes exactly the original init voice.
    // These should be more or less the same.
    let cartridge = vec![make_init_voice(); 32];
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
    rate1: u8,
    rate2: u8,
    rate3: u8,
    rate4: u8,
    level1: u8,
    level2: u8,
    level3: u8,
    level4: u8
}

impl EnvelopeGenerator {
    /// Creates a new EG with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            rate1: 99, rate2: 99, rate3: 99, rate4: 99,
            level1: 99, level2: 99, level3: 99, level4: 0,
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
            rate1: attack, rate2: 99, rate3: decay, rate4: release,
            level1: 99, level2: 99, level3: sustain, level4: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.rate1, self.rate2, self.rate3, self.rate4,
            self.level1, self.level2, self.level3, self.level4
        ]
    }
}

impl fmt::Display for EnvelopeGenerator {
    // The display trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R1={} L1={} R2={} L2={} R3={} L3={} R4={} L4={}",
            self.rate1, self.rate2, self.rate3, self.rate4,
            self.level1, self.level2, self.level3, self.level4)
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
    pub fn to_bytes(&self) -> u8 {
        match self {
            ScalingCurve { curve: CurveStyle::Linear, positive: true } => 1,
            ScalingCurve { curve: CurveStyle::Linear, positive: false } => 0,
            ScalingCurve { curve: CurveStyle::Exponential, positive: true } => 3,
            ScalingCurve { curve: CurveStyle::Exponential, positive: false } => 2,
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

impl KeyboardLevelScaling {
    /// Creates new keyboard level scaling settings with DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            breakpoint: 0,  // A-1
            left_depth:  0,
            right_depth: 0,
            left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
            right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: false },
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
    kbd_rate_scaling: u8,
    amp_mod_sens: u8,  // 0 ~ 3
    key_vel_sens: u8,  // 0 ~ 7
    output_level: u8,
    mode: OperatorMode,
    coarse: u8,  // 0 ~ 31
    fine: u8,  // 0 ~ 99
    detune: u8,   // 0 ~ 14
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
            output_level: 0,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0,  // TODO: voice init for fine is "1.00 for all operators", should this be 0 or 1?
            detune: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.append(&mut self.eg.to_bytes());
        data.append(&mut self.kbd_level_scaling.to_bytes());
        data.push(self.kbd_rate_scaling);
        data.push(self.amp_mod_sens);
        data.push(self.key_vel_sens);
        data.push(self.output_level);
        data.push(self.mode as u8);
        data.push(self.coarse);
        data.push(self.fine);
        data.push(self.detune);
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

        let byte12 = self.kbd_rate_scaling | (self.detune << 3);
        debug!("  b12: {:#08b}", byte12);
        data.push(byte12);

        let byte13 = self.amp_mod_sens | (self.key_vel_sens << 2);
        debug!("  b13: {:#08b}", byte12);
        data.push(byte13);

        debug!("  OL:  {:#08b}", self.output_level);
        data.push(self.output_level);

        let byte15 = self.mode as u8 | (self.coarse << 1);
        debug!("  b15: {:#08b}", byte15);
        data.push(byte15);

        debug!("  FF:  {:#08b}", self.fine);
        data.push(self.fine);

        data
    }
}

#[derive(Debug, Copy, Clone)]
enum LFOWaveform {
    Triangle,
    SawDown,
    SawUp,
    Square,
    Sine,
    SampleAndHold,
}

#[derive(Debug, Clone)]
struct LFO {
    speed: u8,  // 0 ~ 99
    delay: u8,  // 0 ~ 99
    pmd: u8,    // 0 ~ 99
    amd: u8,    // 0 ~ 99
    sync: bool,
    wave: LFOWaveform,
}

impl LFO {
    // Initialize with the DX7 voice defaults
    pub fn new() -> Self {
        Self {
            speed: 35,
            delay: 0,
            pmd: 0,
            amd: 0,
            sync: true,
            wave: LFOWaveform::Triangle,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.speed,
            self.delay,
            self.pmd,
            self.amd,
            if self.sync { 1 } else { 0 },
            self.wave as u8,
        ]
    }

    pub fn to_packed_bytes(&self) -> Vec<u8> {
        vec![
            self.speed,
            self.delay,
            self.pmd,
            self.amd,
            (if self.sync { 1 } else { 0 }) | ((self.wave as u8) << 1),
        ]
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
    lfo: LFO,
    pitch_mod_sens: u8,
    transpose: u8,
    name: String,
    op_flags: [bool; 6],
}

impl Voice {
    /// Creates a new voice and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            op1: Operator { output_level: 0, ..Operator::new() },
            op2: Operator { output_level: 0, ..Operator::new() },
            op3: Operator { output_level: 0, ..Operator::new() },
            op4: Operator { output_level: 0, ..Operator::new() },
            op5: Operator { output_level: 0, ..Operator::new() },
            op6: Operator { output_level: 0, ..Operator::new() },
            peg: EnvelopeGenerator { level1: 50, level2: 50, level3: 50, level4: 50, ..EnvelopeGenerator::new() },
            alg: 0,
            feedback: 0,
            osc_sync: true,
            lfo: LFO::new(),
            pitch_mod_sens: 3,
            transpose: 0,  // Massey book says "Middle C = C3", so I guess it's zero then?
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

        data.push(self.alg);
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

        //let (all_but_last, last) = lfo_data.split_at(lfo_data.len() - 1);
        //data.append(&mut all_but_last.to_vec());  // leave the last byte out for now

        // Add to the last byte the pitch modulation sensitivity
        //let mut last_byte = last[0];  // this slice has only this one byte
        //last_byte |= self.pitch_mod_sens << 4;  // somewhat unclear if shift by 4 or 5
        //data.push(last_byte);

        data.push(self.transpose);
        debug!("  TRNSP: {:#02X}", self.transpose);

        let padded_name = format!("{:<10}", self.name);
        debug!("  NAME: '{}'", padded_name);
        data.append(&mut padded_name.into_bytes());

        data
    }
}

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
            rate1: 64,
            rate2: 64,
            rate3: 64,
            rate4: 64,
            level1: 32,
            level2: 32,
            level3: 32,
            level4: 32,
        };
        assert_eq!(eg.to_bytes(), vec![64u8, 64, 64, 64, 32, 32, 32, 32]);
    }

    #[test]
    fn test_scaling_curve_exp_pos_to_bytes() {
        let curve_exp_pos = ScalingCurve {
            curve: CurveStyle::Exponential,
            positive: true
        };
        assert_eq!(curve_exp_pos.to_bytes(), 3);
    }

    #[test]
    fn test_scaling_curve_exp_neg_to_bytes() {
        let curve_exp_neg = ScalingCurve {
            curve: CurveStyle::Exponential,
            positive: false
        };
        assert_eq!(curve_exp_neg.to_bytes(), 2);
    }

    #[test]
    fn test_scaling_curve_lin_pos_to_bytes() {
        let curve_lin_pos = ScalingCurve {
            curve: CurveStyle::Linear,
            positive: true
        };
        assert_eq!(curve_lin_pos.to_bytes(), 1);
    }

    #[test]
    fn test_scaling_curve_lin_neg_to_bytes() {
        let curve_lin_neg = ScalingCurve {
            curve: CurveStyle::Linear,
            positive: false
        };
        assert_eq!(curve_lin_neg.to_bytes(), 0);
    }

    #[test]
    fn test_kbd_level_scaling_to_packed_bytes() {
        let ks = KeyboardLevelScaling {
            breakpoint: 60,
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
        };

        assert_eq!(
            ks.to_packed_bytes(),
            vec![60, 54, 50, 0b00001010]
        )
    }

    #[test]
    fn test_op_to_packed_bytes() {
        let op = Operator {
            eg: EnvelopeGenerator {
                rate1: 49,
                rate2: 99,
                rate3: 28,
                rate4: 68,
                level1: 99,
                level2: 98,
                level3: 91,
                level4: 0,
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60,
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
            output_level: 82,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0, detune: 0
        };

        let data = op.to_packed_bytes();
        assert_eq!(
            data,
            vec![49, 99, 28, 68, 99, 98, 91, 0, 60, 54, 50, 0b00001010, 0b00000100, 0b0001000, 82, 0b0000010, 0]
        );
        assert_eq!(data.len(), 17);
    }

    #[test]
    fn test_lfo_to_packed_bytes() {
        let lfo = LFO { speed: 37, delay: 0, pmd: 5, amd: 0, sync: false, wave: LFOWaveform::Sine };
        assert_eq!(
            lfo.to_packed_bytes(),
            vec![37, 0, 5, 0, 0b00001000]
        );
    }

    #[test]
    fn test_voice_to_packed_bytes() {
        let op6 = Operator {
            eg: EnvelopeGenerator {
                rate1: 49,
                rate2: 99,
                rate3: 28,
                rate4: 68,
                level1: 99,
                level2: 98,
                level3: 91,
                level4: 0,
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60,
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
            output_level: 82,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0, detune: 0
        };

        let op5 = Operator {
            eg: EnvelopeGenerator {
                rate1: 79, rate2: 36, rate3: 41, rate4: 71,
                level1: 99, level2: 98, level3: 98, level4: 0
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60, left_depth: 0, right_depth: 0,
                left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
                right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: 98,
            mode: OperatorMode::Ratio,
            coarse: 1, fine: 0, detune: 0
        };

        let op4 = Operator {
            eg: EnvelopeGenerator {
                rate1: 79, rate2: 36, rate3: 41, rate4: 71,
                level1: 99, level2: 98, level3: 98, level4: 0
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60, left_depth: 0, right_depth: 0,
                left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
                right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: 99,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0,
            detune: 0
        };

        let op3 = Operator {
            eg: EnvelopeGenerator {
                rate1: 79, rate2: 36, rate3: 41, rate4: 71,
                level1: 99, level2: 98, level3: 98, level4: 0
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60, left_depth: 0, right_depth: 0,
                left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
                right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: 99,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0,
            detune: 0
        };

        let op2 = Operator {
            eg: EnvelopeGenerator {
                rate1: 79, rate2: 36, rate3: 41, rate4: 71,
                level1: 99, level2: 98, level3: 98, level4: 0
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60, left_depth: 0, right_depth: 0,
                left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
                right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: 99,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0,
            detune: 0
        };

        let op1 = Operator {
            eg: EnvelopeGenerator {
                rate1: 79, rate2: 36, rate3: 41, rate4: 71,
                level1: 99, level2: 98, level3: 98, level4: 0
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: 60, left_depth: 0, right_depth: 0,
                left_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
                right_curve: ScalingCurve { curve: CurveStyle::Linear, positive: true },
            },
            kbd_rate_scaling: 4,
            amp_mod_sens: 0,
            key_vel_sens: 2,
            output_level: 99,
            mode: OperatorMode::Ratio,
            coarse: 1,
            fine: 0,
            detune: 0
        };

        let brass1 = Voice {
            op1: op1,
            op2: op2,
            op3: op3,
            op4: op4,
            op5: op5,
            op6: op6,
            peg: EnvelopeGenerator {
                rate1: 84, rate2: 95, rate3: 95, rate4: 60,
                level1: 50, level2: 50, level3: 50, level4: 50
            },
            alg: 21,  // algorithm 22 - 1
            feedback: 7,
            osc_sync: false,
            lfo: LFO { speed: 37, delay: 0, pmd: 5, amd: 0, sync: false, wave: LFOWaveform::Sine },
            pitch_mod_sens: 3,
            transpose: 60,
            name: "BRASS 1".to_string(),
            op_flags: [true, true, true, true, true, true],
        };

        assert_eq!(
            brass1.to_packed_bytes(),
            vec![
                // op6
                49, 99, 28, 68, 99, 98, 91, 0,
                60, 54, 50, 0b00001010, 0b00000100, 0b0001000, 82, 0b0000010, 0,

                // op5
                79, 36, 41, 71, 99, 98, 98, 0,
                60, 0, 0, 0b00000101, 0b00000100, 0b0001000, 98, 0b0000010, 0,

                // op4
                79, 36, 41, 71, 99, 98, 98, 0,
                60, 0, 0, 0b00000101, 0b00000100, 0b0001000, 99, 0b0000010, 0,

                // op3
                79, 36, 41, 71, 99, 98, 98, 0,
                60, 0, 0, 0b00000101, 0b00000100, 0b0001000, 99, 0b0000010, 0,

                // op2
                79, 36, 41, 71, 99, 98, 98, 0,
                60, 0, 0, 0b00000101, 0b00000100, 0b0001000, 99, 0b0000010, 0,

                // op1
                79, 36, 41, 71, 99, 98, 98, 0,
                60, 0, 0, 0b00000101, 0b00000100, 0b0001000, 99, 0b0000010, 0,

                84, 95, 95, 60, 50, 50, 50, 50,  // peg
                21,  // alg
                0b00000111,  // byte111
                37, 0, 5, 0,  // LFO

                // byte116: mod sens pitch = 3 = 0b11, LFO wave = Sine = 4 = 0b100, sync = 0 = 0b0
                // whole byte should be LPMS=bits 5-6, LFW=bits 1-4, sync = bit 0,
                // but LPMS is 0~7 and needs three bits, whereas LFW is 0~4 and needs only three bits.
                // Actually there are six waveforms for the LFO...
                0b01101000, // byte116  0b00001000
                60,  // transpose

                 // "BRASS 1" padded from right with spaces to 10 characters
                0x42, 0x52, 0x41, 0x53, 0x53, 0x20, 0x31, 0x20, 0x20, 0x20,
            ]
        )
    }

    #[test]
    fn test_bulk_b116() {
        let lfo = LFO { speed: 37, delay: 0, pmd: 5, amd: 0, sync: true, wave: LFOWaveform::Square };
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
