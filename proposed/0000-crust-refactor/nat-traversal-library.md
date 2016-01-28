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
separate library. A new API is suggested which provides cancellable blocking
calls and is intended to be forward-compatible with future versions of this
library which may implement standard NAT traversal techniques such as STUN and
ICE.

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
    /// A server which speaks the simple hole punching protocol.
    Simple(SocketAddrV4),
}

/// Maintains a list of connections to Internet Gateway Devices (if there are any) as well as a set
/// of addresses of hole punching servers.
struct MappingContext {
    gateway: Vec<Result<igd::Gateway, igd::SearchError>>,
    servers: RwLock<Vec<HolePunchServerAddr>>,
}

impl MappingContext {
    /// Create a new mapping context.
    fn new() -> MappingContext

    /// Inform the context about external hole punching servers.
    fn add_servers<S>(&self, servers: S)
        where S: IntoIterator<Item=HolePunchServerAddr>
}

/// A socket address obtained through some mapping technique.
pub struct MappedSocketAddr {
    /// The mapped address
    pub addr: SocketAddrV4,

    /// Indicated that hole punching needs to be used for an external client to connect to this
    /// address. `nat_restricted` will not be set if this is a fully mapped address such as the
    /// external address of a full-cone NAT or one obtained through UPnP.
    pub nat_restricted: bool,
}

/// A bound udp socket for which we know our external endpoints.
struct MappedUdpSocket {
    pub socket: UdpSocket,
    pub endpoints: Vec<MappedSocketAddr>
}

/// Info needed by both parties when performing a udp rendezvous connection.
struct UdpRendezvousInfo {
    /// A vector of all the mapped addresses that the peer can try connecting to.
    endpoints: Vec<MappedSocketAddr>,
    /// Used to identify the peer.
    secret: [u8; 4],
}

impl UdpRendezvousInfo {
    /// Create rendezvous info for being sent to the remote peer.
    pub fn from_endpoints(endpoints: Vec<MappedSocketAddr>);
}

impl MappedUdpSocket {
    /// Map an existing `UdpSocket`.
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
    pub fn punch_hole(socket: UdpSocket, their_rendezvous_info: UdpRendezvousInfo)
        -> PunchedUdpSocket
}

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

// Optionally, the can inform the context about any external port-mapping
// servers they know about.
mc.add_servers([some_well_known_server]);

// Now they create a mapped udp socket to use for the connection.
let mapped_socket = MappedUdpSocket::new(&mc);

// A mapped udp socket consists of a socket and a list of known external
// endpoints for the socket.
let MappedUdpSocket { socket, endpoints } = mapped_socket;

// Now they create a `UdpRendezvousInfo` packet that they can share with the
// peer they want to rendezvous connect with.
let our_rendezvous_info = UdpRendezvousInfo::from_endpoints(endpoints);

// Now, the peers share rendezvous info out-of-band somehow.
let their_rendezvous_info = ???

// Now they do the hole-punching.
let punched_udp_socket = PunchedUdpSocket::punch_hole(socket, their_rendezvous_info)

// Extract the socket and peer address
let PunchedUdpSocket { socket, peer_addr } = punched_udp_socket;

// Congratualtions! If everthing succeeded then `socket` is a `UdpSocket` that
// can be used to talk to `peer_adddr` through NATs and firewalls.
```

# Drawbacks

I can't see a reason not to do this.

# Alternatives

Not do this.

# Unresolved questions

What other existing techniques are there for NAT traversal? Is this API
actually forward-compatible if we wish to add these techniques in the future?

