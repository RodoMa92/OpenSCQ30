use crate::packets::inbound::InboundPacket;

use super::DeviceState;

pub trait DeviceStateTransformer {
    fn transform(&self, state: &DeviceState) -> DeviceState;
}

pub fn inbound_packet_to_state_transformer(
    value: InboundPacket,
) -> Option<Box<dyn DeviceStateTransformer + Send + Sync>> {
    match value {
        InboundPacket::StateUpdate(packet) => Some(Box::new(packet)),
        InboundPacket::SoundModeUpdate(packet) => Some(Box::new(packet)),
        InboundPacket::SetSoundModeOk(_packet) => None,
        InboundPacket::SetEqualizerOk(_packet) => None,
    }
}
