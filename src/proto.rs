#[derive(PartialEq, Debug)]

/// Field in an Ethernet Frame used to determine which protocol is in the payload
/// https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml#ieee-802-numbers-1
/// We really only care about IP packets
pub(crate) enum EtherType {
    IPv4 = 0x0800,
    IPv6 = 0x86DD,
    Undefined,
}

impl Into<EtherType> for u16 {
    fn into(self) -> EtherType {
        match self {
            0x0800 => EtherType::IPv4,
            0x86DD => EtherType::IPv6,
            _ => EtherType::Undefined,
        }
    }
}

/// https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
/// These are just some of the Protocols that we may care about and is not meant to be exhaustive
#[derive(PartialEq, Debug)]
pub(crate) enum ProtocolType {
    ICMP = 0x001,
    TCP = 0x006,
    UDP = 0x011,
    Undefined,
}

impl Into<ProtocolType> for u8 {
    fn into(self) -> ProtocolType {
        match self {
            0x001 => ProtocolType::ICMP,
            0x006 => ProtocolType::TCP,
            0x011 => ProtocolType::UDP,
            _ => ProtocolType::Undefined,
        }
    }
}
