/// Network interface information
#[derive(Debug, Clone, Default)]
pub struct NetworkInfo {
    pub present: bool,
    pub mac_address: [u8; 6],
    pub media_header_size: u32,
    pub max_packet_size: u32,
    pub state: NetworkState,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum NetworkState {
    #[default]
    Unknown,
    Started,
    Initialized,
    Stopped,
}

/// JSON payload for device identity report
pub struct DeviceReport {
    pub device_guid: [u8; 16],
    pub bios_vendor: heapless::String<64>,
    pub bios_version: heapless::String<64>,
    pub system_manufacturer: heapless::String<64>,
    pub system_product: heapless::String<64>,
    pub baseboard_manufacturer: heapless::String<64>,
    pub baseboard_product: heapless::String<64>,
    pub cpu_signature: u32,
    pub tpm_present: bool,
    pub mac_address: [u8; 6],
    pub identity_hash: [u8; 32],
}
