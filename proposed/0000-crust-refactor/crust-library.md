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

TODO: explain this design

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
    pub acceptor: ServiceAcceptGo,
    /// Used to control the state of the beacon.
    pub beacon_state: ServiceBeaconState,
    /// Used to receive endpoints broadcasted by peers on the local network.
    pub beacon_receiver: ServiceNextBeaconEndpointGo,
    /// Used to control the service.
    pub controller: ServiceController,
}

impl Go for ServiceAcceptGo {
    type Output = (ServiceAcceptGo, Stream);
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

impl Go for ServiceNextBeaconEndpointGo {
    type Output = (ServiceNextBeaconEndpointGo, Endpoint)
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

    /// Get a mapped udp socket using the `Service's internal `MappingContext`
    pub fn mapped_udp_socket<'c>(&'c self)
        -> (MappedUdpSocketGo<'c>, MappedUdpSocketKill<'c>)
    /// Connect a `MappedUdpSocket`.
    pub fn utp_rendezvous_connect<'c>(&'c self, mapped_socket: MappedUdpSocket, their_info: UdpRendezvousInfo)
        -> (UtpRendezvousConnectGo<'c>, UtpRendezvousConnectKill<'c>)

    /// Connecting
    pub fn connect<'c>(&'c self, endpoint: Endpoint)
        -> (ConnectGo<'c>, ConnectKill<'c>)
    
    /// Cacheing/Bootstrapping
    pub fn cache_endpoint(&self, endpoint: Endpoint)
    pub fn iter_endpoint_cache<'c>(&'c self) -> EndpointCacheIterator<'c>
}

impl<'c> Iterator for AcceptingEndpoints<'c> {
    type Item = AcceptingEndpoint<'c>;
}

impl<'c> AcceptingEndpoint<'c> {
    fn listen_endpoint(&self) -> &ListenEndpoint;
    fn known_endpoints<'e>(&'e self) -> KnownEndpoints<'e, 'c>;
    fn mapped_endpoints<'e>(&'e self) -> (MappedEndpointsGo<'e, 'c>, MappedEndpointsKill<'e, 'c>),
}

struct MappedEndpoint {
    pub endpoint: Endpoint,
    pub nat_restricted: bool,
}

impl<'e, 'c> Iterator for KnownEndpoints<'e, 'c> {
    type Item = MappedEndpoint;
}

impl<'e, 'c> Go for MappedEndpointsGo<'e, 'c> {
    type Output = (MappedEndpointsGo<'e, 'c>, MappedEndpoint);
}

impl<'c> Go for MappedUdpSocketGo<'c> {
    type Output = (MappedUdpSocket<'c>, UdpRendezvousInfo);
}

impl<'c> Go for UtpRendezvousConnectGo<'c> {
    type Output = Stream;
}

impl<'c> Go for ConnectGo<'c> {
    type Ouput = Stream;
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
    pub fn connect(self) -> (ConnectGo<'c>, ConnectKill<'c>)
}
```

# Drawbacks

Why should we *not* do this?

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
