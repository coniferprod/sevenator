# Sevenator

DX7 cartridge generator tool

Sevenator generates a single voice or a "cartridge" of 32 patches
for the [Yamaha DX7](https://www.yamaha.com/en/about/design/synapses/id_009/)
as a bulk dump file in MIDI System Exclusive format. You can use it to generate patches for the
original DX7 synthesizer and also emulations and/or clones, such as

* [Arturia DX7 V](https://www.arturia.com/dx7-v/overview),
* [Native Instruments FM8](https://www.native-instruments.com/en/products/komplete/synths/fm8/),
* [Dexed](https://asb2m10.github.io/dexed/)
* [Plogue Chipsynth OPS7](https://www.plogue.com/products/chipsynth-ops7.html)

The [KORG opsix](https://www.korg.com/us/products/synthesizers/opsix/) can also
import DX7 bulk dump files by MIDI.

Sevenator implements the Yamaha DX7 patch file data model, so you can create
programs that generate patches in any way you can think of. For example, you
can randomize every parameter, but you will get better results if you select
a subset of the parameters to randomize.

Currently Sevenator generates either a single voice (initialized to factory defaults, or
completely random) or a "cartridge" with the same 32 patches
repeated over, with some random envelopes, but it may get some patch generation
methods of its own as it is developed. The DX7 data model and API may also be split
into a dedicated package, which would make Sevenator just one client of the API.

## The Yamaha DX7 patch format

The Yamaha DX7 patch format is well documented in the DX7 Owner's Manual,
available online in the Yamaha Manual Library. A description of the packed format
can be found in the [Dexed documentation](https://github.com/asb2m10/dexed/blob/master/Documentation/sysex-format.txt).

The Yamaha DX7 cartridge data is 4,096 bytes long (not counting the System
Exclusive header and terminator, which bring it up to 4,104 bytes). It contains
packed data for 32 voices, so the data for one voice is 128 bytes. An unpacked
single voice is 155 bytes (with SysEx header 163 bytes).

## Rust considerations

### Operators and EGs

A Yamaha DX7 voice has six operators, while an envelope generator has four rates and
four levels. These numbers are never going to change when dealing with DX7 data.
That is why each operator has its own member in the `Voice` struct, and each rate and
level similarly has its own member in the `Envelope` struct. In my opinion,
this makes the code significantly easier to read and write, compared to traditional
zero-based array/vector indexing. It is more intuitive to write `op1.level` than
`op[0].level`.

### The newtype pattern

The data types of some struct members are defined using the newtype pattern in Rust.

Each voice parameter has an allowed range of values. For example, operator levels
go from 0 to 99 inclusive, detune values are -7 to +7, and so on.

To be able to catch or suppress errors in setting parameter values, I wanted to have
a data type that would restrict its values to a given range, and possibly clamp any
value that falls outside the range. Also, it would convenient to create random values
for a parameter, and maybe also restrict those random values into a subrange.

In Rust, a newtype is "a struct with a single component that you define to get stricter
type checking" ("Programming Rust, 2nd Edition", p. 213). As with any struct, it is
possible to define traits for the newtype. I defined a newtype for every relevant
parameter value, such as `UnsignedLevel` and `Detune`, and defined a simple interface
that allows me to make new values and retrieve them, and also get a byte representation
for System Exclusive messages.

Each newtype is backed by an `i32` value. The parameter values would fit into an `i16`,
but since `i32` is the integer type inferred by default, it is much more convenient
to use. For example, the value of the detune parameter ranges from -7 to 7.
It is represented in System Exclusive messages as a value from 0 to 14.

### Constructing parameter values

Now, when I have newtype like `Detune`, I can implement a method that returns the
range:

    #[derive(Debug, Clone, Copy)]
    pub struct Detune(i32);

    impl Detune {
        pub fn range() -> RangeInclusive<i32> {
            RangeInclusive::new(-7, 7)
        }
    }

When a new `Detune` struct is constructed, the tentative value is checked against
the range:

    impl Detune {
        pub fn new(value: i32) -> Self {
            let range = Detune::range();
            if range.contains(&value) {
                Detune(value)
            }
            else {
                if Self::is_clamped() {
                    Detune(num::clamp(value, *range.start(), *range.end()))
                }
                else {
                    panic!("expected value in range {}...{}, got {}", *range.start(), *range.end(), value);
                }
            }
        }
    }

If the value is out of range, it gets clamped, using the `clamp` function in the
`num` crate. The clamping is controlled by the `is_clamped` function:

    impl Detune {
        pub fn is_clamped() -> bool {
            return true
        }
    }

## Patch data structure

Packed data format:

    0x31, 0x63, 0x1c, 0x44,  // OP6 EG rates
    0x62, 0x62, 0x5b, 0x00,  // OP6 EG levels
    0x27,                    // level scl brkpt
    0x36,                    // scl left depth
    0x32,                    // scl right depth
    0x05,                    // scl left curve and right curve
    0x3c,                    // osc detune and osc rate scale
    0x08,                    // key vel sens and amp mod sens
    0x52,                    // OP6 output level
    0x02,                    // freq coarse and osc mode
    0x00,                    // freq fine
    0x4d, 0x24, 0x29, 0x47,  // same for OP5
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x40,
    0x08,
    0x62,
    0x02,
    0x00,
    0x4d, 0x24, 0x29, 0x47,  // OP4
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x38,
    0x08,
    0x63,
    0x02,
    0x00,
    0x4d, 0x4c, 0x52, 0x47,  // OP3
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x28,
    0x08,
    0x63,
    0x02,
    0x00,
    0x3e, 0x33, 0x1d, 0x47,  // OP2
    0x52, 0x5f, 0x60, 0x00,
    0x1b,
    0x00,
    0x07,
    0x07,
    0x70,
    0x00,
    0x56,
    0x00,
    0x00,
    0x48, 0x4c, 0x63, 0x47,  // OP1
    0x63, 0x58, 0x60, 0x00,
    0x27,
    0x00,
    0x0e,
    0x0f,
    0x70,
    0x00,
    0x62,
    0x00,
    0x00,
    0x54, 0x5f, 0x5f, 0x3c,  // PEG rates
    0x32, 0x32, 0x32, 0x32,  // PEG levels
    0x15,                    // ALG
    0x0f,                    // osc key sync and feedback
    0x25,                    // LFO speed
    0x00,                    // LFO delay
    0x05,                    // LFO pitch mod dep
    0x00,                    // LFO amp mod dep
    0x38,                    // LFO pitch mode sens, wave, sync
    0x18,                    // transpose
    0x42, 0x52, 0x41, 0x53, 0x53, 0x20, 0x20, 0x20, 0x31, 0x20,   // name (10 characters)

Note that there seems to be an error in the DX7 packed format description.
I couldn't have made this without the information found therein, but the packed LFO caused
some trouble. The document states describes byte 116 of the packed format like this:

    byte             bit #
    #     6   5   4   3   2   1   0   param A       range  param B       range
    ----  --- --- --- --- --- --- ---  ------------  -----  ------------  -----
    116  |  LPMS |      LFW      |LKS| LF PT MOD SNS 0-7   WAVE 0-5,  SYNC 0-1

Actually it seems to be like this:

    byte             bit #
    #     6   5   4   3   2   1   0   param A       range  param B       range
    ----  --- --- --- --- --- --- ---  ------------  -----  ------------  -----
    116  |   LPMS    |  LFW      |LKS| LF PT MOD SNS 0-7   WAVE 0-5,  SYNC 0-1

The LFO pitch modulation sensitivity value (three bits, 0...7) is in bits 4...6,
and the LFO waveform (four bits, 0...5) is in bits 1...3.

I cross-checked this with the "BRASS 1" patch from the original ROM1 cartridge data.
The corresponding byte in the original data is 0x38 = 0b00111000, which parses to
sync = false, LFO waveform = 4 or sine, and pitch mod sens = 3. These match the
patch chart on page 28 of the DX7 Operating Manual.

## Yamaha TX802 notes

The Yamaha TX802 voice edit buffer accepts single voices in DX7 format.

## XML documents

The voices and cartridges have an XML Schema definition (subject to change).
To validate an XML document, use xmllint like so:

    xmllint --xinclude --schema cartridge.xsd testcartridge.xml

This will process the XInclude elements first, and then validate the result
against the XML Schema.

See also [XML Parsing in Rust](https://mainmatter.com/blog/2020/12/31/xml-and-rust/).
