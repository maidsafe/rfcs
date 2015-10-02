- Feature Name: UDP-hole-punching
- Type new: feature
- Related components: crust
- Start Date: 22-09-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

In this text we describe an implementation of UDP hole punching process
to allow for P2P connections of nodes behind NATs in a fully decentralised
network.

# Motivation

Most users of the Crust library are expected to run the software on home PCs connected
to the internet through a router. Routers usually implement some kind of a firewall to
prevent anyone from the outside internet to connect to user's PC directly. As Crust
is a library that concentrates at P2P connections, it should try available techniques
to circumvent these firewall restrictions and UDP hole punching is one such
technique.

Additionally, traditional UDP hole punching process involves a well known
server to tell each peer's their external UDP endpoints. Since our intent
is a fully decentralised system, this text also describes how nodes to which
we're already connected can be used as a replacement for such server.

In the following text the reader is expected to know basic NAT types and how they work.
The section 'Methods of translation' on [wikipedia](https://en.wikipedia.org/wiki/Network_address_translation)
gives a nice overview.

# Detailed design

The UDP hole punching process described here consists of two steps:

1. Finding an external mapping of a UDP socket
2. Do the actual hole punching.

## Finding an external mapping of a UDP socket

Suppose, we have two nodes `A` and `B`, where each one is possibly behind a NAT and
they want to connect to each other.  Routing needs to send `A` an endpoint of `B` and
vice versa. But `A` and `B` don’t know their public
endpoints a priori (they may know their IP addresses from previous connection establishments,
but not their ports). Each node therefore needs to create a UDP socket and use it to ask some
other node it is connected to what the public endpoints of the UDP socket is. We’ll refer
to such helping nodes `C(A)` and `C(B)` (it needs not be the same node for `A` and `B`).
We'll call `U(C(X))` the UDP port on which X can contact `C(X)` in order to find its public
endpoint. `X` finds out about `U(C(X))` through the initial handhsake, that is, when
`X` connects to `C(X)`.

In the following text we’ll describe steps taken at node `A`, but the same steps need to
be taken at node `B`.

For `A` to find out its external address `A` first needs to create a UDP socket `S(A)`
with an arbitrary local port.  `A` will then use `S(A)` to periodically send
datagrams to `U(C(A))` containing a MAGIC number and the ID of the request. Each time
`C(A)` receives such datagram, it sends back a datagram containing public endpoint
of `S(A)` together with the request ID.

The above protocol shall be initiated from upper layers by calling

    Service::get_mapped_udp_socket(&self, result_token: u32)

And the result of such call shall be an event `OnUdpSocketMapped` holding a structure

    struct MappedUdpSocket {
        result_token: u32,
        udp_socket: UdpSocket,
        public_address: SocketAddr, // of node A
    }

Once upper layers receive such event, they can send/route `MappedUdpSocket::public_address`
to node `B`. Once `B` does the same, and upper layers receive `B`’s public endpoint, upper
layers are ready for the actual hole punching.

## Hole punching

The act of hole punching shall be initiated by a function with the following signature:

    Service::udp_punch_hole(&self,
                            result_token: u32,
                            udp_socket : UdpSocket,
                            secret: Option<[u8; 4]>,
                            peer_addr : mut SocketAddr /* of node B */)

This call will initiate reading on the `udp_socket` and will also
start periodically sending small datagrams to the `peer_addr`.
These datagrams will be of type:

    struct HolePunch {
        request_id: u32,
        secret: Option<[u8; 4]>,
        ack: bool,
    }

At this stage, we may receive the above datagram either from `peer_addr`
or from some other endpoint. If the later, it can mean one of two things:

* The other end is behind a Symmetric NAT and we're behind a Full-cone or no NAT.
  This is OK but we need to adjust the `peer_addr` variable to point
  to this new peer.
* Some other application is sending us irrelevant or malicious packets. We
  can distinguish this case by matchin the `secret` passed to the function as an
  argument with `datagram.secret`. If secrets don't match, we ignore the datagram.

Once we receive a datagram from the remote peer, we know our hole has been
punched. Since the other end is waiting for our datagram as well, we need
to make a "best effort" to ensure the other end receives our packet as well.
That is, we need to make sure at least one of the following conditions is true:

1. We sent our datagram to `peer_addr` `K` times (for some predefined `K`).
2. We receive `HolePunch` packet with the `ack` field set to true.

Note that the condition ((1) or (2)) will always be satisfied eventually,
so the decision whether hole punching was successfull or not shall
be based purely on the fact whether we have received a datagram or not, that is,
once ((1) or (2)) is true, we check whether we've received a datagram,
if we did, we call the `callback` with `(udp_socket, Ok(peer_addr))`,
otherwise we call it with an appropriate `io::Error`.

Result of this call shall be sent to the user as an `Event::OnHolePunched` holding
the following structure:

    struct HolePunchResult {
      result_token: u32,
      udp_socket: UdpSocket,
      peer_addr: io::Result<SocketAddr>,
    }

## Using the punched hole

After a successful hole-punch, it is assumed that the two peers can communicate
with each other over the unreliable UDP protocol. At this point we'd like to
use that UDP socket with protocols such as uTP for added reliability.

Unfortunatelly, the rust-utp library doesn't currently support rendezvous
connections, so this functionality will need to be added.

# Drawbacks

The alternative approach described in the next section is simpler in that it doesn't require
two async calls, instead, it uses only one. Other than that, the simpler approach
is limited in number of NAT types it can successfully punch through.

# Alternatives

Another approach would work with the use of multiplexing. That is, if we had a UDP socket
which is already connected to a remote peer, we could also use it to communicate
with other peers as the hole has already been punched. Problem with this approach is 
that it would only help with `Full-cone NAT` types because it is the only NAT
type where a hole punched to one peer/host can be reused with another peers/hosts.

# Unresolved questions

## What asynchronous primitives should be used for this?

Here are the available options:

1. Enum events over channels. This is what Routing is currently using. It is an improvement
   from the visitor pattern we had before, but is still very hard to use. The main
   disadvantage is that with this approach it is most inconveniet to combine
   two or more async calls into one. E.g. in our current context, we'd like to
   combine the `get_mapped_udp_socket` call with the `udp_punch_hole` call.
   Using event enums approach, user needs to hold a state outside of the Crust library
   to know what should happen after `get_mapped_udp_socket` event arrives.

2. Coroutines: these are nice primitives to work with, they do allow for two
   or more coroutines to be combined together. However they are not generic
   enough to express all different combinations asynchronous programming has to offer.
   I.e. coroutines only allow for sequentinal combination of coroutines,
   more advanced patterns (such as "start many async actions and continue when
   all of them finish") need to be complemented with other approaches.

3. Futures: these (AFAIK) have bigger combinatorial power than coroutines and
   many libraries already ship with these patterns abstracted (think of functions
   like `when_any` or `when_all`). That said, they have the disadvantage
   that they combine blocking and non blocking paradigms, resulting in 
   slower code (read [this](http://www.open-std.org/jtc1/sc22/wg21/docs/papers/2014/n4045.pdf) for
   more info on the topic).

4. Continuation Passing Style: Callbacks are ligthweight, fast and offer
   biggest combinatorial power. This comes with the price that they are often
   referred to as hard to read. On the other hand, the combinatorial power allows
   for complex and hard-to-read patterns to be hidden behind
   other callback taking functions with more descriptive names.

## How to chose `C(X)`?

One criterion for `C(X)` is that it should not be on the same local network as `X`.
For this we could use the `getifaddrs` function, it gives us `SocketAddrs`
of our network interfaces, plus the netmasks they use. This information should
be enough to determine whether `C(X)` is in the same subnet as a given
interface.

## Should `C(X)` be only one node?

If we allow for `C(X)` to be more than one node, we could get a reponse quicker
and more reliable. Additionally getting more than one response can reveal
the information whether we're behind a Symmetric NAT as two different
`C(X)` nodes will disagree in port number in such cases.

On the other hand, if X is behind a Symmetric NAT, then contacting
multiple `C(X)`s would disable port prediction.

# Implementation details

```rust
/// Generate a new number each time this function is called, the new
/// number shall allways be (1+previous_number) and the first
/// number shall be generated by random. Once the maximum number is
/// reached, next number will be 0.
fn State::generate_request_id(&mut self: State) -> u32

/// Returns a vector of SocketAddrs pointing to U(C(X)),
/// the vector should be sorted so that addresses that
/// are not on our LAN are first (use `getifaddr` here).
fn sort_helping_nodes_by_preference() -> Vec<SocketAddr>

struct PeriodicSender {
  udp_socket: UdpSocket,
  payload: Vec<u8>,
  times_sent: u32,
  // Specifies how many times the payload should be sent.
  // If set to None, sends indefinitely.
  times_to_send: Option<u32>,
}

impl PeriodicSender {
  /// Starts periodicaly sending `what` `times_to_send` times.
  fn start<T: Serializable>(&mut self, where: SockAddr, times_to_send: Option<u32>, what: T);

  fn stop();

  /// Resets the payload, `times_sent` will remain unchainged.
  fn reset_payload<T: Serializable>(&mut self, what: T);

  /// Wait til times_sent == times_to_send.unwrap()
  /// (panic if times_to_send == None and this function is called)
  fn block_until_finished(&self);
}

/// Stop sending when the sender is dropped.
impl Drop for PeriodicSender;

fn blocking_get_mapped_udp_socket(request_id: u32, helper_nodes: Vec<SocketAddr>)
    -> Result<(UdpSocket, Result<SocketAddr>)>
  const TIMES_TO_SEND = 5;
  let udp_socket = UdpSocket::bind("0.0.0.0:0");
  let periodic_sender = PeriodicSender::new(udp_socket);

  for cx in helper_nodes {
    periodic_sender.start(cx, TIMES_TO_SEND, GetExtAddr::new(request_id));

    loop {
      let our_ext_address = match udp_socket.timed_blocking_read(2000ms) {
        Err(_) => {
          break;
        },
        Ok(datagram) => {
          if datagram.request_id != request { continue }
          if datagram.from != cx { continue }
          datagram.ext_address
        }
      }
      return (udp_socket, Ok(our_ext_address));
    }
  }
  (udp_socket, Err(...))
}

// The non blocking version that users of the Crust library will use.
pub fn Service::get_mapped_udp_socket(&self, result_token: u32) {
  send_job_to_state_thread(move |state| {
    let request_id = generate_request_id();
    let event_sender = state.event_sender.clone();
    let helpers = self.sort_helping_nodes_by_preference();

    thread::spawn(move || {
      let (socket, mapped_address) = blocking_get_mapped_udp_socket(request_id, helpers);
      event_sender.send(Event::OnUdpSocketMapped(result_token, socket, mapped_address));
    });
  });
}

fn blocking_udp_punch_hole(request_id: u32, // Note: this is not the result token
                           udp_socket : UdpSocket,
                           secret: Option<[u8; 4]>,
                           peer_addr : mut SocketAddr /* of node B */)
      -> (UdpSocket, Result<SocketAddr> /* peer's address */) {
  const TIMES_TO_SEND = 5;
  let mut received = false;
  let mut periodic_sender = PeriodicSender(udp_socket);
  periodic_sender.start(peer_addr,
                        TIMES_TO_SEND,
                        HolePunch::new(request_id, secret, received));

  loop {
    match udp_socket.timed_blocking_read(2000ms) {
      Ok(datagram) => {
        if datagram.request_id != request_id { continue }
        if datagram.secret != secret { continue }

        received = true;

        if datagram.ack {
          // He received our packet and we received his, we're done.
          return (udp_socket, Ok(datagram.from));
        }

        if datagram.from != peer_addr {
          peer_addr = datagram.from;
          periodic_sender.start(peer_addr,
                                TIMES_TO_SEND,
                                HolePunch::new(request_id, secret, received));
        }
        else {
          // New payload with `received` set to true.
          periodic_sender.reset_payload(HolePunch::new(request_id, secret, received));
        }
 
        periodic_sender.block_until_finished();
        break;
      },
      Err(what) => {
        return (udp_socket, Err(what));
      },
    }
  }

  (udp_socket, Ok(peer_addr))
}

// The non blocking version that users of the Crust library will use.
pub fn Service::udp_punch_hole(&self,
                               result_token: u32,
                               udp_socket: UdpSocket,
                               secret: Option<[u8,4]>,
                               peer_addr: mut SocketAddr /* of  node B */) {
  send_job_to_state_thread(move |state| {
    let request_id = generate_request_id();
    let event_sender = state.event_sender.clone();

    thread::spawn(move || {
      let (socket, peer_addr) = blocking_udp_punch_hole(request_id,
                                                        udp_socket,
                                                        secret,
                                                        peer_addr);
      event_sender.send(Event::OnHolePunched(HolePunchResult::new(result_token,
                                                                  udp_socke,
                                                                  peer_addr)));
    });
  });
}

```
