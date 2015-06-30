- Feature Name: Sentinel
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
simply A Group when H is known from the context.

The key idea is the assumption that it is cryptographically hard for an
adversary to enter an arbitrary group of close nodes. The purpose of the
Sentinel is then to ensure that the set of these instructing
nodes really lives in a Group surrounding the given HASH.

# Detailed design

This is the bulk of the RFC. Explain the design in enough detail for somebody familiar
with the network to understand, and for somebody familiar with the code practices to implement.
This should get into specifics and corner-cases, and include examples of how the feature is used.

Currently we have a need for three kinds of sentinels:

PureSentinel: This sentinel is dedicated to validate `Put` and `Post` messages
which arrived from a `distant group` (i.e. a group we're not part of).

KeySentinel: This one is in a sense very similar to the `PureSentinel`
but instead of acting on behalf of `Put` and `Post` messages originating
from arbitrary locations (plural because it originates from a group) in the
network it acts on behalf of `FindGroupResponse` messages where the
`FindGroup` request message originated from us. This subtle distinction
allows us to request fewer network send calls as shall be explained later.

AccountSentinel: This one validates `Refresh` messages which arrived from
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
TODO

## AccountSentinel
TODO

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

