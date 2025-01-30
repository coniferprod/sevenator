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

pub struct PerformanceMemory {
    pub performances: [Performance; 64],
}
