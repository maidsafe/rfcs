# Node Ageing

- Status: proposed
- Type: new feature
- Related components: routing
- Start Date: 25-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)

## Summary

The balance between easily starting a node and enforcing security against group targeting requires a
mechanism to both constantly dilute any attacker across the network and also ensure such nodes are
behaving (providing value). This RFC outlines a mechanism to relocate nodes throughout each on-line
session.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT",
  "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC
  2119](http://tools.ietf.org/html/rfc2119).

## Motivation

The network must secure against targeting groups. Earlier RFCs such as [disjoint
groups](https://github.com/maidsafe/rfcs/blob/master/text/0037-disjoint-groups/0037-disjoint-groups.md) and [node key as
name](https://github.com/maidsafe/rfcs/blob/master/text/0030-secure-node-join/0030-nodes_key_as_name.md)
as well as data chains and locked data chains (linked below) have solutions for improved group
security, nodes faking group membership, this RFC specifically focusses on targeting groups.

This RFC will ensure that nodes are relocated and therefore dilute their impact on the network
across the address range.

## Expected outcome

The outcome will be that an attacker or large participant in the network has an amount of work that
must be carried out that is prohibitively expensive.

## Detailed design

### Pre-requisites

This RFC requires that nodes that are not "well behaved" will be excluded from the network.
Additionally it is assumed that data chains are
[implemented](https://github.com/maidsafe/rfcs/blob/master/text/0029-data-chains.md/0029-data-chains.md) and
optionally (but preferred)  [secured from key reuse](https://github.com/dirvine/rfcs/blob/0041-locked-data-chains/0041-locked-data-chains/0041-locked-data-chains.md).

Nodes will be continually tested that they
  - Forward messages
  - Continually send `NodeBlock`s for churn events (any missing two consecutive churn events are
    considered invalid nodes).
  - Groups will **not** accept connections from the same IP address.
  - Groups will only allow one node with an age of zero.

### Definitions

G : Group size

N : network population

Z : Number of groups (N/G)

Q : Quorum

A : Age of node

### Overview

Without any node relocation an attacker with a single join will enter a group and therefore requires
(Q - 1) nodes To take over the group completely. An attack will therefore require the attacker try
(Q - 1) * Z  times. If an attacker could run X% of the network population then this would mean the
attack would require starting the nodes in parallel X times. Therefore even if nodes had a time to
wait prior to joining this would only delay the attack by that time.

A simple way to think of this is that if you start a node it will join a group A. If you then start
another node it will join a random group (1/number of groups). If this happens quickly then it is
easy to 1: test if the group you join is A and 2: if not then simply restart. In this scenario a
node will simply restart until it's group == A. Repeating this process will quickly gain control of
a group.

This proposal relocates nodes using an exponentially increasing period between such relocations.
This prevents the above attack as the nodes joining group A are moved to new groups with increasing
periods of effort in between relocations. In addition and very importantly the nodes gain "age" as
they behave and provide value to the network as time passes.

Doing so is a natural design, this design can be found in many aspects of the natural world,
whereby a new member of a group, whether a stranger or an infant, gains trust over time as they prove
themselves to the community. This should allow the implementation to be easily understood and the goals
to be modelled with some certainty as the pattern should be a familiar one.

To achieve this a node will be allocated a "Age" starting at 0. On each relocation this age (A)
will be incremented. Here we increment the age by 1 on each relocation, but the work required in
each relocation is exponential. This means with age 2 there requires twice as much work, age 3
requires twice as much again.

Effort required == 2^age in each group the node is moved to. If a receiving group refuses the
relocation then the node will be relocated on next churn event.

### Consensus measurement

A group consensus will require >=50% of nodes and 50% of the age of the whole group.

### Transmitting age to network

Nodes throughout the network will have to be able to trust a node's age in any group communications.
To achieve this the node id requires to carry this information in a manner that does not allow the
node to fake an age or address. This is described [here](#network-name-redefined).

### Network name redefined

As an addition to using the PublicKey as the name, this RFC  will benefit greatly from adding a
single byte onto the name. This u8 will allow us to gradually promote nodes 254 times, which is
likely beyond any reasonable possibility (2^254 is likely beyond the number of churn events any node
will be involved in).

```rust

struct NodeName {
  key: sign::PublicKey,
  age: u8
}

impl NodeName {
  name(&self) -> [u8;32]
       hash(age: u8, key: PublickKey)
  }
}

```

The `NodeName.name` is the nodes address on the network. We can then confirm this `name` is actually
a name that corresponds to the group in question. This NodeName is also stored in the data chain to
allow further validation of a nodes group membership.

This struct adds a single byte to the address and requires that byte to be transmitted as part of
any message the node sends, as is the current mode of operations (nodes send their Id in every
message and this allows the messages to self-validate).

### Starting a node

A node will join a start group X with an Id it creates. Group X will age this node at 0. As the
node has zero age it has zero impact on any consensus age requirement. **Group X will now not accept
any more new nodes**.

### Selecting a node to relocate

As groups churn they agree on a new data chain "Link" this block of the chain is named using the
hash of all nodes in the group at that time. This provides a pseudo random piece of data that the
group can agree on. The group checks A and the number of churn events so far, if 2^A churn events
have occurred the node will be relocated.

A further check of at least a single data block existing between any churn events is required. This
ensures that in times of massive network churn (network collapse or partition) this process will not
create more churn by relocating nodes. Therefore any two consecutive churn events with no data
manipulation between them will not be counted.

If there exists more than one node to relocate then the oldest node should be chosen. If two nodes
have the same oldest age then the one that has been present for the most churn events is chosen to
relocate.

### A node joining proof

On joining a group a node will require to prove capablity. This is on every join attempt to a group
and the proof must be sent directly to each node as a connection is made. The steps to create this
proof are:

1. Concatenate the key create to join the group 32768 times to create a chunk of ~1Mb in size.

2. Increment an integer value to the end of this message until the sha3 of the message has 5 leading
zero's (a proof of work similar to [hashcash](https://en.wikipedia.org/wiki/Hashcash)). A simple
script demonstrates this process with sha256 ``time (perl -e '$n++ while`echo "A Public
key$n"|sha256sum`!~/^00000/;print$n')``

### Joining the network for the first time

A node must create a key to join the network and send a `JoinRequest` message to that group. On
connecting to each group member the node must supply the `JoiningProof` as described above. AS there
is no history for this node it will be allocated an age == zero. This means the node itself on
successful join will be immediately relocated.

### Relocating a node

When a churn even does force a node to relocate to a destination group (D) from this source group
(S) then the hash that is the link block is combined with the nodes existing name and hashed to
find D. All members of S then send the nodes age incremented (++A) to D with the nodes current
PublicKey (Id).

Members of D, accumulate this message and store the node Id and age in a pending connections for a
period of 10 minutes.

The relocating node then generates a key that will fit in D and attempts to join D.

This node must then

1. Create a `JoinProof` for D

2. The current group (S) sends the join request with this nodes current ID + age incremented by 1.
__if this is a new node then it must Send a join request to the joining group (via bootstrap as it
is now), the receiving group will set the age at 0 for this node and relocate it on the first churn
event (this join will create that churn event, effectively meaning immediate relocate).__

3. This node then makes direct connections to each member of group (D) and then Sends this
`proof` to each node in the joining group on connect to confirm ability to compute and transfer
data. Each member of the joining group will send the `NodeBlock` for the new Link when this is
received. When the link validates this node is added to the routing table.

If a connect attempt as a node is made to the group the pending connections container is queried for
the old node Id (that signed the message). If the node "fits" in the group then it can try to join,
otherwise a message is sent back to the node refusing connection. On receipt of these messages the
node would be able to refine the generated key to "fit" into D. The node will retry the connection
in that time.

If a node cannot join in time then it will require to start on the network again at age 0.

### Subsequently joining the network

When a node recieves an identity it should store this locally. On restart the node will join the
network at it's last known group. This group will accept the node, but with an age of 0. The group
members will request any data the node has and use the agreed churn event that the node created by
joining.

As each node is satisfied that it recieved the data requested then it sends a `JoinRequest` to D on
behalf of this node with the nodes existing correct **age/2** (the age in the datachain divided by
2).  This node then joins D as per normal. D accumulates the `JoinRequest`s therefore if the node
attempts to not provide the requested data it risks non accumulation and therefore having to start
again from an actual age of 0.

The reason for the node restarting with age 0 is to prevent off line agreement / sale of ID's, which
is unlikely but possible. The age divinding by 2 prevents malicious restarts to target a group. This
relocation type attack is thwarted by a very fast reduction in nodes age.

### Limits on relocating or refusing a node

**Whilst the group size is less than `G` a node will not be relocated or refused joining. If
relocating a node reduces the group to less than `G` it will not be relocated.** These steps are
necessary to maintain groups and also to allow the network to start.

### Measuring a node

As a node forwards messages to other groups that group can confirm a message was sent or not. As
data is relayed via group messages (such as Put) then the group selects the node to forward the
message based on it's `closeness` to the name of the data. This process distributes the workload of
forwarding messages throughout the group. A nice feature is the receiving group can be trusted and
if they receive the message from every member of this group which node is sending the data they can
also confirm the data never arrived. Thereby they will send back a resend request to the group with
the name of the node that did not send the data in time. This node can then be disconnected or have
it's age reduced (initially disconnecting will probably work best).

If a node does not send `NodeBlock`s then it potentially damages the group integrity. Any node that
has not sent a link `NodeBlock` since the last churn event should be disconnected, if and only if,
there are data `Nodeblock`s transmitted between churn events (to prevent relocate when churn is high).

If a node misses X `NodeBlock`s sequentially then it should be disconnected. Initially this may be
set at `G/2`.

This process is a small part of a nodes `work` on the network, a very nice measure would be delivery
of data to the client. This is more complex as we do not trust client nodes. This is an area of
further investigation though (see below).

### Quorum redefined

A quorum can now be defined as 50% of the groups total age score. In addition a minimum number of
participants should take part (the new Quorum). This may be set to a low number such as 25% of the
group size or similar. It is proposed in this RFC the quorum is set at >= 50% of the group members
plus 50% of the group total age.


## Future considerations

Groups may only allow one node of each age per group. This further distributes age through the
network. This requires further modelling.

Further measurements of a node would certainly ensure a more robust network, one such measure is
mentioned previously, delivery of data to a client. There is a possibility zk-snark technology could
help here to ensure the node sent data via a network channel and did execute that set of
instructions on the cpu. The group can then ask for proof of sending if a client did accuse a node
of non delivery. The client itself can send a snark to prove it waited on the data, perhaps.

Blacklisting of bad nodes IP address may be considered, although this may be a problem in that it
can exclude full networks (i.e. via vpn's or NAT traversal etc.)

Again zk-snarks may be interesting here, if a node is required to query it's own cpu/mac address then
these can be blacklisted.

## Drawbacks

This does require,in it's current form, data chains RFC's are in place and these still require
deeper investigation and discussion.

In it's current form this RFC suggests a brutal binary penalty for a misbehaving node with no scope
for error. This is likely too brutal and will cause unnecessary traffic as an unintended
consequence. For this reason significant testing should be carried out to perhaps reduce the
strict requirement of handling every message.

## Alternatives

- Relocate each node at some interval without age.

## Unresolved questions

The maths model of age based relocation is, as yet, incomplete, although it's very difficult to imagine
this does not significantly increase security, whilst also allowing nodes to accrue an age that
allows them to store significant amounts of data (archive nodes).
