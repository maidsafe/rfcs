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
/// A socket address obtained through some mapping technique.
pub struct MappedSocketAddr {
    /// The mapped address
    pub addr: SocketAddrV4,

    /// Indicated that hole punching needs to be used for an external client to connect to this
    /// address. `nat_restricted` will not be set if this is a fully mapped address such as the
    /// external address of a full-cone NAT or one obtained through UPnP.
    pub nat_restricted: bool,
}

/// The result of mapping a UDP socket.
pub struct MapUdpSocketResult {
    /// The result of attempting UPnP mapping.
    pub upnp_result: Result<(), ...>,

    /// A vector of all addresses we were able to obtain for this socket.
    pub endpoints: Vec<MappedSocketAddr>,
}

/// The address of a server that can be used to obtain an external address.
pub enum HolePunchServerAddr {
    /// A server which speaks the simple hole punching protocol.
    Simple(SocketAddrV4),
}

/// Used to map a udp socket.
struct UdpSocketMapper<'a> {
    socket: UdpSocket,
    hole_punch_servers: &'a MappingContext,
}

impl<'a> UdpSocketMapper<'a> {
    /// Create a `UdpSocketMapper` which maps `socket` using `hole_punch_servers`.
    /// `hole_punch_servers` can be `&[]` if we only want to use UPnP. The
    /// `UdpSocketMapperController` can be dropped to asynchronously cancel the `map` and
    /// `map_timeout` methods.
    fn new(socket: UdpSocket, hole_punch_servers: &'a MappingContext) -> (UdpSocketMapper<'a>, UdpSocketMapperController);

    /// the mapping. Returns `None` if the operation was canceled by dropping the
    /// `UdpSocketMapperController`.
    fn map(self) -> Option<(UdpSocket, MapUdpSocketResult)>
    
    /// Perform the mapping. Returns `None` if the operation was canceled by dropping the
    /// `UdpSocketMapperController` or if the timeout was reached.
    fn map_timeout(self, timeout: Duration) -> Option<(UdpSocket, MapUdpSocketResult)>
}

/// Drop this object to unblock the corresponding `UdpSocketMapper` `map` or `map_timeout` method.
struct UdpSocketMapperController { ... }

/// The result of attempting udp hole punching.
struct UdpHolePunchResult {
    result: io::Result<SocketAddrV4>
}

/// Used to punch a hole through a NAT.
struct UdpHolePuncher<'a, 'b> {
    socket: UdpSocket
    target: &'a [SocketAddrV4],
    secret: &'b [u8],
}

impl<'a, 'b> UdpHolePuncher<'a, 'b> {
    fn new(socket: UdpSocket, target: &'a [SocketAddrV4], secret: &'b [u8])
        -> (UdpHolePuncher<'a, 'b>, UdpHolePuncherController)
    fn punch_hole(self) -> Option<(UdpSocket, UdpHolePunchResult)>
    fn punch_hole_timeout(self) -> Option<(UdpSocket, UdpHolePunchResult)>
}

/// RAII type for a hole punch server which speaks the simple hole punching protocol.
struct SimpleUdpHolePunchServer<'a> {
    mapping_context: &'a MappingContext,
}

impl<'a> SimpleUdpHolePunchServer<'a> {
    /// Create a new server. This will spawn a background thread which will serve requests until
    /// the server is dropped.
    fn new(mapping_context: &'a MappingContext) -> SimpleUdpHolePunchServer<'a>;

    /// Obtain the addresses that this server can be contacted on. The `Addresses` object can be
    /// used to get the addresses. The `AddressesController` object can dropped to abort the
    /// `Addresses` `get` methods.
    fn addresses(&self) -> (Addresses, AddressesController)
}

impl Addresses {
    /// Get the addresses. Will block until the addresses are obtained or the operation is
    /// cancelled by dropping the `AddressesController`.
    fn get(self) -> Option<MapUdpSocketResult>

    /// Get the addresses. Will block until the addresses are obtained or the operation times out
    /// or is cancelled by dropping the `AddressesController`.
    fn get_timeout(self, timeout: Duration) -> Option<MapUdpSocketResult>
}

/// Drop this object to cancel the corresponding `Addresses::get` call.
struct AddressesController;

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

```

# Drawbacks

I can't see a reason not to do this.

# Alternatives

* Not do this.
* Use a simpler API with non-cancellable method calls.

# Unresolved questions

What other existing techniques are there for NAT traversal? Is this API
actually forward-compatible if we wish to add these techniques in the future?
