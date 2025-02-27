use nom::{combinator::all_consuming, error::VerboseError};

use crate::devices::standard::{
    packets::inbound::{FirmwareVersionUpdatePacket, InboundPacket},
    state::DeviceState,
};

pub fn firmware_version_update_handler(input: &[u8], state: DeviceState) -> DeviceState {
    let result: Result<_, nom::Err<VerboseError<&[u8]>>> =
        all_consuming(FirmwareVersionUpdatePacket::take)(input);
    let packet = match result {
        Ok((_, packet)) => packet,
        Err(err) => {
            tracing::error!("failed to parse packet: {err:?}");
            return state;
        }
    };
    DeviceState {
        firmware_version: Some(
            packet
                .left_firmware_version
                .max(packet.right_firmware_version),
        ),
        serial_number: Some(packet.serial_number),
        ..state.clone()
    }
}
