pub trait OutboundPacket {
    fn bytes(&self) -> Vec<i16>;
}
