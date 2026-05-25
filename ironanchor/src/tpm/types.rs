/// TPM2 response codes
pub const TPM2_RC_SUCCESS: u32 = 0x000;

/// TPM info collected from capabilities
#[derive(Debug, Clone, Default)]
pub struct TpmInfo {
    pub present: bool,
    pub manufacturer: u32,
    pub firmware_version: u64,
    pub active_pcr_banks: u32,
    pub pcr_count: u32,
}
