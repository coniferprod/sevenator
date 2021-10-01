# Sevenator

DX7 cartridge generator tool

Sevenator generates a set of 32 patches for the [Yamaha DX7](https://www.yamaha.com/en/about/design/synapses/id_009/)
as a bulk dump file in MIDI System Exclusive format. You can use it to generate patches for the
original DX7 synthesizer and also emulations and/or clones, such as
[Arturia DX7 V](https://www.arturia.com/dx7-v/overview),
[Native Instruments FM8](https://www.native-instruments.com/en/products/komplete/synths/fm8/),
or [Dexed](https://asb2m10.github.io/dexed/), just to name a few.
The [KORG opsix](https://www.korg.com/us/products/synthesizers/opsix/) can also
import DX7 bulk dump files by MIDI.

Sevenator implements the Yamaha DX7 patch file data model, so you can create
programs that generate patches in any way you can think of. For example, you
can randomize every parameter, but you will get better results if you select
a subset of the parameters to randomize.

Currently Sevenator only generates a bank or "cartridge" with the same 32 patches
repeated over, or with some random envelopes, but it may get some patch generation
methods of its own as it
is developed. The DX7 data model and API may also be split into a dedicated
package, so that Sevenator will become just one client of the API.

## The Yamaha DX7 patch format

The Yamaha DX7 patch format is well documented in the DX7 Owner's Manual,
available online in the Yamaha Manual Library. A description of the packed format
can be found in the [Dexed documentation](https://github.com/asb2m10/dexed/blob/master/Documentation/sysex-format.txt).

The Yamaha DX7 cartridge data is 4,096 bytes long (not counting the System
Exclusive header and terminator, which bring it up to 4,104 bytes). It contains
packed data for 32 voices, so the data for one voice is 128 bytes.

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

For this I couldn't use a primitive Rust data type, since the range of even the smallest
integer type, `i8`, was larger than the smallest value range. (Of course it was close;
you could use an `i8` or a `u8` type for the parameters, but you would need to handle
the values that fall outside the range anyway).

In Rust, a newtype is "a struct with a single component that you define to get stricter
type checking" ("Programming Rust, 2nd Edition", p. 213). As with any struct, it is
possible to define traits for the newtype. I defined a newtype for every relevant
parameter value, such as `UnsignedLevel` and `Detune`, and defined a simple interface
that allows me to make new values and retrieve them, and also get a byte representation
for System Exclusive messages.

For example, the value of the detune parameter ranges from -7 to 7. It is represented
in System Exclusive messages as a value from 0 to 14. The single component of the newtype
for `Detune` is an `i8`.

### Wrapper

The newtypes need a range for the allowed values. I could not use the standard Rust
range, because it "is not `Copy`", meaning that it doesn't implement the `Copy` trait.
Since I wanted to use these parameter values in structs that are `Copy`, I had to
make my own wrapper type with the start and end of the allowed value range.

The `Wrapper` type is generic:

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct Wrapper<T> where T: Ord {
        start: T,
        end: T,
    }

### Constructing parameter values

Now, when I have newtype like `Detune`, I can implement a method that returns the
range:

    #[derive(Debug, Clone, Copy)]
    pub struct Detune(i8);

    impl Detune {
        fn range() -> Wrapper<i8> {
            Wrapper { start: -7, end: 7 }
        }
    }

When a new `Detune` struct is constructed, the tentative value is checked against
the range:

    impl Detune {
        pub fn new(value: i8) -> Detune {
           let range = Detune::range();
            Detune(num::clamp(value, range.start, range.end))
        }
    }

If the value is out of range, it gets clamped, using the `clamp` function in the
`num` crate.


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
I couldn't have made this without that information, but the packed LFO caused
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
