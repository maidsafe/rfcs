# Relocate Nodes

- Status: proposed proposed
- Type: new feature
- Related components: routing
- Start Date: 25-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

## Summary

The balance between easily starting a node and enforcing security against group targeting requires
a mechanism to constantly dilute any attacker across the network. This RFC outlines a mechanism to
relocate nodes throughout each on-line session.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT",
  "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC
  2119](http://tools.ietf.org/html/rfc2119).

## Motivation

The network must secure against targeting groups. Earlier RFC's have solutions for group security,
nodes faking group membership, but this RFC specifically focusses on targeting groups, via such
mechanisms such as repeatedly starting nodes, even millions of times to target a group.

This RFC will ensure that nodes are relocated and therefore dilute their impact on the network
across the address range.

## Exepcted outcome

The outcome will be that an attacker or large participant in the network has an amount of work that
must be carried out that is prohibitively expensive.

## Detailed design

### Pre-requisites

This RFC requires that nodes that are not "well behaved" will be excluded from the network.
Additionally it is assumed that data chains are implemented and secured.
Nodes will be continually tested that they
  - Forward messages
  - Continually send `NodeBlock`s for churn events (any missing two consecutive churn events are
    considered invalid nodes).
  - Groups will ***not** accept connections from the same IP address.

### Definitions

G : Group size
N : network population
Z : Number of groups (N/G)
Q : Quorum
A : Attacker % of network nodes
R : Rank of node

### Overview

Without any node relocation an attacker with a single join will enter a group and therefore requires
(Q - 1) nodes To take over the group completely. An attacker will therefore require the attacker try
(Q - 1) * Z  times. If an attacker could run X% of the network population then this would mean the
attack would require starting the nodes in parallel X times. Therefore even if nodes had a time to
wait prior to joining this would only delay the attack by that time.

This proposal relocates nodes using an exponentially increasing period between such relocations.

To achieve this a node will be allocated a "Rank" starting at 0. On each relocation this rank (R)
will be incremented.

### Starting a node

A node will join a start group X with an Id it creates. Group X will rank this node at 0. As the
node has zero rank it has zero impact on any vote (it will not be counted in any consensus).

### Selecting a node to relocate

As groups churn they agree on a new data chain "Link" this block of the chain is named using the
hash of all nodes in the group at that time. This provides a pseudo random piece of data that the
group can agree on. The group checks R and the number of churn events so far, if 2^R churn events
have occurred the node will be relocated.

A further check of at least a single data block existing between any churn events is required. This
ensures that in times of massive network churn (network collapse or partition) this process will not
create more churn by relocating nodes. Therefore any two consecutive churn events with no data
manipulation between them will not be counted.

### Relocating a node

When a churn even does force a node to relocate to a destination group (D) from this source group
(S) then the hash that is the link block is combined with the nodes existing address and hashed to
find D. All members of S then send the nodes rank incremented (++R) to D with the nodes current
PublicKey (Id) (see below for redefinition of network name).

Members of D, accumulate this message and store the node Id and rank in a pending connections for a
period of 10 minutes.

The relocating node then generates a key that will fit in D and attempts to join D.

If a join attempt is made to the group the pending connections container is queried for the old node
Id (that signed the message). If the node "fits" in the group then it is accepted, otherwise a
message is sent back to the node refusing connection. On receipt of these messages the node would be
able to refine the generated key to "fit" into D. The node will retry the connection in that time.

If a node cannot join in time then it will require to start on the network again at rank 0.

### Quorum redefined

A quorum of nodes will be defined as reaching above 50% of the total rank of any group. To achive
this intra group is simple as all nodes in a group "know" the rank of each node.

Inter group messages though will require that this rank be transmitted therefor it pushes the design
towards redefining the network name, once more.

A quorum can now be defined as 50% of the groups total rank score. In addition a minimum number fo
participents should take part (the new Quorum). This may be set to a low number such as 25% of the
group size or similar. It is proposed in this RFC the quorum is maintained at > 50% of the `G`.

### Network name redefined

As an addition to using the PublicKey as the name, this RFC  will benefit greatly from adding a
single byte onto the name. This u8 will allow us to gradually promote nodes 254 times, which is
likely beyond any reasonable possability (2^254 is likely beyond the number of churn events any node
will be involved in).

```rust

struct NodeName {
  key: PublicKey,
  rank: u8
}

```

## Drawbacks

Why should we *not* do this?

## Alternatives

What other designs have been considered? What is the impact of not doing this?

## Unresolved questions

What parts of the design are still to be done?
