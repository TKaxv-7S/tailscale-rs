use bytes::Buf;

/// Heuristic identification of a packet.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PacketIdent {
    /// The type of the packet.
    pub ty: PacketType,
    /// Whether the packet is encapsulated.
    pub encapsulation: Encapsulation,
}

/// Heuristically-determined packet type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PacketType {
    /// This is a Tailscale disco packet.
    Disco,
    /// This is a WireGuard packet.
    #[default]
    Wireguard,
    /// This is a STUN binding packet.
    StunBinding,
    /// The type of this packet is unknown.
    Unknown,
}

/// The encapsulation format of a packet.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Encapsulation {
    /// Packet is not encapsulated.
    #[default]
    None,
    /// Packet is encapsulated in a valid Geneve header.
    Geneve,
}

impl PacketIdent {
    /// The length of a geneve header.
    pub const GENEVE_HEADER_LEN: usize = 8;

    /// Geneve ethertype field indicating this is a Tailscale disco message.
    pub const GENEVE_PROTO_DISCO: u16 = 0x7a11;
    /// Geneve ethertype field indicating this is a WireGuard message.
    pub const GENEVE_PROTO_WIREGUARD: u16 = 0x7a12;

    /// A mask on the first two bytes of the Geneve header that we always transmit as zero.
    pub const GENEVE_MASK: u16 = 0xc03f;

    /// Determine the type and encapsulation of `pkt`.
    ///
    /// This differs from the Go (`magicsock.go:packetLooksLike`) in that
    /// `PacketType::Unknown` is returned if the data makes no positive indication of what
    /// it is. For compatibility with the Go, these cases should be treated as
    /// unencapsulated WireGuard.
    pub fn identify(pkt: &[u8]) -> PacketIdent {
        if let Some(ty) = Self::parse_geneve(pkt) {
            return Self {
                encapsulation: Encapsulation::Geneve,
                ty,
            };
        }

        Self {
            encapsulation: Encapsulation::None,
            ty: if disco::is_disco_message(pkt) {
                PacketType::Disco
            } else {
                PacketType::Unknown
            },
        }
    }

    /// Attempt to parse `pkt` as a Geneve-encapsulated packet.
    pub fn parse_geneve(mut pkt: &[u8]) -> Option<PacketType> {
        if pkt.len() < Self::GENEVE_HEADER_LEN || pkt[7] != 0 {
            return None;
        }

        let version_reserved = pkt.get_u16();
        if version_reserved & Self::GENEVE_MASK != 0 {
            return None;
        }

        let ethertype = pkt.get_u16();
        let _geneve_rest = pkt.split_off(..4);

        match ethertype {
            Self::GENEVE_PROTO_DISCO if disco::is_disco_message(pkt) => Some(PacketType::Disco),
            Self::GENEVE_PROTO_DISCO => Some(PacketType::Unknown),
            Self::GENEVE_PROTO_WIREGUARD => Some(PacketType::Wireguard),
            _ => None,
        }
    }

    /// Return the offset to the
    pub const fn payload_offset(&self, unknown_as_wireguard: bool) -> usize {
        if unknown_as_wireguard && matches!(self.ty, PacketType::Unknown) {
            return 0;
        }

        match self.encapsulation {
            Encapsulation::None => 0,
            Encapsulation::Geneve => Self::GENEVE_HEADER_LEN,
        }
    }
}
