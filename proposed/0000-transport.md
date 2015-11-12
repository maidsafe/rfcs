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

The API is blocking based API, similar to the Rust standard library, and pursues the
following design goals:
 * It should be possible to read, write or accept on multiple sockets
   simultaneously.
 * If the user is waiting for an event they should *only* be waiting for that
   event. If another part of the program wants to wait for some other event let
   it do so in another thread.
 * Any functions that may be expected to block for more than a second should
   provide a way to be cancelled asynchronously.
 * To the extant that it's possible, the user should not be able to get into a
   state where a thread is blocked indefinitely with no way to unblock it.

To satisfy these goals, the library is built around mio's cross-platform epoll
implementation. 

# Detailed design

This section walks through the logic followed in designing the API. Part of
this design has been implemented (the `Acceptor` section) in order to make sure
that the ideas are sound and implementable.

In what follows, the implementation of types and functions has generally been
omitted as the purpose here is to outline the API. `Result`/`Error` usages have
also been omitted for clarity. In the real API most of these functions will
return `Result`s and, thanks to the blocking-based design, errors can simply be
handled where they occur.

## Base types

Firstly, we need some types for streams and addresses.

```rust
// A connection to a peer.
type Stream;

// An endpoint that we can connect to.
type Endpoint;

// An endpoint that we can listen for incoming connections on.
type ListenEndpoint;
```

## Accepting connections

Now we need a way to accept incoming connections. One way to do this would be
to simply have an `accept` function which blocks waiting for a connection.

```rust
fn accept(addr: ListenEndpoint) -> Stream
```

However this conflicts with our requirement that we need to be able to cancel
blocking operations at will. To fix this, we can evolve the design to instead
use two objects - one which can be used to perform the blocking call and
another which can be used to cancel the blocking call.

```rust
fn acceptor(addr: ListenEndpoint) -> (Acceptor, AcceptorController)

impl Acceptor {
    // Returns `None` if the call was cancelled.
    fn accept(&mut self) -> Option<Stream>
}

impl Drop for AcceptorController {
    // unblock the corresponding `accept()` call
}
```

However, this design can still be improved. Currently, `accept()` may unblock
due to being cancelled but the user can immediately call `accept()` again. If
we allow the user to accept multiple times on the same endpoint (as they are
likely to want to do) this will put the thread into a state where there is no
clean way to unblock it. To fix this, we can make the `accept()` call consume
the `Acceptor` and return it again for reuse on a successful connection.

```rust
impl Acceptor {
    fn accept(self) -> Option<(Acceptor, Stream)>
}
```

So far, so good. However the user may want to accept on multiple
`ListenEndpoint`s simultaneously. For example, they want to listen for incoming
TCP and UtP connections on all network interfaces. They may also want to change
this set of listening endpoints dynamically as configurations are changed or
interfaces are added or removed. We can further evolved the design to handle
these use cases.

```rust
impl Acceptor {
    fn new() -> Acceptor;
    fn start<'a>(&'a mut self) -> (AcceptorReactor<'a>, AcceptorController<'a>)
}

impl<'a> AcceptorReactor<'a> {
    fn accept(self) -> Option<(AcceptorReactor<'a>, Stream)>
}

impl<'a> AcceptorController<'a> {
    fn add_listener(&self, addr: ListenEndpoint);
    fn remove_listener(&self, addr: ListenEndpoint);
}

impl<'a> Drop for AcceptorController<'a> {
    // unblock the corresponding `accept()` call
}
```

This design allows us to build a set of listening endpoints and accept on all
of them simultaneously. Under the surface, this can be implemented efficiently
using a `mio::Poll`. It also allows us to start accepting again on a set of
endpoints without having to build the set again from scratch.

We now have a fully-featured API for accepting incoming connections. The
following example shows how this API can be used to listen on a single endpoint
for a specified duration before timing out.

```
fn accept_timeout(listen: ListenEndpoint, timeout: Duration) -> Option<Stream> {
    let mut acceptor = Acceptor::new();
    let (acc_reactor, acc_controller) = acceptor.start();
    crossbeam::scope(|scope| {
        let conn = scope.spawn(move {
            match acc_reactor.accept() {
                None => {
                    println!("Timed out");
                    None
                },
                Some((_, conn)) => {
                    println!("Got a connection {:?}", conn);
                    Some(conn)
                }
            }
        });
        acc_controller.add_listener(listen);
        thread::sleep(timeout);
        drop(acc_controller);
        conn.join()
    })
}
```

Note that although this design is fully-featured, it is still somewhat
cumbersome to use. A complete API, outlined towards the end of this RFC,
includes several extra convenience methods such as an `accept_timeout` method.

## Making connections.

We want a method that will return a connection to a remote endpoint but, again,
must be asynchronously stoppable. Following the same logic as for accepting
connections we arrive at something like this:

```rust
impl Connector {
    fn new(addr: Endpoint) -> (Connector, ConnectorController)
    fn connect(self) -> Option<Stream>;
}

impl Drop for ConnectorController {
    // unblock corresponding connect call
}
```

## Reading from streams.

One way to implement reading from a stream could be to add a method like this.

```rust
impl Stream {
    fn read(&mut self, buf: &mut [u8]) -> usize;
}
```

However, borrowing the `Stream` mutably means we are unable to do anything else
with the `Stream` while we are blocked on reading, including writing to it or
cancelling the read. On the other hand, if we borrow the `Stream` immutably
then the user could read from the one `Stream` on multiple threads
simultaneously (unless we disable `Sync`, but then we would still be unable to
write to the stream concurrently). The solution is firstly to recognise that
`Stream` consists of two independent streams - an input and an output.

```rust
impl Stream {
    fn split(self) -> (ReadStream, WriteStream)
}

impl ReadStream {
    fn read(&mut self, &mut [u8]) -> usize
}
```

I think it's debatable whether we should have a `Stream` type at all or
whether functions that return `Stream`s should just return a `ReadStream`
and `WriteStream` directly. But for now, assume we do use a `Stream` type.

The above design is still incomplete. We need to be able to cancel the read
operation and we also need a way to read on multiple `ReadStream`s
simultaneously and react to whichever receives data next. Let's solve the
second problem first:

```rust
impl ReadStream {};

impl ReaderSet {
    fn new() -> ReaderSet;
    fn add_reader(&mut self, reader: ReadStream)
    fn read(&mut self, buf: &mut [u8]) -> usize
}
```

Now we can read on multiple streams simultaneously. However we might want to
know which `ReadStream` received a packet. We also need a way to remove
specific `ReadStream`s from the set. One solution would be to use the stream's
`Endpoint` for these purposes but a more general solution is to index the
streams by an arbitrary type. In this design, it is assumed that multiple
`ReadStream`s can be indexed by identical keys (ie. we have a multimap). This
allows the user to index by `()` if they don't need to keep track of individual
`ReadStream`s.

```rust
impl<T> ReaderSet<T> {
    fn add_reader(&mut self, reader: ReadStream<'s>. token: T)
    fn remove_reader(&mut self, which: &T) -> Vec<(ReadStream, T)>
    fn read(&mut self, buf: &mut [u8]) -> (&T, usize)
}
```

Now lets solve the problem of being able to cancel the read. Following the same
design pattern as we did for accepting connections brings us to:

```rust
impl<T> ReaderSet<T> {
    fn new() -> ReaderSet;
    fn start<'r>(&'r mut self) -> (ReaderSetReactor<'r, T>, ReaderSetController<'r, T>)
}

impl<'r, T> ReaderSetReactor<'r, T> {
    fn read<F, R>(self, buf: &mut [u8], callback: F) -> R
        where F: FnOnce(usize, &T) -> R
}

impl<'r, T> ReaderSetController<'r, T> {
    fn add_reader(&self, reader: ReadStream, token: T);
    fn remove_reader(&self, which: &T) -> Vec<(ReadStream, T)>
}

impl<'r, T> Drop for ReaderSetController<'r, T> {
    // unblock the corresponding `read()` call
}
```

The first thing to note is that `read()` now takes a callback. This is called
synchronously with a reference to the token for the receiving `ReadStream`. The
`read` call returns the value returned by the callback. This setup is necessary
because `read` moves `self`, and thus we can't just return a `&T` that
references into `self`. We could clone the token and return a `T` but that may
be inefficient and would require enforcing `T: Clone`, if the user wants this
behaviour they can just pass `T::clone` as the callback to get the same effect.

One last way we could generalise this design is to allow the user to decide
what they want to do with a `ReadStream` when it becomes ready. For example,
they may want to read into a different buffer depending on which stream is
readable.

```rust
impl<'r, T> ReaderSetReactor<'r, T> {
    fn next_reader<F, R>(self, F: callback) -> R
        where F: for<'e> FnOnce(ReadyReadStream<'e, 'r, T>) -> Option<(ReaderSetReactor<'r, T>, R)>
}

impl<'e, 'r: 'e, T> ReadyReadStream<'e, 'r, T> {
    fn token(&self) -> &T
    fn remove(self) -> (ReadStream, T)
}

impl<'e, 'r: 'e, T> Read for ReadyReadStream<'e, 'r, T> {
    ...
}
```

In this design, the implementation of `Read` for `ReadyReadStream` is
NON-blocking to allow the user to read from it until it is drained completely.
This is compatible with our blocking-based design philosophy as the user must
use a `ReaderSet` to block until data becomes available, at which point they
are provided with a `ReadyReadStream` to read it with.

We now have a fully-featured API for reading from streams. An example of it's
usage follows.

```rust
fn read_timeout(reader: ReadStream, buf: &mut [u8], timeout: Duration) -> Option<usize> {
    let mut rs = ReaderSet::new();
    let (rs_reactor, rs_controller) = rs.start();
    crossbeam::scope(|scope| {
        let ret = scope.spawn(move || {
            rs_reactor.next_reader(|rr| rr.read(buf)).map(|(_, x) x|)
        })
        rs_controller.add_reader(reader);
        thread::sleep(timeout);
        ret.join();
    })
}
```

## Writing to streams

Developing the API for writing to streams follows that exact same logic as for
reading from streams. As such we end up with an identical API (described in
full towards the end of this RFC).

## Nat traversal

So far we have ignored the problem of network address translation. One of the
goals of this library is to provide a way to create streams through NATs.

First we need a way to accept connections from behind a NAT. This means both
being aware of what addresses can be used to connect to us and trying to create
more endpoints through whatever techniques the NAT traversal library provides.

```rust
impl<'a> AcceptorController<'a> {
    fn accepting_endpoints<'c>(&'c self) -> AcceptingEndpoints<'c, 'a>
}

impl<'c, 'a> Iterator for AcceptingEndpoints<'c, 'a> {
    type Item = AcceptingEndpoint<'c, 'a>;
}

impl<'c, 'a> AcceptingEndpoint<'c, a> {
    fn listen_endpoint(&self) -> &ListenEndpoint;
    fn known_endpoints(&self) -> Endpoints;
    fn mapped_endpoints<'m>(&'m self, mapping_context: &MappingContext) -> (MappedEndpoints<'m, 'c, 'a>, MappedEndpointsController<'m, 'c, 'a>),
}

impl Iterator for Endpoints {
    type Item = Endpoint;
}

impl MappedEndpoints<'m, 'c, 'a> {
    fn next_endpoint(self) -> Option<(MappedEndpoints<'m, 'c, 'a>, Endpoint)>
}

impl<'m, 'c, 'a> Drop for MappedEndpointsController<'m, 'c, 'a> {
    // unblock any `next_endpoint` call
}

```

Here, `accepting_endpoints` returns an iterator that we can use to iterate over
all the endpoints that we are accepting on in the form of an
`AcceptingEndpoint`. The methods of interest here are `known_endpoints` which
allows the user to iterate over all the currently known external endpoints for
this listener and `mapped_endpoints` which does the same but asynchronously
attempts to create external mappings for this listener. Successfully mapping a
socket can mean one of two things. If we are able to use UPnP to create a
mapping or we use hole punching but detect that we are behind a full-cone NAT
then the mapped endpoint can be considered to be fully functional.
Fully-function here means that any machine outside our local network should be
able to connect to it. On the other hand, if we are behind a more restrictive
form of NAT and UPnP is not available then we only have a semi-functional
endpoint. This endpoint will, at best, allow us to receive data only from peers
that we have already messaged through it. Connecting to such an endpoint
requires that the remote peer is aware of our attempt to connect to it so that
it can manually accept the connection using a rendezvous connection procedure.

Whether an endpoint is NATted or not is relevant both when we map an endpoint
and when we attempt to connect to an endpoint. As such it should be part of the
`Endpoint` type. An `Endpoint` type that allows us to consider TCP, UtP and
NAT-restricted UtP endpoints thus looks like:

```rust
enum Endpoint {
    Tcp(SocketAddr),
    Utp(SocketAddr),
    NatRestrictedUtp(SocketAddrV4),
}
```

Making a connection from A to B where B is a NATted endpoint is a three stage
process. First, A initiates the connection and in doing so generates
rendezvous info that B can use to accept the connection. This info is then
routed to B out-of-band by the user. B then uses this info to accept the
connection. The APIs for this process are shown below:

```rust
struct RendezvousInfo {
    target_endpoint: Endpoint,
    mapped_endpoints: Vec<Endpoint>,
    secret: [u8; 4],
}

impl Connector {
    fn new(addr: Endpoint, mapping_context: Option<&nat_traversal::MappingContext>) -> (Connector, ConnectorController)
    fn rendezvous_info(&self) -> Option<RendezvousInfo>
}

impl Drop for ConnectorController {
    // unblock rendezvous_info and connect calls
}

impl<'a> AcceptorController<'a> {
    fn rendezvous_acceptor<'c>(&'c self, info: RendezvousInfo) -> (RendezvousAcceptor<'c, 'a>, RendezvousAcceptorController<'c, 'a>)
}

impl<'c, 'a> RendezvousAcceptor<'c, 'a> {
    fn accept(self) -> (RendezvousInfo, Option<Stream>)
}

impl<'c, 'a> Drop for RendezvousAcceptorController<'c, 'a> {
    // unblock the accept call
}
```

First off, we add an extra parameter to the `Connector::new` function. If the
`Connector` sees that it needs to perform a rendezvous connect then it may need
to use igd and external hole punching servers in order to perform the
connection. After the `Connector` obtains a mapped socket it puts any
information needed to accept the connection into a `RendezvousInfo` which the
user can obtain through the `rendezvous_info` method. The `RendezvousInfo`
struct contains the endpoint that A is trying to connect to, a list of all the
mapped endpoints of the socket A is using to perform the connection, and a
secret needed to accept the connection. If any of A's mapped endpoints for this
connection are NATed it starts the hole punching procedure, otherwise it simply
waits for B to connect to it. While this is happening, the user routes the
rendezvous info to the target node. The target node accepts the connection
through the appropriate `Acceptor` using the `rendezvous_acceptor` method. This
method checks whether it can connect directly to the initiator's endpoint. If
so it does so, otherwise it starts the hole-punching procedure.

## Full API

Here is a fleshed-out version of the API described above. Error-type usages are
still omitted for clarity.

```rust
type Stream;

type Endpoint;

trait ToEndpoint {
    type Err;
    fn to_endpoint(self) -> Result<Endpoint, Self::Err>;
}

impl ToEndpoint for Endpoint { ... }
impl ToEndpoint for ListenEndpoint { ... }
impl ToEndpoint for &str { ... }

type ListenEndpoint;

trait ToListenEndpoint {
    type Err;
    fn to_listen_endpoint(self) -> Result<ListenEndpoint, Self::Err>
}

type Acceptor;
type AcceptorReactor<'a>;
type AcceptorController<'a>;

impl Acceptor {
    fn new() -> Acceptor;
    fn from_endpoint<A: ToListenEndpoint>(addr: A) -> Acceptor;
    fn add_listener<A: ToListenEndpoint>(&mut self, addr: A) -> bool;
    fn remove_listener(&mut self, addr: ListenEndpoint) -> bool;
    fn start<'a>(&'a mut self) -> (AcceptorReactor<'a>, AcceptorController<'a>)
}

impl<'a> AcceptorReactor<'a> {
    fn accept(self) -> Option<(AcceptorReactor<'a>, Stream)>
    fn accept_timeout(self) -> Option<(AcceptorReactor<'a>, Option<Stream>)>
}

impl<'a> AcceptorController<'a> {
    fn add_listener(&self, addr: ListenEndpoint);
    fn remove_listener(&self, addr: ListenEndpoint);
    fn rendezvous_acceptor<'c>(&'c self, info: RendezvousInfo) -> (RendezvousAcceptor<'c, 'a>, RendezvousAcceptorController<'c, 'a>)
}

impl<'a> Drop for AcceptorController<'a> {
    // unblock the corresponding `accept()` call
}

type Connector;
type ConnectorController;

impl Connector {
    fn new<A: ToEndpoint>(addr: A, mapping_context: Option<&nat_traversal::MappingContext>) -> (Connector, ConnectorController)
    fn rendezvous_info(&self) -> Option<RendezvousInfo>
    fn connect(self) -> Option<Stream>;
}

impl Drop for ConnectorController {
    // unblock corresponding connect call
}

type ReadStream;
type ReaderSet;
type ReadyReadStream;

impl ReaderSet {
    fn new() -> ReaderSet;
    fn from_reader(&mut self, reader: ReadStream) -> ReaderSet<()>
    fn add_reader(&mut self, reader: ReadStream, token: T);
    fn remove_reader(&mut self, which: &T) -> Vec<(ReadStream, T)>
    fn start<'r>(&'r mut self) -> (ReaderSetReactor<'r>, ReaderSetController<'r>)
}

impl<'r, T> ReaderSetReactor<'r, T> {
    fn read(self, buf: &mut [u8]) -> Option<(ReaderSetReactor<'r, T>, usize)>
    fn read_timeout(self, buf: &mut [u8], timeout: Duration) -> Option<(ReaderSetReactor<'r, T>, Option<usize>)>
    fn next_reader<F, R>(self, F: callback) -> R
        where F: for<'e> FnOnce(ReadyReadStream<'e, 'r, T>) -> Option<(ReaderSetReactor<'r, T>, R)>
}

impl<'e, 'r, T> ReadyReadStream<'e, 'r, T> {
    fn token(&self) -> &T
    fn remove(self) -> (ReadStream, T)
}

impl Read for ReadyReadStream {
    // non-blocking impl
}

type WriteStream;
type WriterSet;
type ReadyWriteStream;

impl WriterSet {
    fn new() -> WriterSet;
    fn from_writer(&mut self, writer: WriteStream) -> WriterSet<()>
    fn add_writer(&mut self, writer: WriteStream, token: T);
    fn remove_writer(&mut self, which: &T) -> Vec<(WriteStream, T)>
    fn start<'r>(&'r mut self) -> (WriterSetReactor<'r>, WriterSetController<'r>)
}

impl<'r, T> WriterSetReactor<'r, T> {
    fn write(self, buf: &mut [u8]) -> Option<(WriterSetReactor<'r, T>, usize)>
    fn write_timeout(self, buf: &[u8], timeout: Duration) -> Option<(WriterSetReactor<'r, T>, Option<usize>)>
    fn next_writer<F, R>(self, F: callback) -> R
        where F: for<'e> FnOnce(ReadyWriteStream<'e, 'r, T>) -> Option<(WriterSetReactor<'r, T>, R)>
}

impl<'e, 'r, T> ReadyWriteStream<'e, 'r, T> {
    fn token(&self) -> &T
    fn remove(self) -> (WriteStream, T)
}

impl Write for ReadyWriteStream {
    // non-blocking impl
}

impl From<Stream> for ReadStream { ... }
impl From<Stream> for WriteStream { ... }

struct RendezvousInfo {
    target_endpoint: Endpoint,
    mapped_endpoints: Vec<Endpoint>,
    secret: [u8; 4],
}

impl<'c, 'a> RendezvousAcceptor<'c, 'a> {
    fn accept(self) -> (RendezvousInfo, Option<Stream>)
}

impl<'c, 'a> Drop for RendezvousAcceptorController<'c, 'a> {
    // unblock the accept call
}
```

# Drawbacks

Needs to be implemented.

# Alternatives

* Leave transport abstraction as part of crust.
* Build a transport abstraction library around mioco instead.

# Unresolved questions

* How easy will it be to use these `*Controller` APIs the way they're intended?
  In practice, the user can always just ignore the controller and perform the
  blocking call directly eg.

  ```rust
  let (conn, _c) = Connector::new(addr, None);
  let stream = conn.connect();
  ```

  However this defeats the point. The intention is that the user can always be
  in position to - for example - cancel any pending operations if they receive
  a signal to shutdown. What lengths will they have to go to keep themselves in
  this position as their becomes more complicated?

* Should we have a `Stream` type? Or just `ReadStream` and `WriteStream` types?

