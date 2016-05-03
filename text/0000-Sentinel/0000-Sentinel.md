- Feature Name: Sentinel
- Status: rejected
- Type: Security component
- Related components: routing
- Start Date: 30-06-2015
- RFC PR:
- Issue number:

# Summary

This document tries to explain basic principles behind security
on the SAFE network.

# Motivation

Big part of algorihms behind the SAFE network rely on so called local
group consensus. What it basically means is that for a node to act in some
way, it needs to be instructed to do so by a quorum of other nodes.
The set of these nodes is determined by the network, but in general
they are expected to be as close as possible to a certain HASH address
within the XOR network.

We generally call such a set a Group(H) where H is the HASH address or
simply a Group when H is known from the context.

The key idea is the assumption that it is cryptographically hard for an
adversary to enter an arbitrary group of close nodes. The purpose of the
Sentinel is then to ensure that the set of these instructing
nodes really lives in a Group surrounding the given HASH.

# Detailed design

Currently we have a need for three kinds of sentinels:

* __PureSentinel__: This sentinel is dedicated to validate `Put` and `Post` messages
which arrived from a `distant group` (i.e. a group we're not part of).

* __KeySentinel__: This one is in a sense very similar to the `PureSentinel`
but instead of acting on behalf of `Put` and `Post` messages originating
from arbitrary locations (plural because it originates from a group) in the
network it acts on behalf of `FindGroupResponse` messages where the
`FindGroup` request message originated from us. This subtle distinction
allows us to request fewer network send calls as shall be explained later.

* __AccountSentinel__: This one validates `Refresh` messages which arrived from
our `close group` (i.e. a group we're part of).

## PureSentinel

Let's consider a situation where a group G sends us a `Put` message. Since
this is a group message we expect to receive many copies of it. And since we want
a quorum of such messages, sentinel must start storing them while identifying
which nodes sent it.

At the same time (when first of a kind message arrives), this sentinel needs
to send a `GetGroupKey` message to the group where this message originated
to trigger yet another group message from G to us.

This newly triggered message is of type `GetGroupResponse` and is
isomorphic to a list of (NameType, PublicKey) pairs. Once
each such NameType is confirmed to have the same PublicKey by
a quorum number of different sources, we declare such NameType as
`validated` and can be used to validate the original message.

A __message__ is declared `validated` once it arrives from at least
a quorum number of __validated sources__. Once a message is validated
sentinel releases it back to Routing for further processing.

Note that it might be tempting to avoid the step where we send
the `GetKey` message and consider sending the public keys as
part of the original PUT message to decrease network traffic,
it turns out that this step is essential because this way it
is the network that decides who belongs to G (by means of the
parallel send implemented by routing).

## KeySentinel

This sentinel is intended for use when a node wants to find
a group of nodes at some location. This normally hapens during
startup before it is connected to anyone.

The sequesnce of steps is as follows:

1. We send a `FindGroup` message to the group around some location
2. When the nodes in the group receive this message they respond
   with a `FindGroupResponse` which is roughly isomorphic to a list
   of (NameType, PublicKey, _additional data_) tuples.

Notice the analogy with the previous sentinel where the step (1)
would correspond to sending the `GetGroupKey` message and step (2)
would correspond to receiving the group keys. The difference is
that the `FindGroupResponse` message contains additional information
apart from the NameType and the PublicKey which is then used
by the Routing library. The rest of the validation procedure
is analoguous as well.

## AccountSentinel

This one is simplest of the three. It is because we're only expecting
messages to go through this sentinel if they arrive from our own
close group. As such, it is assumed that we already know public keys
of the senders so there is no need to explicitly ask for them.

Having said that, the responsibility of this sentinel reduces to
accumulating messages by a certain key and once a quorum of messages
with the same key is reached, they are gathered in a list and returned
for further processing by the Routing library.

# Drawbacks

The Pure and Key sentinels request public keys of group members, this
crates additional traffic.

# Alternatives

Sentinels currently validate nodes by requiring that at least
a quorum size of other nodes confirm their public key. It is unclear
yet whether this condition is sufficient. A stronger requirement
might be that if a node A confirms identity of a node B, then
node B also needs to confirm identity of A. Note that this confirmation
needs not be direct as confirmation can be assumed transitive. I.e.
if A confirms C and C confirms B then that means A confirms B.

If we did decide this stronger requirement is essential, the
algorighm would reduce to finding loops in a graph.

# Unresolved questions

All three types of sentinels currently live in the `Sentinel` library.
In order to achieve genericity of such library it avoids using
types internal to the Routing library such as `Put`, `Post`, `GetKey`...
This in turn requires more code in both libraries and reduces expresiveness
of the code using it. As it is debatable whether these sentinels
have use in other libraries apart from `Routing` it might be
desirable to make it part of the `Routing` library.
