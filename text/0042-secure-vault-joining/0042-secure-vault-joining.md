# Secure vault joining

- Status: proposed 
- Type: enhancement
- Related components: (vault/routing)
- Start Date: 03-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this) 
- Supersedes: None
- Superseded by: N/A

## Summary

This RFC will propose a method to enforce a joining "cost" for new or restarting vaults to join the network. This is a mechanism to prevent mass joining quickly and therefor will introduce a cost of such an attack that is proportional to the network "effort" over time.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

**Effort** - An amount of work performed.


## Motivation

SAFE has already taken advantage of securing groups and group consensus. It also, with disjoint groups, secures hops which prevents nodes masquarading as network identities. It still though has a weakness in a possible attack where an attacker can easily restart many nodes constantly to target a network location. The difficulty of such an attack is not considered here, merely the opportunity of such an attack exists in some form. 

This RFC prevents this attack and claims that this will provide an overall network security level that would be appropriate for securing group creation and thereby groups holding even the most valuable data such as digital currencies.

## Detailed design

The Effort of the network should allow that a proportionate amount of effort is carried out by a node wishing to join. This distinction requires a measurement of such work as well as a task that is adventageous to the network. One such tas that is currently under-rewarded is providing the access points (relays) for client nodes.  This function is onerous on nodes in the netwoork, but at the same time it is critical.This is the initial observation. 

### `RelayNode` overview

When a client selects a node to `bootstrap` from then this node will route client requests and responses to and from the network. (it is assumed clients will select several such nodes in parallel). 

Nodes acting as proxies have a recodnisable address on the network, which is a client address wrapped around the nodes actual network address. This requires that such nodes are already network connected and located in a group. It is suggested here that there will be a limited number of such nodes in any (non full) group. The number of these nodes should be tested during implementaiton testing. Additional attemtps to connect will be rejected in this group. This follows the joining limit already used and tested in the network testnets. 

These nodes will be a fundimental type `RelayNode` This node will not be considered in group refresh messages, or any messages related to data, but it would have messages routed back to it (responses). 

Of course any node can also provide these resources in times of need.

### `RelayNode` reward

On reciept of a `Get` request DataManager trigger safecoin checks. These checks use a balancing algorithm to calculate a modulo that will be tested against the `Get` request and if successful a safecoin is awarded. In this case the very same principle is utlised with a slight twist. The same balancing algorithm is used as this ensure the network has the resources that it requires. 

As the network requires new nodes at whichever rate the algorithm has calculated at any point in time a `RelayNode` can be promoted to a `ManagedNode`. This will require the node is located to a new group and the simplest method do achive this is to allow the node to start via the normal join method. 

This presents the larger picture here, a node can **only** become a `RelayNode` if it starts fresh with no joining token. 

A node that starts with a joining token though will become a `ManagedNode` As with safecoin this joining token will be recycled on use.

### Joining token 

A joining token is a structured data type 6. Whereas safecoin itself is type 7 this token can be considered a safecoin clone. 

####Node

A node can start in two modes:

1. Without a joining token, in which case it can only become a `RelayNode`
2. With a joining token, in which case it will create and address as usual, connect to that group and pay the joining token (i.e. submit a `Delete`for that token) to the group. That group will then allocate a second group for that node to connect to as in the case above (fresh start). 

####Client

TBD 

### 

## Drawbacks

* Users vaults will now take longer to become full nodes and therefore will see an increased delay in the time to farm safecoin. 

## Alternatives

* Forced random relocation of nodes (many variants).
* Nodes performing a proof of work type algorithm.
* Node ranking and relocation rules.

## Unresolved questions

What parts of the design are still to be done?
