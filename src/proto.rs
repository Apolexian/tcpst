#[derive(PartialEq, Debug)]

/// Field in an Ethernet Frame used to determine which protocol is in the payload 
/// https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml#ieee-802-numbers-1
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
