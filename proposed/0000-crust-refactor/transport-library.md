Feature Name: transport_library
- Type: enhancement
- Related components: crust
- Start Date: 11-11-2015
- RFC PR:
- Issue number:

# Summary

This RFC outlines the design of a transport abstraction library to sit beneath crust.

# Motivation

Crust is intended to be a connection management library for use in implementing
P2P networks. One of the features crust provides is the ability to abstract
over various transport protocols, allowing the user to send data over TCP, UtP
and in the future possibly other transport protocols.

As crust's transport layer is set to become more advanced with the porting of
crust to mio, this RFC proposes that it be spun off into a separate library
that can be used independently of crust, or of P2P networking in general.

The aim of this library is to provide an abstraction over various transport
protocols, such TCP and UtP, as well as providing the functionality to create
streams through network address translators. The library provides raw streams
which could be used for implementing an http server, bittorrent client, or
whatever.

# Detailed design

The proposed API for this crate outlined below.

```
/// A connection to a peer.
type Stream;

impl From<TcpStream> for Stream;
impl From<UtpStream> for Stream;

/// An endpoint address which can be connected to.
type Endpoint;

/// Items which can be converted to an Endpoint.
trait ToEndpoint {
    /// Errors returned by `to_endpoint`
    type Err;

    /// Convert to an endpoint.
    fn to_endpoint(self) -> Result<Endpoint, Self::Err>;
}

impl ToEndpoint for Endpoint { ... }
impl ToEndpoint for ListenEndpoint { ... }
impl ToEndpoint for &str { ... }

/// An endpoint address which can be listened on.
type ListenEndpoint;

/// Items which can be converted to a `ListenEndpoint`
trait ToListenEndpoint {
    /// Errors returned by `to_listen_endpoint`
    type Err;

    /// Convert to a `ListenEndpoint`
    fn to_listen_endpoint(self) -> Result<ListenEndpoint, Self::Err>
}

impl ToListenEndpoint for ListenEndpoint { ... }
impl ToListenEndpoint for &str { ... }

type StreamListener
type ListenerSet;
type ListenerSetController;

/// Listen on an endpoint for incoming connections.
impl StreamListener {
    pub fn bind<A: ToListenEndPoint>(addr: A) -> StreamListener;
    pub fn accept(&mut self, bop_handle: &BopHandle) -> BopResult<Stream>;
}

/// Listens on multiple listening endpoints and accepts connections from any of them.
impl ListenerSet {
    // Create a new `ListenerSet` and a controller for it.
    pub fn new() -> (ListenerSet, ListenerSetController);

    // Accept an incoming connection.
    pub fn accept(&mut self, bop_handle: &BopHandle) -> BopResult<Stream>;
}

impl ListenSetController {
    pub fn add_listener<A: ToListenEndpoint>(&self, addr: A);
    pub fn remove_listener<A: ToListenEndpoint>(&self, addr: A);

    /// Iterate over the endpoints that the `ListenSet` is listening on.
    pub fn listening_endpoints<'c>(&'c self) -> ListeningEndpoints<'c>
}

impl<'c> Iterator for ListeningEndpoints<'c> {
    type Item = ListeningEndpoint<'c>;
}

/// An endpoint that the `ListenerSet` is listening on.
impl<'c> ListeningEndpoint<'c> {
    /// The local listening address.
    fn local_endpoint(&self) -> &ListenEndpoint;

    /// Iterates over the known external endpoints of this address which other peers may be able to
    /// connect to.
    fn known_endpoints<'e>(&'e self) -> KnownEndpoints<'e>;

    /// Create external mappings for this endpoint and iterate over them.
    fn mapped_endpoints<'e>(&'e self, bop_handle: &BopHandle, mapping_context: &MappingContext)
        -> MappedEndpoints<'e, 'c>
}

impl<'e, 'c> Iterator for KnownEndpoints<'e, 'c> {
    type Item = MappedEndpoint;
}

impl<'e, 'c> Iterator for MappedEndpoints<'e, 'c> {
    type Item = MappedEndpoint;
}

/// The result if mapping an `ListenEndpoint`.
struct MappedEndpoint {
    pub endpoint: Endpoint,
    pub nat_restricted: bool,
}

/// A connection to a peer.
impl Stream {
    pub fn connect<A: ToEndpoint>(bop_handle: &BopHandle, addr: A)
        -> BopResult<Stream>;
    pub fn read(&mut self, bop_handle: &BopHandle, buf: &mut [u8])
        -> BopResult<usize>;
    pub fn write(&mut self, bop_handle: &BopHandle, buf: &[u8])
        -> BopResult<usize>;

    /// Split the `Stream` into reading and writing halves which can be used
    /// independently.
    pub fn split(self) -> (ReadStream, WriteStream);
}

impl From<Stream> for ReadStream { ... }
impl From<Stream> for WriteStream { ... }

impl ReadStream {
    pub fn read(&mut self, bop_handle: &BopHandle, buf: &mut [u8])
        -> BopResult<usize>;
}

/// Read from multiple `ReadStream`s simultaneously
impl<T> ReaderSet<T> {
    pub fn new() -> (ReaderSet, ReaderSetController);
    pub fn read(&mut self, bop_handle: &BopHandle, buf: &mut [u8])
        -> BopResult<usize>;

    /// Block until a `ReadStream` has data ready to be read and then
    /// synchronously calls `callback` and returns it's result.
    pub fn read_inspect<F, R>(&mut self, bop_handle: &BopHandle, callback: F)
            -> R
        where F: for<'e> FnMut(ReadyReadStream<'e, T>) -> R
}

/// Created by `ReaderSet::read_inspect`, this represents a `ReadStream` in the set which has data
/// ready to be read. This type can be read without blocking.
impl<'e, T> ReadyReadStream<'e, T> {
    /// The token the `ReadStream` was registered in the set with.
    pub fn token(&self) -> &T
    /// Remove the `ReadStream` from the `ReaderSet`.
    pub fn remove(self) -> (ReadStream, T)
}

impl std::io::Read for ReadyReadStream {
    // non-blocking impl
}

/// Controls the `ReaderSet` it was created with. Can be dropped to cancel any blocked reads.
impl<T> ReaderSetController<T> {
    /// `ReadStream`s can be registered with a token to identify them.
    pub fn add_reader(&self, reader: ReadStream, token: T);
    pub fn remove_reader(&self, which: &T) -> Vec<(ReadStream, T)>
}

impl WriteStream {
    pub fn write(&mut self, bop_handle: &BopHandle, buf: &[u8])
        -> BopResult<usize>;
}

/// Write to multiple `WriteStream`s simultaneously.
impl<T> WriterSet<T> {
    pub fn new() -> (WriterSet, WriterSetController)
    pub fn write(&mut self, bop_handle: &BopHandle, buf: &[u8])
        -> BopResult<usize>;

    /// Block until a `WriteStream` is ready to be written and then
    /// synchronously calls `callback` and returns it's result.
    pub fn write_inspect<F, R>(&mut self, bop_handle: &BopHandle, callback: F)
            -> R
        where F: for<'e> FnMut(ReadyWriteStream<'e, T>) -> R
}

/// Created by `WriterSet::write_inspect`, this represents a `WriteStream` in the set which is
/// ready to be writen. This type can be written to without blocking.
impl<'e, T> ReadyWriteStream<'e, T> {
    /// The token the `WriteStream` was registered in the set with.
    pub fn token(&self) -> &T
    /// Remove the `WriteStream` from the `WriterSet`.
    pub fn remove(self) -> (WriteStream, T)
}

impl std::io::Write for ReadyWriteStream {
    // non-blocking impl
}

/// Controls the `WriterSet` it was created with. Can be dropped to cancel any blocked writes.
impl<T> WriterSetController<T> {
    /// `WriteStream`s can be registered with a token to identify them.
    pub fn add_writer(&self, writer: WriteStream, token: T);
    pub fn remove_writer(&self, which: &T) -> Vec<(WriteStream, T)>
}
```

# Drawbacks

Needs to be implemented.

# Alternatives

* Leave transport abstraction as part of crust.
* Build a transport abstraction library around mioco instead.

# Unresolved questions

None

