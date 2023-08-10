use super::OutboundPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RequestFirmwareVersionPacket {}

impl RequestFirmwareVersionPacket {
    pub fn new() -> Self {
        Self {}
    }
}

impl OutboundPacket for RequestFirmwareVersionPacket {
    fn command(&self) -> [u8; 7] {
        [0x08, 0xee, 0x00, 0x00, 0x00, 0x01, 0x05]
    }

    fn body(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::outbound::{OutboundPacketBytes, RequestFirmwareVersionPacket};

    #[test]
    fn it_matches_a_manually_crafted_packet() {
        let expected: &[u8] = &[0x08, 0xee, 0x00, 0x00, 0x00, 0x01, 0x05, 0x0a, 0x00, 0x06];
        assert_eq!(expected, RequestFirmwareVersionPacket::new().bytes())
    }
}
