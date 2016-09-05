# Secure vault joining

- Status: proposed
- Type: enhancement
- Related components: (vault/routing)
- Start Date: 03-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: None
- Superseded by: N/A

## Summary

This RFC will propose a method to enforce a joining "cost" for new or restarting vaults to join the network. This is a mechanism to prevent mass joining quickly and therefore will introduce a cost of such an attack that is proportional to the network "effort" over time.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

**Effort** - An amount of work performed.


## Motivation

SAFE has already taken advantage of securing groups and group consensus. Also, with disjoint groups, secures hops which prevents nodes masquerading as network identities. It still though has a weakness in a possible attack where an attacker can easily restart many nodes constantly to eventually target a network location. The difficulty of such an attack is not considered here, merely the opportunity of such an attack exists in some form.

This RFC prevents this attack and claims that this will provide an overall network security level that would be appropriate for securing group creation and thereby groups holding even the most valuable data such as digital currencies.

## Detailed design

The "effort" of the network should allow that a proportionate amount of effort is carried out by a node wishing to join. In a balanced network the whole network effort is directly proportional to any group effort at. We do not consider the network fully balanced but do assume there is a large enough population and the hashing algorithm does indeed provide a suitable balanced network.

This distinction requires a measurement of such work as well as a task that is advantageous to the network. One such task that is currently under-rewarded is providing the access points (relays and bootstrap nodes) for client or joining nodes.  This function is onerous on nodes in the network, but at the same time it is critical.  When a node is acting as a `RelayNode` then the node it is relaying for depends on that `RelayNode` to route messages back. This is a problem if there is a single relay, however not so much if the relay is in a group or at least connected to several closest nodes. If the relays are disparate though then the network traffic is amplified. It makes sense the relays are all in the same group. If a client is waiting on a response it makes sense the response returns without a single relay having to persist. For these reasons clients should join groups, but not routing tables.

**For security and anonymity a client connection to the network should not use a key more than once, so a client must create a keypair every time and throw them away at the end of each session. **

### Network authorities

These would change from

```
pub enum Authority {
    /// Manager of a Client.  XorName is the hash of the Client's `client_key`.
    ClientManager(XorName),
    /// Manager of a network-addressable element.  XorName is the name of the element in question.
    NaeManager(XorName),
    /// Manager of a ManagedNode.  XorName is that of the ManagedNode.
    NodeManager(XorName),
    /// A non-client node (i.e. a vault) which is managed by NodeManagers.  XorName is provided
    /// by the network relocation process immediately after bootstrapping.
    ManagedNode(XorName),
    /// A Client.
    Client {
        /// The client's public signing key.  The hash of this specifies the location of the Client
        /// in the network address space.
        client_key: sign::PublicKey,
        /// The Crust peer ID of the client.
        peer_id: PeerId,
        /// The name of the single ManagedNode which the Client connects to and proxies all messages
        /// through.
        proxy_node_name: XorName,
    },
}
```
To

```
pub enum Authority {
    /// Manager of a Client.  XorName is the hash of the Client's `client_key`.
    ClientManager(XorName),
    /// Manager of a network-addressable element.  XorName is the name of the element in question.
    NaeManager(XorName),
    /// Manager of a ManagedNode.  XorName is that of the ManagedNode.
    NodeManager(XorName),
    /// A non-client node (i.e. a vault) which is managed by NodeManagers.  XorName is provided
    /// by the network relocation process immediately after bootstrapping.
    ManagedNode(XorName),
    /// A Client. Name is hash of PublicKey
    Client (XorName),
    /// This node address, relayed for address
    RelayNode(XorName, XorName)
    },
}
```

### `RelayNode` overview

A `RelayNode` has two functions:

1. Allow "not yet connected" nodes to establish network communications (bootstrap).
2. Relay messages to a non routing table node (relayed connections).

These nodes will be a fundamental type `RelayNode` This node will not be considered in group refresh messages, or any messages related to data (i.e. these are **not** routing table nodes), but it would have messages routed back to it (responses).

Of course any node can also provide these resources in times of need. The difference is though, that a `ManagedNode` joins via the security of the network and can be added to the routing table, regardless of providing this service. A node that joins as a `RelayNode` is not added to the routing table.

Nodes acting as `RelayNodes` have a recognisable address type on the network. This requires that such nodes are already network connected and located in a group, but importantly do **not** require to be in the routing table of group members. It is suggested here that there will be a limited number of such nodes in any (non full) group. The number of these nodes should be tested during implementation testing (initially restricted to 1). Additional attempts to connect will be rejected in this group. This follows the joining limit already used and tested in the network testnets.


#### Bootstrap connections

When a node selects a node to `bootstrap` from then this node will route requests and responses to and from the network.

A bootstrap connection will be limited to only provide a `request_connection` request and response for any key the connecting node provides. The response will be a list of nodes to connect to with the range that the joining node must create a key for (as per this [RFC](https://github.com/maidsafe/rfcs/blob/master/text/0030-secure-node-join/0030-nodes_key_as_name.md). The connection will be dropped on delivery of these messages to the joining node or a period of 60 seconds (to be confirmed)


#### Relayed connections


### `RelayNode` reward

On receipt of a `Get` request a DataManager will trigger safecoin checks. These checks use a balancing algorithm to calculate a modulo that will be tested against the `Get` request and if successful a safecoin is awarded. In this case the very same principle is utilised with a slight twist. The same balancing algorithm is used as this ensure the network has the resources that it requires.

As the network requires new nodes at whichever rate the algorithm has calculated at any point in time a `RelayNode` can be promoted to a `ManagedNode`. This will require the node is located to a new group and the simplest method to achieve this is to allow the node to start via the normal join method.

* A node can **only** become a `RelayNode` if it starts fresh with no joining token.
* A node that starts with a joining token though will become a `ManagedNode` As with safecoin this joining token will be recycled on use.
* A routing table node that is acting as a `RelayNode`, however will be rewarded in safecoin instead of a joining token.
* On payment of a reward, if the node is routing table connected then the reward is safecoin, otherwise the reward is a joining token.


### Joining token

A joining token is a structured data type 6. Whereas safecoin itself is type 7 this token can be considered a safecoin clone. A significant difference though that a joining token is not intended as trasferrable.

To further prevent large scale collusion type attacks, these tokens will be short lived. The most approprite mechanism right now is to not store these on a data chain or transmit them via churn events. This will mean that these will die after time, unless they are used. A disadvantage, maybe, is that in periods of massive churn then nodes may have difficulty quicly recovering the network. This is not necessarily a disadvantage though as the network is already in high flux and even settling to a smaller network first can be an advantage.

The joining token will be a very simple struct.

```
struct JoiningToken {
	created_for: sign::PublicKey,
}

```

####Node

A node can start in two modes:

1. Without a joining token, in which case it can only become a `RelayNode` and earn a single Joining token. The node joining process will set a routing_node flag to indicate this is a `RelayNode` joining. IF the recieving group has reached it's limit of `RelayNode`s then this process will start again and the node must re-establish a conneciton with a bootstrap node.

2. With a joining token, in which case it will create and address as usual, connect to that group and pay the joining token (i.e. submit a `Delete` for that token) to the group. That group will then allocate a second group for that node to connect to as in the case above (fresh start). The node joining process will set a routing_node flag to indicate this is a full node joiining.

####Client

A client will boostrap and then join a group exactly the same way a node does. When in this group it will connect to at least the closest 3 nodes in the group and send requests through these at random. On losing any node the client will re-establish connection to a node to maintain conneciton to the closest three nodes.



## Drawbacks

* Users vaults will now take longer to become full nodes and therefore will see an increased delay in the time to farm safecoin.
* When a node requests membership of the network the intial group will require to confirm a joining token (`Get`) and send the delete on to that address. This could cause churn issues and requires discussed in detail.

## Alternatives

* Forced random relocation of nodes (many variants).
* Nodes performing a proof of work type algorithm.
* Node ranking and relocation rules.

* Clients join groups as non routing table nodes. Use a random Id to do so. Still send Put requests through `MaidManagers`. They need not connect to every group member, but  should at least connect to the closest 2 members.

* `RelayNode`s join a group as non routing table Nodes and act as `Bootstrap` nodes and `RelayNodes`.

* `RelayNode`s earn tokens from group members who agree the modulo of their address on any `GetResponse` is 0. This modulo number is calculated as per safecoin reward and uses the same algorithm. As the network requires resources the rewards are more frequent and as the network decides it has enough resources these nodes will take longer to earn a token. For this reason these nodes must connect to every group member that is a routing table node.

## Future work

* Joining token storage and time to live shoudl be further investigated.
* Effort measurement and joining node tasks are expected to evolve.


## Unresolved questions

* `RelayNode`s will require to broadcast their availability on the network for boostrap and relay. This process will be defined in a seperate RFC.
