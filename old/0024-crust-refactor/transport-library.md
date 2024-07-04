Feature Name: transport_library
- Status: proposed
- Type: enhancement
- Related components: crust
- Start Date: 11-11-2015
- Discussion: https://github.com/maidsafe/rfcs/issues/107
- Supersedes:
- Superseded by:

# Summary

This RFC outlines the design of a transport abstraction library to sit beneath
crust.

# Motivation

Crust is intended to be a connection management library for use in implementing
P2P networks. One of the features crust provides is the ability to abstract
over various transport protocols, allowing the user to send data over TCP, uTP
and in the future possibly other transport protocols.

Crust's low-level transport abstraction layer can be seen as logically
independent from the rest of the system. It doesn't need to have knowledge of
higher-level concepts to do with bootstrapping and routing. It's role, as far
as this RFC is concerned, is solely to make and receive encrypted,
protocol-generic connections. As this is indepently useful functionality, this
RFC proposes it be split off into a seperate, reusable library. Having this
functionality in a seperate place will also ease crust's eventual transition to
mio.

# Detailed design

## Endpoints

If we're going to accept and make connections the first thing we need is
addresses to connect to and accept on. This RFC proposes that crust's
`Endpoint` type be split into two types: `Endpoint`s which are addresses that
can be connected to, and `ListenEndpoint`s which are addresses that can be
listened on.

```
type Endpoint;
type ListenEndpoint;
```

There are also corresponding `ToEndpoints` and `ToListenEndpoints` traits that
mirror the `ToSocketAddrs` trait in the standard library.

## Streams

Connections are renamed to streams in order to bring them in line with rust's
naming conventions (`TcpStream`, `UtpStream` etc.). A `Stream` may be either a
TCP or a uTP stream. A `Stream` can be created by connecting directly to an
endpoint:

```
fn Stream::direct_connect<E: ToEndpoints>(E) -> Result<Stream>
```

`Stream`s can also be created via rendezvous connect:

```
fn Stream::rendezvous_connect(&MappingContext, PrivRendezvousInfo, PubRendezvousInfo) -> Result<Stream>
```

`Stream`s implement `Read + Write` and can also be split into separate reader
and writer components:

```
fn Stream::split(self) -> (ReadStream, WriteStream)
```

Currently, crust streams have an unbounded write buffer. This allows calls to
`Service::send` to always return immediately but means that if a crust user
tries to write data to the network faster than the machine can send it they
will eventually fill up their outgoing buffer and crash. One way to fix this
would be to make `send` a blocking call or make `send` return an error on
failed write then have the higher layers attempt to `send` the same data again.
A more practical way would be to switch to non-blocking io (ie. mio). As such,
this RFC proposes that each `Stream`/`WriteStream` maintains an unbounded
buffer and it's own writer thread in order to keep the current behaviour until
we are ready to switch to mio.

## Listeners

A `Listener` listens on a TCP or uTP `ListenEndpoint` and accepts incoming
connections. It is created similarly to a `TcpListener` or `UtpListener`:

```
fn Listener::bind<E: ToListenEndpoints>(&MappingContext, E) -> Result<Listener>
```

`Listener::bind` takes a `&MappingContext` so that it can create and discover
as many connectable endpoints for itself as possible. Once created, you can
accept connections on a `Listener` and query it's endpoints.

```
fn Listener::accept(&self) -> Result<Stream>
fn Listener::endpoints(&self) -> Result<EndpointIterator>
```

## ListenerSet

A `ListenerSet` is a set of listeners. Once created you can add `Listener`s to
it and accept on all of them at once.

```
fn ListenerSet::new(&MappingContext) -> ListenerSet
fn ListenerSet::add_listener(&self, Listener);
fn ListenerSet::accept(&self) -> Result<Stream>
```

`ListenerSet`s can also be used to make rendezvous connections:

```
fn ListenerSet::rendezvous_connect(&self, PrivRendezvousInfo, PubRendezvousInfo) -> Result<Stream>
```

This is the purpose of the `ListenerSet` type. Sometimes it may be possible for
a peer to directly connect to another peer's static listener but not possible
for them to be able to hole punch to the peer. In these cases, rendezvous
connect needs to make use of the static listener addresses. As such, rendezvous
connect is necessarily coupled with accepting direct connections.
`ListenerSet::rendezvous_connect` uses `Stream::rendezvous_connect` underneath
but it also shares it's listener addresses with the other peer. If the peer
manages to connect to one of these addresses, the resulting stream will be
returned from `ListenerSet::rendezvous_connect` rather than from
`ListenerSet::accept`. As such, crust will know that connections returned from
`ListenerSet::accept` are always bootstrap connections.

Note that it's not necessary for a user of this library to make use of
`ListenerSet`. A simple networking application that just does direct
connections may just use `Listener` and `Stream::direct_connect`. Another may
just use `Stream::rendezvous_connect`. `ListenerSet` is intended for slightly
more advanced users of this library (such as crust) that want to make use of
both kinds of connection.

## Encryption

This library is a natural and easy place to implement encryption. Seeing as we
are already inventing our own `Endpoint` type (which contains information on
socket address and protocol) we can extend this type to also contain
cryptographic information. This library proposes that `Endpoint` contain a
public key of the peer and `ListenEndpoint` a private key. Thus when doing a
direct connect, we have the peers public key and everything can be encrypted
from the first byte.

The initial implementation of this library will not contain encryption as we
will first need to seperate this code out from crust and get it working again.
Once we've done that, encryption can easily be added as we know where it will
need to be implemented.

## Serialisation

In order to avoid having two layers of serialisation and to make the API
resemble that of regular rust streams, `Stream`s transport plain binary data
(via the `Read` and `Write` traits) which is encrypted over the wire. As such,
serialisation of crust messages happens only in crust. The heartbeat also
happens at the crust level as this would otherwise require a seperate message
type or the ability to support zero-sized writes.

# Drawbacks

None that I see.

# Alternatives

* Not do this.
* Not make endpoints contain cryptographic information and instead move
  encryption into crust. The advantage of this would be to not have to share
  our public key alongside our IP address, port and protocol info.

# Unresolved questions

None.

# Future work

* Replace `{Pub,Priv}RendezvousInfo` with a channel. This will allow us to
  greatly improve rendezvous connect but will require modifying the nat
  traversal and routing crates.
* Make endpoints representable as strings (perhaps in a format like
  `"utp://[::1]:345/?pubkey=0123456789abcdef"`)
* Make `Stream`, `Listener` and `ListenerSet` asynchronous and switch to mio.
