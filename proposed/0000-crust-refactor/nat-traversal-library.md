- Feature Name: nat_traversal
- Type: enhancement
- Related components: crust
- Start Date: 11-11-2015
- RFC PR:
- Issue number:

# Summary

Create a separate library for NAT traversal techniques.

# Motivation

The MaidSAFE network needs to be able to operate through NATs. Currently NAT
traversal is implemented in Crust, however these techniques may be useful
elsewhere and to other Rust developers. We can achieve better separation of
concerns by implementing a generic NAT-traversal library as a separate crate.

# Detailed design

The actual techniques used in this library are described in [RFC-008 UDP Hole Punching](https://github.com/maidsafe/rfcs/tree/master/active/0008-UDP-hole-punching).

This RFC proposes that the existing implementation be factored out into a
separate library. A new API is suggested which is intended to be
forward-compatible with future versions of this library which may implement
standard NAT traversal techniques such as STUN and ICE.

Additionally, this RFC proposes that both UPnP and then external-server-based
port mapping techniques be employed whenever the library attempts to map a
socket rather than one or the other technique depending on the situation.

A draft of the public-facing API of the library is proposed below. Most
error/result types are omitted from the proposal both for clarity and because
it's not always obvious where these types are necessary until it comes to
implementing it. As this is a blocking-based API this should not be an issue -
errors will simply be returned where they occur.

```rust
/// The address of a server that can be used to obtain an external address.
pub enum HolePunchServerAddr {
    /// A server which speaks the simple hole punching protocol (ie. our MaidSafe protocol). This
    /// should probably be deprecated and replaced with a proper STUN implementation.
    Simple(SocketAddrV4),
    /// An Internet Gateway Device that can be used for UPnP port mapping.
    IgdGateway(igd::Gateway)
}

/// You need to create a `MappingContext` before doing any socket mapping. This `MappingContext`
/// should ideally be kept throughout the lifetime of the program. Internally it caches a
/// addresses of UPnP servers and hole punching servers.
struct MappingContext {
    servers: RwLock<Vec<HolePunchServerAddr>>,
}

impl MappingContext {
    /// Create a new mapping context. This will block breifly while it searches the network for
    /// UPnP servers.
    fn new() -> MappingContext,

    /// Inform the context about external hole punching servers.
    fn add_servers<S>(&self, servers: S)
        where S: IntoIterator<Item=HolePunchServerAddr>
}

/// A socket address obtained through some mapping technique.
pub struct MappedSocketAddr {
    /// The mapped address. Mapped addresses include all the addresses that a peer
    /// may be able to connect to the socket on. This includes the socket's local address for the
    /// sake of peers that are on the same local network. A Vec of MappedSocketAddr may also
    /// include several different addresses obtained from external servers in the case that
    /// we are behind more that one NAT.
    pub addr: SocketAddrV4,

    /// Indicated that hole punching needs to be used for an external client to connect to this
    /// address. `nat_restricted` will not be set if this is a fully mapped address such as the
    /// external address of a full-cone NAT or one obtained through UPnP.
    pub nat_restricted: bool,
}

/// A bound udp socket for which we know our external endpoints.
struct MappedUdpSocket {
    /// The socket.
    pub socket: UdpSocket,
    /// The known endpoints of this socket. This includes all known endpoints of the socket
    /// including local addresses.
    pub endpoints: Vec<MappedSocketAddr>
}

/// Info exchanged by both parties before performing a rendezvous connection.
struct PubRendezvousInfo {
    /// A vector of all the mapped addresses that the peer can try connecting to.
    endpoints: Vec<MappedSocketAddr>,
    /// Used to identify the peer.
    secret: [u8; 4],
}

/// The local half of a `PubRendezvousInfo`.
struct PrivRendezvousInfo {
    secret: [u8; 4],
}

/// Create a `(PrivRendezvousInfo, PubRendezvousInfo)` pair from a list of mapped socket addresses.
pub fn gen_rendezvous_info(endpoints: Vec<MappedSocketAddr>)
    -> (PrivRendezvousInfo, PubRendezvousInfo)

impl MappedUdpSocket {
    /// Map an existing `UdpSocket`. The mapped addresses include all the addresses that a peer
    /// may be able to connect to the socket on. This includes the socket's local address for the
    /// sake of peers that are on the same local network. It may also include 
    pub fn map(socket: UdpSocket, mc: &MappingContext)
        -> MappedUdpSocket

    /// Create a new `MappedUdpSocket`
    pub fn new(mc: &MappingContext)
        -> MappedUdpSocket
}

/// A udp socket that has been hole punched.
struct PunchedUdpSocket {
    pub socket: UdpSocket,
    pub peer_addr: SocketAddr,
}

impl PunchedUdpSocket {
    /// Punch a udp socket using a mapped socket and the peer's rendezvous info.
    pub fn punch_hole(socket: UdpSocket,
                      our_priv_rendezvous_info: PrivRendezvousInfo,
                      their_pub_rendezvous_info: PubRendezvousInfo)
        -> PunchedUdpSocket
}

/// A tcp socket for which we know our external endpoints.
struct MappedTcpSocket {
    /// A bound, but neither listening or connected tcp socket. The socket is bound to be reuseable
    /// (ie. SO_REUSEADDR is set as is SO_REUSEPORT on unix).
    pub socket: net2::TcpBuilder,
    /// The known endpoints of this socket. This includes all known endpoints of the socket
    /// including local addresses.
    pub endpoints: Vec<MappedSocketAddr>,
}

impl MappedTcpSocket {
    /// Map an existing tcp socket. The socket must not bound or connected. This function will set
    /// the options to make the socket address reuseable before binding it.
    pub fn map(socket: net2::TcpBuilder, mc: &MappingContext)
        -> MappedTcpSocket;

    /// Create a new `MappedTcpSocket`
    pub fn new(mc: &MappingContext)
        -> MappedTcpSocket;
}

/// Perform a tcp rendezvous connect. `socket` should have been obtained from a `MappedTcpSocket`.
pub fn tcp_punch_hole(socket: net2::TcpBuilder,
                      our_priv_rendezvous_info: PrivRendezvousInfo,
                      their_pub_rendezvous_info: PubRendezvousInfo)
    -> TcpStream;

/// RAII type for a hole punch server which speaks the simple hole punching protocol.
struct SimpleUdpHolePunchServer<'a> {
    mapping_context: &'a MappingContext,
}

impl<'a> SimpleUdpHolePunchServer<'a> {
    /// Create a new server. This will spawn a background thread which will serve requests until
    /// the server is dropped.
    pub fn new(mapping_context: &'a MappingContext)
        -> SimpleUdpHolePunchServer<'a>;

    /// Get the external addresses of this server to be shared with peers.
    pub fn addresses(&self)
        -> Vec<MappedSocketAddr>
}
```

The following code demonstrates how one could use this API to perform a udp rendezvous connection.

```rust
// First, one needs a `MappingContext` to do the port mapping with.
let mut mc = MappingContext::new();

// Optionally, they can inform the context about any external port-mapping
// servers they know about.
mc.add_servers([some_well_known_server]);

// Now they create a mapped udp socket to use for the connection.
let mapped_socket = MappedUdpSocket::new(&mc);

// A mapped socket consists of a socket and a list of endpoints.
let MappedUdpSocket { socket, endpoints } = mapped_socket;

// Next, they create a `PrivRendezvousInfo`, `PubRendezvousInfo` pair from the socket's endpoints.
let (our_priv_rendezous_info, our_pub_rendezvous_info) = gen_rendezvous_info(endpoints);

// Now, the peers share their public rendezvous info out-of-band somehow.
let their_pub_rendezvous_info = ???

// Now they do the hole-punching.
let punched_udp_socket = PunchedUdpSocket::punch_hole(socket,
                                                      our_priv_rendezvous_info,
                                                      their_pub_rendezvous_info)

// Extract the socket and peer address
let PunchedUdpSocket { socket, peer_addr } = punched_udp_socket;

// Congratualtions! If everything succeeded then `socket` is a `UdpSocket` that
// can be used to talk to `peer_addr` through NATs and firewalls.
```

# Drawbacks

I can't see a reason not to do this.

# Alternatives

Not do this.

# Unresolved questions

What other existing techniques are there for NAT traversal? Is this API
actually forward-compatible if we wish to add these techniques in the future?

