- Feature Name: crust_library
- Type: enhancement
- Related components crust
- Start Date: 13-12-2015
- RFC PR: 
- Issue number: 

# Summary

The purpose of this document is to outline the proposed API of a refactored
Crust.

# Motivation

See the main Crust RFC for an explanation.

# Detailed design

The user starts their crust session by creating a `Service` object.

```rust
impl Service {
    /// Create a new service from the default configuration file.
    pub fn new() -> Service;
    /// Create a new service from the given config file.
    pub fn new_from_config(cfg: Config) -> Service;
}
```

Crust performs many separate functions, as such `Service` is a transparent
object, that can be disassembled into components to be used independently.

```rust
struct Service {
    /// Used to accept incoming connections.
    pub acceptor: ServiceListener,
    /// Used to control the state of the beacon.
    pub beacon_state: ServiceBeaconState,
    /// Used to receive endpoints broadcasted by peers on the local network.
    pub beacon_receiver: ServiceBeaconReceiver,
    /// Used to control the service.
    pub controller: ServiceController,
}
```

The user moves `acceptor` to where they want to wait for incoming connections
and `beacon_receiver` to where they want to wait for incoming beacon messages.

`controller` is used to perform almost all the other functionality of crust. it
implements `Sync` and it's methods borrow `self` immutably allowing it to be
borrowed and used throughout the rest of the program. `controller` can be used to
 * Add/remove/list endpoints on the service's internal set of listening endpoints.
 * Tell the service about external hole-punching servers.
 * Get the local hole-punching server's addresses.
 * Perform connections and rendezvous connections.
 * Read the cache of recorded endpoints.

```rust
/// Drop this type to shutdown the service.
impl ServiceController {
    /// Accepting.
    /// Crust maintains a set of endpoints that it listens for connections on.
    /// These functions can be used to add/remove/inspect this set.
    pub fn add_listener(&self, addr: ListenEndpoint);
    pub fn remove_listener(&self, addr: ListenEndpoint);
    pub fn accepting_endpoints<'c>(&'c self) -> AcceptingEndpoints<'c>

    /// Hole punching
    /// Crust maintains a list of external hole-punching servers that it can
    /// use to perform rendezvous connections. It also acts as a hole-punching
    /// server for other peers.
    /// Some of these types are described in the nat-traversal doc.
    pub fn mapping_context(&self) -> &MappingContext;
    pub fn add_hole_puncher_server(&self, server: HolePunchServerAddr);
    pub fn hole_punch_addresses(&self)
        -> Vec<HolePunchServerAddr>

    /// Get a mapped udp socket using the `Service's internal `MappingContext`
    pub fn mapped_udp_socket<'c>(&'c self)
        -> MappedUdpSocket;
    /// Connect a `MappedUdpSocket`.
    pub fn utp_rendezvous_connect(&self, mapped_socket: MappedUdpSocket,
                                         their_info: UdpRendezvousInfo)
        -> Stream

    /// Connecting
    pub fn connect(&self, endpoint: Endpoint)
        -> Stream

    /// Cacheing/Bootstrapping
    /// Crust records the endpoint of every peer it successfully connects to or
    /// accepts a connection from so that the user can use them if they lose
    /// connection to the network.
    pub fn cache_endpoint(&self, endpoint: Endpoint)
    pub fn iter_endpoint_cache<'c>(&'c self) -> EndpointCacheIterator<'c>
}
```

The service's beacon state (enabled or disabled) is controlled via a separate
object. This is just to take advantage of the fact that - being a simple
boolean - it's state can be represented at the type level. The user can enforce
compile-time guarantees about the state of the beacon during particular parts
of their code using the `ServiceBeaconState<Enabled>` and
`ServiceBeaconState<Disabled>`.

The full API (sans error handling) is given below.

```rust
impl Service {
    /// Create a new service from the default configuration file.
    pub fn new() -> Service;
    /// Create a new service from the given config file.
    pub fn new_from_config(cfg: Config) -> Service;
}

/// Rather than being a single, opaque type the `Service` is a struct with public fields, allowing
/// it to be disassembled into components which can be used independently.
struct Service {
    /// Used to accept incoming connections.
    pub acceptor: ServiceListener,
    /// Used to control the state of the beacon.
    pub beacon_state: ServiceBeaconState,
    /// Used to receive endpoints broadcasted by peers on the local network.
    pub beacon_receiver: ServiceBeaconReceiver,
    /// Used to control the service.
    pub controller: ServiceController,
}

impl ServiceListener {
    pub fn accept(&mut self) -> Stream;
    pub fn incoming(&mut self) -> Incoming;
}

impl Iterator for Incoming {
    type Item = Stream;
}

/// States that a service beacon can be in. Used to enforce state at
/// compile-time.
enum Dynamic {}
enum Enabled {}
enum Disabled {}

struct ServiceBeaconState<State = Dynamic> { .. }

impl<State> ServiceBeaconState<State> {
    /// Test whether the beacon is enabled.
    fn is_enabled(&self) -> bool,
    /// Enable to beacon.
    fn enable(self) -> ServiceBeaconState<Enabled>,
    /// Disable to beacon.
    fn disable(self) -> ServiceBeaconState<Disabled>,
}

impl ServiceBeaconState<Dynamic> {
    /// Set the period between transmissions.
    fn set_period(&mut self, period: Duration)
    /// Set whether the beacon is enabled.
    fn set_enabled(&mut self, enabled: bool)
}

/// This is because some code may want a static guarantee that the beacon is held in a particular
/// state. Usually, enforcing this sort of thing at the type-level would require typestate or
/// dependent types. But as the state is only a boolean we can acheive this in rust by treating the
/// states as two different types.
impl ServiceBeaconState<Enabled> {
    fn set_period(&mut self, period: Duration);
}

/// If the user wishes to erase which state the beacon is in they can easily cast these types back
/// to a `ServiceBeaconState<Dynamic>`
impl From<ServiceBeaconState<Enabled>> for ServiceBeaconState<Dynamic>
impl From<ServiceBeaconState<Disabled>> for ServiceBeaconState<Dynamic>

impl ServiceBeaconReceiver {
    /// Block until we receive an `Endpoint` from a local peer.
    pub fn next(&mut self) -> Endpoint;
    /// Iterate over endpoints as we receive them.
    pub fn endpoints(&mut self) -> BeaconEndpoints;
}

impl Iterator for BeaconEndpoints {
    type Item = Endpoint;
}

/// Drop this type to shutdown the service.
impl ServiceController {
    /// Accepting
    pub fn add_listener(&self, addr: ListenEndpoint);
    pub fn remove_listener(&self, addr: ListenEndpoint);
    pub fn accepting_endpoints<'c>(&'c self) -> AcceptingEndpoints<'c>

    /// Hole punching
    pub fn mapping_context(&self) -> &MappingContext;
    pub fn add_hole_puncher_server(&self, server: HolePunchServerAddr);
    pub fn hole_punch_addresses(&self)
        -> Vec<HolePunchServerAddr>

    /// Get a mapped udp socket using the `Service's internal `MappingContext`
    pub fn mapped_udp_socket(&self)
        -> MappedUdpSocket;
    /// Connect a `MappedUdpSocket`.
    pub fn utp_rendezvous_connect(&self, mapped_socket: MappedUdpSocket,
                                         their_info: UdpRendezvousInfo)
        -> Stream

    /// Connecting
    pub fn connect(&self, endpoint: Endpoint)
        -> Stream

    /// Cacheing/Bootstrapping
    /// Add an endpoint to the cache.
    pub fn cache_endpoint(&self, endpoint: Endpoint)
    /// Iterate over the endpoints in the cache.
    /// # Example
    /// ```rust
    /// // Attempt to connect to all endpoints in the cache.
    /// for cached_endpoint in controller.iter_endpoint_cache() {
    ///     let stream = try!(cached_endpoint.connect());
    /// }
    /// ```
    pub fn iter_endpoint_cache<'c>(&'c self) -> EndpointCacheIterator<'c>
}

impl<'c> Iterator for AcceptingEndpoints<'c> {
    type Item = AcceptingEndpoint<'c>;
}

/// Represents an endpoint that Crust is currently listening on. An endpoint
/// consists of the local endpoint address and the external addresses
/// connectable by other peers.
impl<'c> AcceptingEndpoint<'c> {
    /// Returns immediately with the local address of this endpoint.
    fn local_endpoint(&self) -> &ListenEndpoint;
    /// Return an iterator over the known mapped addresses of this endpoint.
    /// Does not perform any port-mapping.
    fn known_endpoints<'e>(&'e self) -> KnownEndpoints<'e, 'c>;
    /// Maps this endpoint and returns an iterator that can be used to get
    /// the mapped addresses.
    fn mapped_endpoints<'e>(&'e self)
        -> MappedEndpoints<'e, 'c>,
}

struct MappedEndpoint {
    pub endpoint: Endpoint,
    pub nat_restricted: bool,
}

impl<'e, 'c> Iterator for KnownEndpoints<'e, 'c> {
    type Item = MappedEndpoint;
}

impl<'e, 'c> Iterator for MappedEndpoints<'e, 'c> {
    type Item = MappedEndpoint;
}

/// Iterate over Crust's cached endpoints.
impl<'c> EndpointCacheIterator<'c> {
    type Item = CachedEndpoint<'c>;
}

/// An endpoint in Crust's cache.
impl<'c> CachedEndpoint<'c> {
    /// Remove this endpoint from the cache.
    pub fn remove(self);

    /// Get the endpoint.
    pub fn endpoint(&self) -> Endpoint;

    /// Calls `service.connect(self.endpoint())` then calls `self.remove()` if the connect fails.
    pub fn connect(self) -> Stream
}
```

# Drawbacks

This is a major overhaul of the current crust API and will take time to implement.

# Alternatives

Not do this.

# Unresolved questions

None.

