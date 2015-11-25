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
 * Add/remove/list listening endpoints on the service's internal `Acceptor`.
 * Tell the service about external hole-punching servers.
 * Get the local hole-punching server's addresses.
 * Perform connections and rendezvous connections.
 * Read the cache of recorded endpoints.

```rust
/// Drop this type to shutdown the service.
impl ServiceController {
    /// Accepting
    pub fn add_listener(&self, addr: ListenEndpoint);
    pub fn remove_listener(&self, addr: ListenEndpoint);
    pub fn accepting_endpoints<'c>(&'c self) -> AcceptingEndpoints<'c>

    /// Hole punching
    pub fn mapping_context(&self) -> &MappingContext;
    pub fn add_hole_puncher_server(&self, server: HolePunchServerAddr);
    pub fn hole_punch_addresses(&self, bop_handle: &BopHandle)
        -> BopResult<Vec<HolePunchServerAddr>>

    /// Get a mapped udp socket using the `Service's internal `MappingContext`
    pub fn mapped_udp_socket<'c>(&'c self, bop_handle: &BopHandle)
        -> BopResult<MappedUdpSocket>;
    /// Connect a `MappedUdpSocket`.
    pub fn utp_rendezvous_connect(&self, bop_handle: &BopHandle,
                                         mapped_socket: MappedUdpSocket,
                                         their_info: UdpRendezvousInfo)
        -> BopResult<Stream>

    /// Connecting
    pub fn connect(&self, bop_handle: &BopHandle, endpoint: Endpoint)
        -> BopResult<Stream>

    /// Cacheing/Bootstrapping
    pub fn cache_endpoint(&self, endpoint: Endpoint)
    pub fn iter_endpoint_cache<'c>(&'c self) -> EndpointCacheIterator<'c>
}
```

The service's beacon state (enabled or disabled) is controlled via a separate
object. This is just to take advantage of the fact that - being a simple
boolean - it's state can be represented at the type level. The user can enforce
compile-time guarantees about the state of the beacon during particular parts
of their code, or they can just stick the `ServiceBeaconState` in a `Mutex` and
use it the same way they use the `ServiceController`.

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
    pub fn accept(&mut self, bop_handle: &BopHandle) -> BopResult<Stream>;
    pub fn incoming(&mut self, bop_handle: &BopHandle) -> Incoming;
}

impl Iterator for Incoming {
    type Item = Stream;
}

impl ServiceBeaconState {
    /// Test whether the beacon is enabled.
    fn is_enabled(&self) -> bool,
    /// Set the period between transmissions.
    fn set_period(&mut self, period: Duration)
    /// Enable to beacon.
    fn enable(self) -> ServiceBeaconStateEnabled,
    /// Disable to beacon.
    fn disable(self) -> ServiceBeaconStateDisabled,
}

/// This is because some code may want a static guarantee that the beacon is held in a particular
/// state. Usually, enforcing this sort of thing at the type-level would require typestate or
/// dependent types. But as the state is only a boolean we can acheive this in rust by treating the
/// states as two different types.
impl ServiceBeaconStateEnabled {
    fn set_period(&mut self, period: Duration);
    fn disable(self) -> ServiceBeaconStateDisabled;
}

impl ServiceBeaconStateDisabled {
    fn enable(self) -> ServiceBeaconStateEnabled;
}

/// If the user wishes to erase which state the beacon is in they can easily cast these types back
/// to a `ServiceBeaconState`
impl From<ServiceBeaconStateEnabled> for ServiceBeaconState;
impl From<ServiceBeaconStateDisabled> for ServiceBeaconState;

impl ServiceBeaconReceiver {
    pub fn next(&mut self, bop_handle: &BopHandle) -> BopResult<Endpoint>;
    pub fn endpoints(&mut self, bop_handle: &BopHandle) -> BeaconEndpoints;
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
    pub fn hole_punch_addresses(&self, bop_handle: &BopHandle)
        -> BopResult<Vec<HolePunchServerAddr>>

    /// Get a mapped udp socket using the `Service's internal `MappingContext`
    pub fn mapped_udp_socket(&self, bop_handle: &BopHandle)
        -> BopResult<MappedUdpSocket>;
    /// Connect a `MappedUdpSocket`.
    pub fn utp_rendezvous_connect(&self, bop_handle: &BopHandle,
                                         mapped_socket: MappedUdpSocket,
                                         their_info: UdpRendezvousInfo)
        -> BopResult<Stream>

    /// Connecting
    pub fn connect(&self, bop_handle: &BopHandle, endpoint: Endpoint)
        -> BopResult<Stream>

    /// Cacheing/Bootstrapping
    pub fn cache_endpoint(&self, endpoint: Endpoint)
    pub fn iter_endpoint_cache<'c>(&'c self) -> EndpointCacheIterator<'c>
}

impl<'c> Iterator for AcceptingEndpoints<'c> {
    type Item = AcceptingEndpoint<'c>;
}

impl<'c> AcceptingEndpoint<'c> {
    fn local_endpoint(&self) -> &ListenEndpoint;
    fn known_endpoints<'e>(&'e self) -> KnownEndpoints<'e, 'c>;
    fn mapped_endpoints<'e>(&'e self, bop_handle: &BopHandle)
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

impl<'c> EndpointCacheIterator<'c> {
    type Item = CachedEndpoint<'c>;
}

impl<'c> CachedEndpoint<'c> {
    /// Remove this endpoint from the cache.
    pub fn remove(self);

    /// Get the endpoint.
    pub fn endpoint(&self) -> Endpoint;

    /// Calls `service.connect(self.endpoint())` then calls `self.remove()` if the connect fails.
    pub fn connect(self, bop_handle: &BopHandle) -> BopResult<Stream>
}
```

# Drawbacks

This is a major overhaul of the current crust API and will take time to implement.

# Alternatives

Not do this.

# Unresolved questions

None.

