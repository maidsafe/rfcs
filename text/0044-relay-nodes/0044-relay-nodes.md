# Secure Vault Joining

- Status: proposed
- Type: enhancement
- Related components: safe_vault, routing
- Start Date: 03-09-2016
- Discussion: https://forum.safedev.org/t/rfc-44-secure-vault-joining/119
- Supersedes: https://github.com/maidsafe/rfcs/blob/master/text/0030-secure-node-join/0030-nodes_key_as_name.md

## Summary

This RFC will propose a method to enforce a joining "cost" for new or restarting vaults to join the
network. This is a mechanism to prevent mass joining quickly and therefore will introduce a cost of
such an attack that is proportional to the network "effort" over time.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT",
  "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC
  2119](http://tools.ietf.org/html/rfc2119).

**Effort** - An amount of work performed.

## Motivation

SAFE has already taken advantage of securing groups and group consensus. Also, with disjoint groups,
secures hops which prevents nodes masquerading as network identities. It still though has a weakness
in a possible attack where an attacker can easily restart many nodes constantly to eventually target
a network location. The difficulty of such an attack is not considered here, merely the opportunity
of such an attack exists in some form.

This RFC allows the network to ensure that joining nodes are tested prior to joining a group. As a
node is promoted then it can begin to acquire rank. This RFC does not cover rank or relocation.

Another important feature of This RFC is that the

## Detailed design

The "effort" of the network should allow that a proportionate amount of effort is carried out by a
node wishing to join. In a balanced network the whole network effort is directly proportional to any
group effort at. We do not consider the network fully balanced but do assume there is a large enough
population and the hashing algorithm does indeed provide a suitable balanced network.

This distinction requires a measurement of such work as well as a task that is advantageous to the
network. One such task that is currently under-rewarded is providing the access points (relays and
bootstrap nodes) for client or joining nodes.  This function is onerous on nodes in the network, but
at the same time it is critical.  When a node is acting as a `RelayNode` then the node it is
relaying for depends on that `RelayNode` to route messages back. This is a problem if there is a
single relay, however not so much if the relay is in a group or at least connected to several
closest nodes. If the relays are disparate though then the network traffic is amplified. It makes
sense the relays are all in the same group. If a client is waiting on a response it makes sense the
response returns without a single relay having to persist.

**For security and anonymity a client connection to the network should not use a key more than once,
so a client must create a keypair every time and throw them away at the end of each session. **


### `RelayNode` overview

A `RelayNode` has a single function: To relay messages to and from clients.

These nodes will be a fundamental type `RelayNode` This node will not be considered in group refresh
messages, or any messages related to data (i.e. these are **not** routing table nodes), but it would
have messages routed back to it (responses).

Of course any node can also provide these resources in times of need. The difference is though, that
a `ManagedNode` joins via the security of the network and can be added to the routing table,
regardless of providing this service. A node that joins as a `RelayNode` is not added to the routing
table.

Nodes acting as `RelayNodes` have a recognisable address type on the network. This requires that
such nodes are already network connected and located in a group, but importantly do **not** require
to be in the routing table of group members. It is suggested here that there will be a limited
number of such nodes in any (non full) group. The number of these nodes should be tested during
implementation testing (initially restricted to < 50% of group size>). Additional attempts to
connect will be rejected in this group. This follows the joining limit already used and tested in
the network testnets.

#### Flow diagram
```
|----Node----|----Group X----|------Group Y------|------Group z------|
|
|Createkey -> Create Y and
|             Contact Y to
|             confirm Range
|             of new key.
|                          -> Accept N if
|                             # of `RelayNode`
|                             < 50% of group
|                            **N is now a
|                              `RelayNode`**
|                            Not in routing table
|                            ----------------
|                            Group keeps count of
|                            `Put` responses per
|                            `RelayNode` (sync)
|                            -----------------
|                            On group create
|                            safecoin the top
|                            `RelayNode` is
|                            promoted to Z
|                            (hash of safecoin
|                             addr) as per
|                            joining Y
|                                                   Node is now `ManagedNode`
|                                                   Will require to grow in rank
```

#### Relayed connections


### `RelayNode`

On receipt of a `Get` request a DataManager will trigger safecoin checks. These checks use a
balancing algorithm to calculate a modulo that will be tested against the `Get` request and if
successful a safecoin is awarded. A group in any part of the network will earn / farm safecoins at
approximately the same rate. All group members are involved in the creation of a safecoin.

As the network requires new nodes at whichever rate the algorithm has calculated at any point in
time a `RelayNode` can be promoted to a `ManagedNode`. This will require the node is located to a
new group (Z).

#### `RelayNode` connections

A `RelayNode` will connect to all group members in the current close group only. Relay response
messages will be relayed through the relay node in the address, if the node is still present,
otherwise it is relayed via the first available `RelayNode`. This process ensures clients should
always receive responses, even when the relay used is no longer available.

#### `RelayNode` monitoring

All group members are aware of all `RelayNode`s as they are connected to the group. If a `RelayNode`
leaves the group all clients connected will instantly be aware of this. If a new `RelayNode` joins
the group then all group members send this `RelayNode` address to clients to allow the client to
accumulate this address and make a connection,

#### Selection of the `RelayNode` to promote

Several `RelayNode`s will exist in any group and vie for promotion. Some may be malicious or unable
to handle traffic for clients. When a client recognises responses are not coming back from a
`RelayNode` or are to slow then the client will stop routing via that node. This will mean the
responses to the client will not come via that node either. Thus the number of client relay
responses through a group allows the group to measure the `RelayNode`s value to the network.

Counting and refreshing each response throughout the group is a lot of traffic and potentially open
to abuse. However, `Put` responses mean a client has at least paid safecoin to make the request.
These requests are therefore more indicative of a measure of work, beyond free network requests.
This RFC proposes that `Put` responses are counted per `RelayNode`.

The number of responses per `RelayNode` can still be gamed, but at a cost. The node with most
responses addressed via itself is the `RelayNode` that will be promoted to group Z and `ManagedNode`
status. From there it will require to build rank, this is another RFC and deemed outwith the scope
of this RFC.

As the group are counting what will be valuable information then this is not wasteful. If it were
calculating info to prove a node is not valid then it would be potentially useless.


#### `RelayNode` promotion

The group that has now created a safecoin will send a `FullNodeJoin` to the group that is closest to
the hash of the safecoin farmed. This allows the group to agree on a random piece of known
information. This group will reply as in the normal node join process as specified in
[RFC-30](https://github.com/maidsafe/rfcs/blob/master/text/0030-secure-node-join/0030-nodes_key_as_name.md)

A node joins the network

A node (A) creates a keypair and connects to a group (X).  Group (X) then take the 2 closest nodes
to this and hash the three values (2 closest plus node).  The resultant hash of the 2 closest nodes
yields the target group for the node to join (Y).

After the group (X) mint a safecoin then

Group (X) then sends a JoinRequest (Y).  Group (Y) then calculates the furthest two nodes apart in
group (Y) Group (Y) then respond to (A) with JoinResponse with the middle third of the two furthest
apart nodes as the target range. (JoinResponse includes all Y members names).  Group (Y) will set a
pending join request for a period of 30 seconds or so.  This group will not accept any more
JoinRequest during this time.  Node (A) then must create an address that falls between these two
nodes and then join (Y).

When the group creates a safecoin a node can be selected for promotion. All existing counts are
zero'd in this case and the process resumes.

####Client

A client will bootstrap and then join a group exactly the same way a node does. When in this group it
will connect to at least the `RelayNodes` nodes in the group and send requests through these at random.
On losing any node the client will re-establish connection to the groups `RelayNode`s again.

This requires a new RPC, `Get_relays()` that the client can send to group X. Nodes will **not** send
IP information of `ManagedNode`s to any client.



## Drawbacks

* Users vaults will now take longer to become full nodes and therefore will see an increased delay
* in the time to farm safecoin.  When a node requests membership of the network the intial group
* will require to confirm a joining token (`Get`) and send the delete on to that address. This could
* cause churn issues and requires discussed in detail.

## Alternatives

* Forced random relocation of nodes (many variants).  Nodes performing a proof of work type
* algorithm.  Node ranking and relocation rules.

* Clients join groups as non routing table nodes. Use a random Id to do so. Still send Put requests
* through `MaidManagers`. They need not connect to every group member, but  should at least connect
* to the closest 2 members.

* `RelayNode`s join a group as non routing table Nodes and act as `Bootstrap` nodes and
* `RelayNodes`.

* `RelayNode`s earn tokens from group members who agree the modulo of their address on any
* `GetResponse` is 0. This modulo number is calculated as per safecoin reward and uses the same
* algorithm. As the network requires resources the rewards are more frequent and as the network
* decides it has enough resources these nodes will take longer to earn a token. For this reason
* these nodes must connect to every group member that is a routing table node.

## Future work

* joining node tasks are expected to evolve to perhaps use more sophisticated measurements of
  ability.

## Unresolved questions

* The bootstrap process requires further clarification and should remain seperate form this process.
