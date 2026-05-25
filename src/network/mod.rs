pub mod display;
pub mod types;

use uefi::prelude::*;
use uefi::proto::network::snp::SimpleNetwork;
use uefi::table::boot::ScopedProtocol;

use self::types::*;

/// Collect network interface information via SimpleNetwork protocol
pub fn collect(boot_services: &BootServices) -> Result<NetworkInfo, &'static str> {
    // Try to locate SimpleNetwork protocol
    let handle = boot_services
        .get_handle_for_protocol::<SimpleNetwork>()
        .map_err(|_| "No SimpleNetwork protocol handle found")?;

    let snp: ScopedProtocol<SimpleNetwork> = boot_services
        .open_protocol_exclusive(handle)
        .map_err(|_| "Failed to open SimpleNetwork protocol")?;

    let mut info = NetworkInfo::default();

    // Get current mode
    let mode = snp.mode();

    // Map state
    info.state = match mode.state {
        uefi::proto::network::snp::NetworkState::STOPPED => NetworkState::Stopped,
        uefi::proto::network::snp::NetworkState::STARTED => NetworkState::Started,
        uefi::proto::network::snp::NetworkState::INITIALIZED => NetworkState::Initialized,
        _ => NetworkState::Unknown,
    };

    // Get MAC address from current_address field
    // MAC address is the first 6 bytes of the 32-byte MacAddress struct
    let mac = mode.current_address.0;
    info.mac_address.copy_from_slice(&mac[..6]);

    info.media_header_size = mode.media_header_size;
    info.max_packet_size = mode.max_packet_size;
    info.present = true;

    Ok(info)
}
