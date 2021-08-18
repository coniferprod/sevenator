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
level similarly has its own member in the `EnvelopeGenerator` struct. In my opinion,
this makes the code significantly easier to read and write, compared to traditional
zero-based array/vector indexing. It is more intuitive to write `op1.level` than
`op[0].level`.

### The `RangedValue` data type

To make it easier to ensure that parameter values are restricted to the allowed range,
and to be able to generate random parameter values (for metric parameters),
the `RangedValue` data type is used, together with the `RangeKind` enum, which gives
semantic meaning to the values. The `RangedValue` struct could use Rust's range, if not
for the fact that [Rust ranges are not `Copy`](https://github.com/rust-lang/rfcs/issues/2848),
so we can't pass `RangedValue` instances around casually if we use the standard range.
That is why the `RangedValue` struct uses a wrapper type, `RangeInclusiveWrapper`. It wraps
an `i16` value with the start and end of the allowable range. `i16` is used as the base
type because it is the smallest data type that can fit all the metric parameter values
(some of them extend to negative values).
