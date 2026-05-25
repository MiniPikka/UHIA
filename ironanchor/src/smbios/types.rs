/// SMBIOS structure types we parse

pub const BIOS_INFO_TYPE: u8 = 0;
pub const SYSTEM_INFO_TYPE: u8 = 1;
pub const BASEBOARD_INFO_TYPE: u8 = 2;
pub const PROCESSOR_INFO_TYPE: u8 = 4;
pub const MEMORY_DEVICE_TYPE: u8 = 17;

#[derive(Debug, Clone, Default)]
pub struct BiosInfo {
    pub vendor: heapless::String<64>,
    pub version: heapless::String<64>,
    pub release_date: heapless::String<64>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub manufacturer: heapless::String<64>,
    pub product_name: heapless::String<64>,
    pub serial: heapless::String<64>,
    pub uuid: [u8; 16],
}

#[derive(Debug, Clone, Default)]
pub struct BaseboardInfo {
    pub manufacturer: heapless::String<64>,
    pub product: heapless::String<64>,
    pub serial: heapless::String<64>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessorInfo {
    pub socket: heapless::String<64>,
    pub manufacturer: heapless::String<64>,
    pub signature: u32,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryDevice {
    pub size_kb: u32,
    pub manufacturer: heapless::String<64>,
    pub serial: heapless::String<64>,
    pub part_number: heapless::String<64>,
}

#[derive(Debug, Clone, Default)]
pub struct SmbiosData {
    pub bios: BiosInfo,
    pub system: SystemInfo,
    pub baseboard: BaseboardInfo,
    pub processor: ProcessorInfo,
    pub memory_devices: heapless::Vec<MemoryDevice, 8>,
}

/// Raw SMBIOS structure header
#[derive(Debug, Clone, Copy)]
pub struct StructureHeader {
    pub typ: u8,
    pub length: u8,
}
