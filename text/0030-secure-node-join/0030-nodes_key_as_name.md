# Nodes key as name

- Status: proposed
- Type: enhancement
- Related components: routing
- Start Date: 08-03-2013
- Discussion: https://github.com/maidsafe/rfcs/issues/127
- Supersedes:
- Superseded by:

## Summary

Join a network and be known directly by the value of a cryptographically secured public key. This
will allow nodes to send messages that are self validating and removes the requirement of holding 
maps of publick keys and node names. 

## Motivation

Currently node relocation involves a convoluted mechanism that ends with a non cryptographically
validatable Id.A single NodeId will be a public key.

In the current implementation as seen in the diagram below involves several steps. This
proposal involves altering this only slightly.

## Detailed design

The following outline alters these steps to ensure the joining node does some work. This work 
is in itself not enough to prevent group collusion, but is a start in the right direction.  

1. A node (A) creates a keypair and connects to a group (X).
2. Group (X) then take the 2 closest nodes to this and hash the three values (2 closest plus node).
3. The resultant hash of the 2 closest nodes yields the target group for the node to join (Y).
4. Group (X) then sends a `JoinRequest` (Y).
5. Group (Y) then calculates the furthest two nodes apart in group (Y)
6. Group (Y) then respond to (A) with `JoinResponse` with the middle third of the two furthest 
   apart nodes as the target range. (`JoinResponse` includes all Y members names).
7. Group (Y) will set a pending join request for a period of 30 seconds or so.
   - This group will not accept any more `JoinRequest` during this time.
8. Node (A) then must create an address that falls between these two nodes and then join (Y).

This process scales each group in a balanced manner.

## Further work

This process alone **may** not prevent collusion attacks completely (dependent on network population 
and churn). It does force distribution within groups. Further steps may require to be taken, such as:

- Every new node connecting to a group forces relocation of the node closest to any other 
  in the group. (splitting close nodes apart and again helping balance).
- Every new node connecting to a group forces relocation of the oldest node.

There are many such actions routing can take, but combined with the upcoming RFC for `DataChains` 
the options for also forced relocation of nodes is increased, particularly misbehaving nodes should 
be easily spotted and forced out of any group. 

This is a partial mechanism to overall group security and only focusses on the joining mechanism. 
The network will require that nodes are ranked and prove themselves prior to becoming full nodes, 
during that process it may prove beneficial to forcibly relocate a node at intervals. If this 
process is chosen, then the node should again create a new key pair and be forced to join a 
random part of the network. 

## Drawbacks

This does provide a slightly smaller amount of control over the location a node choses within a 
group. 

## Alternatives

The status quo provides relocation capability, but requires an additional mapping  and larger 
message sizes.

## Unresolved questions

TBD
