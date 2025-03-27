# Sevenator

Sevenator generates a single voice or a "cartridge" of 32 patches
for the [Yamaha DX7](https://www.yamaha.com/en/about/design/synapses/id_009/)
as a bulk dump file in MIDI System Exclusive format. You can use it to generate patches for the
original DX7 synthesizer and also emulations and/or clones, such as

* [Arturia DX7 V](https://www.arturia.com/dx7-v/overview),
* [Native Instruments FM8](https://www.native-instruments.com/en/products/komplete/synths/fm8/),
* [Dexed](https://asb2m10.github.io/dexed/)
* [Plogue Chipsynth OPS7](https://www.plogue.com/products/chipsynth-ops7.html)
* and many others

The [KORG opsix](https://www.korg.com/us/products/synthesizers/opsix/) can also
import DX7 bulk dump files by MIDI.

Sevenator implements the Yamaha DX7 patch file data model, so you can create
programs that generate patches in any way you can think of. For example, you
can randomize every parameter, but you will get better results if you select
a subset of the parameters to randomize.

Currently Sevenator generates either a single voice (initialized to factory defaults, or
completely random) or a "cartridge" with the same 32 patches
repeated over, with some random envelopes, but it may get some patch generation
methods of its own as it is developed. 

The DX7 data model and its API have been split
into a dedicated package [sevenate-rs](https://github.com/coniferprod/sevenate-rs), so Sevenator is just one possible client of the API.

## Building the program

Sevenator is written in Rust. You can use the Rust development tools to make
an executable version for your (supprted) operating system. See [the Rust home page](https://www.rust-lang.org) for details about installation.

Clone the source code from the Git repository into a subdirectory, and then
issue the command

    cargo build

If you want to make a release build (smaller, no debug information included), use

    cargo build --release

You will find the resulting executable in the `target/` subdirectory, under
`debug` or `release`.

## Running the program

To run Sevenator, either copy the executable to a directory of your choice,
or issue the command

    cargo run

Sevenator will show you its command line options.

For example, to convert a Yamaha DX7 cartridge in MIDI System Exclusive format
to the XML format you use the `make-xml` subcommand. To show its options, use

    sevenator help make-xml

You should see the following:

    Usage: sevenator make-xml --input-file <INPUT_FILE> --output-file <OUTPUT_FILE>

    Options:
      -i, --input-file <INPUT_FILE>    
      -o, --output-file <OUTPUT_FILE>  
      -h, --help                       Print help

So, for example, to convert the original ROM 1A cartridge of the Yamaha DX7 
to the XML format, use the command

    sevenator make-xml --input-file ROM1A.SYX --output-file rom1a.xml


## The Yamaha DX7 patch format

The Yamaha DX7 patch format is well documented in the DX7 Owner's Manual,
available online in the Yamaha Manual Library. A description of the packed format
can be found in the [Dexed documentation](https://github.com/asb2m10/dexed/blob/master/Documentation/sysex-format.txt).

The Yamaha DX7 cartridge data is 4,096 bytes long (not counting the System
Exclusive initiator and terminator, and the dump header, which bring it up to 4,104 bytes). 
It contains packed data for 32 voices, so the data for one voice is 128 bytes. An unpacked
single voice is 155 bytes (with SysEx header 163 bytes).

## Yamaha TX802 notes

The Yamaha TX802 voice edit buffer accepts single voices in DX7 format.

## XML documents

The voices and cartridges have an XML Schema definition (subject to change).
To validate an XML document, use xmllint like so:

    xmllint --xinclude --schema cartridge.xsd testcartridge.xml

This will process the XInclude elements first, and then validate the result
against the XML Schema.

See also [XML Parsing in Rust](https://mainmatter.com/blog/2020/12/31/xml-and-rust/).
