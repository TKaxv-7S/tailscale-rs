use core::error::Error;

use crate::{BatchRecvIter, BatchSendIter, MapPeerKey, PeerLookup};

/// The unique id of an underlay transport.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnderlayTransportId(pub u32);

impl From<u32> for UnderlayTransportId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<UnderlayTransportId> for u32 {
    fn from(value: UnderlayTransportId) -> Self {
        value.0
    }
}

/// An abstract transport that can carry packets to configurable destinations.
pub trait UnderlayTransport {
    /// The type of key this transport uses to identify peers.
    ///
    /// The runtime generally wants to use [`PeerId`][crate::PeerId] here, but transports
    /// will almost always want to use a different key type for communication (however the
    /// peer is known on the wire).
    ///
    /// To decouple, transport implementations can use their wire type here, while the
    /// runtime wraps the implementation with [`UnderlayTransportExt::with_key_lookup`]
    /// to provide functionality to convert the wire type to and from
    /// [`PeerId`][crate::PeerId].
    type PeerKey: Send + Sync + 'static;

    /// The error type that this transport may produce.
    type Error: Error + Send + Sync + 'static;

    /// Send packets through the transport.
    ///
    /// The return type should be interpreted as meaning essentially
    /// `HashMap<PeerId, Vec<PacketMut>>`. It is set up this way to enable the caller
    /// to use iterators to transform a collection of a slightly different shape, or e.g.
    /// look up `PeerId`s on-the-fly, without having to `.collect()` into an
    /// intermediary collection.
    fn send(
        &self,
        packet_batch: impl BatchSendIter<Self::PeerKey>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Receive packets from the transport.
    ///
    /// The return type should be interpreted as meaning essentially
    /// `HashMap<PeerId, Vec<PacketMut>>`, but allows for the implementation to
    /// use iterators to map a collection of a slightly different shape, or e.g. look up
    /// `PeerId`s on-the-fly, without having to `.collect()` into an intermediary
    /// collection.
    fn recv(
        &self,
    ) -> impl Future<Output = impl BatchRecvIter<Self::PeerKey, Error = Self::Error>> + Send;
}

/// Extension methods on [`UnderlayTransport`].
pub trait UnderlayTransportExt: UnderlayTransport {
    /// Map the keys used by this transport with the given [`PeerLookup`].
    fn with_key_lookup<DstKey, Lookup>(self, lookup: Lookup) -> MapPeerKey<Self, Lookup, DstKey>
    where
        Self: Sized + Send + Sync,
        Lookup: PeerLookup<Self::PeerKey, DstKey> + PeerLookup<DstKey, Self::PeerKey> + Send + Sync,
        DstKey: Send + Sync + 'static,
    {
        MapPeerKey::new(self, lookup)
    }
}

impl<T> UnderlayTransportExt for T where T: UnderlayTransport {}
