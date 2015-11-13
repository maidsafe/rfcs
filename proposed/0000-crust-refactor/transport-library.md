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

/// Listens on multiple listening endpoints and accepts connections from any of them.
impl Acceptor {
    /// Create a new `Acceptor`
    pub fn new() -> Acceptor;

    /// Create an `Acceptor` which listens on `addr`.
    pub fn from_endpoint<A: ToListenEndpoint>(addr: A) -> Acceptor;
    pub fn add_listener<A: ToListenEndpoint>(&mut self, addr: A) -> bool;
    pub fn remove_listener<A: ToListenEndpoint>(&mut self, addr: A) -> bool;
    pub fn accept<'a>(&'a mut self) -> (AcceptGo<'a>, AcceptorController);
}

impl<'a> Go for AcceptGo<'a> {
    type Output = (AcceptGo<'a>, Stream);
}

impl<'a> AcceptorController<'a> {
    pub fn add_listener<A: ToListenEndpoint>(&self, addr: A) -> bool;
    pub fn remove_listener<A: ToListenEndpoint>(&self, addr: A) -> bool;

    /// Iterate over the endpoints that this `Acceptor` is listening on.
    pub fn accepting_endpoints<'c>(&'c self) -> AcceptingEndpoints<'c, 'a>
}

impl<'c, 'a> Iterator for AcceptingEndpoints<'c, 'a> {
    type Item = AcceptingEndpoint<'c, 'a>;
}

/// An endpoint that this `Acceptor` is listening on.
impl<'c, 'a> AcceptingEndpoint<'c, 'a> {
    /// The local listening address.
    fn listen_endpoint(&self) -> &ListenEndpoint;

    /// Iterates over the known external endpoints of this address which other peers may be able to
    /// connect to.
    fn known_endpoints<'e>(&'e self) -> KnownEndpoints<'e>;

    /// Create external mappings for this endpoint and iterate over them.
    fn mapped_endpoints<'e>(&'e self, mapping_context: &MappingContext)
        -> (MappedEndpointsGo<'e, 'c, 'a>, MappedEndpointsKill<'e, 'c, 'a>),
}
 
/// The result if mapping an `ListenEndpoint`.
struct MappedEndpoint {
    pub endpoint: Endpoint,
    pub nat_restricted: bool,
}

impl<'e, 'c, 'a> Iterator for KnownEndpoints<'e, 'c, 'a> {
    type Item = MappedEndpoint;
}

impl<'e, 'c, 'a> Go for MappedEndpointsGo<'e, 'c, 'a> {
    type Output = (MappedEndpointsGo<'e, 'c, 'a>, MappedEndpoint);
}

impl Stream {
    pub fn connect<A: ToEndpoint>(addr: A) ->
        (ConnectGo, ConnectKill)
    pub fn read<'r>(&'r mut self, buf: &mut [u8]) -> (ReadGo<'r>, ReadKill<'r>)
    pub fn write<'w>(&'w mut self, buf: &[u8]) -> (WriteGo<'w>, WriteKill<'w>)
    pub fn split(self) -> (ReadStream, WriteStream);
}

impl Go for ConnectGo {
    type Output = Stream;
}

impl From<Stream> for ReadStream { ... }
impl From<Stream> for WriteStream { ... }

impl ReadStream {
    pub fn read<'r>(&'r mut self, buf: &mut [u8]) -> (ReadGo<'r>, ReadKill<'r>)
}

impl<'r, T> Go for ReadGo<'r, T> {
    type Output = (ReadGo<'r, T>, usize);
}

/// Read from multiple `ReadStream`s simultaneously
impl<T> ReaderSet<T> {
    pub fn new() -> ReaderSet;
    pub fn from_reader(reader: ReadStream) -> ReaderSet<()>;

    /// `ReadStream`s can be registered with a token to identify them.
    pub fn add_reader(&mut self, reader: ReadStream, token: T);
    pub fn remove_reader(&mut self, which: &T) -> Vec<(ReadStream, T)>;
    pub fn read<'r>(&'r mut self, buf: &mut [u8]) -> (ReadGo<'r>, ReaderSetController<'r, T>)

    /// `ReadInspectGo` will block until a `ReadStream` has data ready to be read and then
    /// synchronously calls `callback` and return's it's result.
    pub fn read_inspect<'r, F, R>(&'r mut self, callback: F)
            -> (ReadInspectGo<'r, T, F, R>, ReaderSetController<'r, T>)
        where F: for<'e> FnMut(ReadyReadStream<'e, 'r, T>) -> R
}

impl<'r, T, F, R> Go for ReadInspectGo<'r, T, F, R> {
    type Output = (ReadInspectGo<'r, T, F, R>, R);
}

/// Created by `ReaderSet::read_inspect`, this represents a `ReadStream` in the set which has data
/// ready to be read. This type can be read without blocking.
impl<'e, 'r, T> ReadyReadStream<'e, 'r, T> {
    /// The token the `ReadStream` was registered in the set with.
    pub fn token(&self) -> &T
    /// Remove the `ReadStream` from the `ReaderSet`.
    pub fn remove(self) -> (ReadStream, T)
}

impl Read for ReadyReadStream {
    // non-blocking impl
}

/// Controls the `ReaderSet` it was created with. Can be dropped to cancel any blocked reads.
impl<'r, T> ReaderSetController<'r, T> {
    pub fn add_reader(&self, reader: ReadStream, token: T);
    pub fn remove_reader(&self, which: &T) -> Vec<(ReadStream, T)>

    /// Downcast to a `ReadKill`.
    pub fn to_kill(self) -> ReadKill<'r>
}

impl WriteStream {
    pub fn write<'w>(&'w mut self, buf: &[u8]) -> (WriteGo<'w>, WriteKill<'w>)
}

impl<'w, T> Go for WriteGo<'w, T> {
    type Output = (WriteGo<'w, T>, usize);
}

/// Write to multiple `WriteStream`s simultaneously.
impl<T> WriteSet<T> {
    pub fn new() -> WriteSet;
    pub fn from_writer(writer: WriteStream) -> WriterSet<()>;

    /// `WriteStream`s can be registered with a token to identify them.
    pub fn add_writer(&mut self, writer: WriterStream, token: T);
    pub fn remove_writer(&mut self, which: &T) -> Vec<(WriteStream, T)>;
    pub fn write<'w>(&'w mut self, buf: &[u8]) -> (WriteGo<'w>, WriterSetController<'w, T>)

    /// `WriteInspectGo` will block until a `WriteStream` is ready to be written and then
    /// synchronously calls `callback` and return's it's result.
    pub fn write_inspect<'w, F, R>(&'w mut self, callback: F)
            -> (WriteInspectGo<'w, T, F, R>, WriterSetController<'w, T>)
        where F: for<'e> FnMut(ReadyWriteStream<'e, 'w, T>) -> R
}

impl<'w, T, F, R> Go for WriteInspectGo<'w, T, F, R> {
    type Output = (WriteInspectGo<'w, T, F, R>, R);
}

/// Created by `WriterSet::write_inspect`, this represents a `WriteStream` in the set which is
/// ready to be writen. This type can be written to without blocking.
impl<'e, 'w, T> ReadyWriteStream<'e, 'w, T> {
    /// The token the `WriteStream` was registered in the set with.
    pub fn token(&self) -> &T
    /// Remove the `WriteStream` from the `WriterSet`.
    pub fn remove(self) -> (WriteStream, T)
}

impl Write for ReadyWriteStream {
    // non-blocking impl
}

/// Controls the `WriterSet` it was created with. Can be dropped to cancel any blocked writes.
impl<'w, T> WriterSetController<'w, T> {
    pub fn add_writer(&self, writer: WriteStream, token: T);
    pub fn remove_writer(&self, which: &T) -> Vec<(WriteStream, T)>

    /// Downcast to a `WriteKill`.
    pub fn to_kill(self) -> WriteKill<'w>
}
rust
```

# Drawbacks

Needs to be implemented.

# Alternatives

* Leave transport abstraction as part of crust.
* Build a transport abstraction library around mioco instead.

# Unresolved questions

* None (for now, there were some here but I feel I now have satisfactory
  answers for them)
