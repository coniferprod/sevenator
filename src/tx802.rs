use bit::BitIndex;
use syxpack::{Message, Manufacturer, ManufacturerId};
use crate::{Byte, ByteVector, SystemExclusiveData};

pub struct Performance {
    pub voice_channel_offset: u8,
    pub receive_channel: u8,
    pub voice_number: u8,
    pub micro_tuning_table_number: u8,
    pub output_volume: u8,
    pub output_assign: u8,
    pub note_limit_low: u8,
    pub note_limit_high: u8,
    pub note_shift: u8,
    pub eg_forced_damp: u8,
    pub name: String,
}

trait SystemExclusiveData for Performance {
    /// Makes a performance from relevant SysEx message bytes.
    fn from_bytes(data: ByteVector) -> Self {
        // TODO: Parse the bytes
        Performance
    }

    /// Makes a performance from packed SysEx message bytes.
    fn from_packed_bytes(data: ByteVector) -> Self {
        Envelope::from_bytes(data)
    }

    fn to_packed_bytes(&self) -> ByteVector {

        self.to_bytes()
    }

    /// Gets the SysEx bytes of this EG.
    fn to_bytes(&self) -> ByteVector {
        self.to_packed_bytes()
    }

    fn data_size(&self) -> usize { 168 }
}

pub struct PerformanceMemory {
    pub performances: [Performance; 64],
}
