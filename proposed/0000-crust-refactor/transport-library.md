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
whatever. The streams are non-blocking and implement the `mio::Evented` trait
for use with mio/mioco.

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

/// Listen on an endpoint for incoming connections.
impl StreamListener {
    pub fn bind<A: ToListenEndPoint>(addr: A) -> StreamListener;
    pub fn accept(&mut self) -> io::Result<Stream>;
}

// A connection to a peer.
impl Stream {
    pub fn connect<A: ToEndpoint>(addr: A)
        -> io::Result<Stream>;
    pub fn read(&mut self, buf: &mut [u8])
        -> io::Result<usize>;
    pub fn write(&mut self, buf: &[u8])
        -> io::Result<usize>;

    /// Split the `Stream` into reading and writing halves which can be used
    /// independently.
    pub fn split(self) -> (ReadStream, WriteStream);
}

impl From<Stream> for ReadStream { ... }
impl From<Stream> for WriteStream { ... }

impl ReadStream {
    pub fn read(&mut self, buf: &mut [u8])
        -> io::Result<usize>;
}

impl WriteStream {
    pub fn write(&mut self, buf: &[u8])
        -> io::Result<usize>;
}
```

# Drawbacks

Needs to be implemented.

# Alternatives

* Leave transport abstraction as part of crust.

# Unresolved questions

How to implement the cancellable blocking calls that were talked about in
earlier revisions of this RFC. It's possible that these could be implemented as
part of mioco itself.

