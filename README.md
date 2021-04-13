# Sevenator

DX7 cartridge generator tool

Sevenator generates a set of 32 patches for the Yamaha DX7 as a bulk dump file
in MIDI System Exclusive format. You can use it to generate patches for the
original DX7 synthesizer and also emulations and/or clones, such as Arturia DX7 V,
Native Instruments FM8, or Dexed, just to name a few. The KORG opsix can also
import DX7 bulk dump files by MIDI.

Sevenator implements the Yamaha DX7 patch file data model, so you can create
programs that generate patches in any way you can think of. For example, you
can randomize every parameter, but you will get better results if you select
a subset of the parameters to randomize.

Currently Sevenator only generates a bank or "cartridge" with the same 32 patches
repeated over, but it may get some patch generation methods of its own as it
is developed. The DX7 data model and API may also be split into a dedicated
package, so that Sevenator will become just one client of the API.