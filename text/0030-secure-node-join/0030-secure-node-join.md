# Secure node join

- Status: proposed
- Type: enhancement
- Related components: routing
- Start Date: 08-03-2013
- Discussion: https://github.com/maidsafe/rfcs/issues/127
- Supersedes:
- Superseded by:

## Summary

Join a network and be known directly by the value of a cryptographically secured public key. This
will allow nodes to send messages that are self validating without requiring any intermediate steps
 or lookups. Further steps then ensure the joining nodes are distributed across the network.


## Motivation

Currently node relocation involves a convoluted mechanism that ends with a non cryptographically
validatable Id. A much more secure mechanism will ensure group membership is easily confirmed as
well as allow a much simpler API, ridding the network of PeerId as well as NodeID. A single NodeId
will be a public key, This also allows encrypted messages to nodeId's on the
network to confirm they are in fact network connected nodes. 

This is an important first step in ensuring security. A node must not be able to define the space 
in the address range  that it can join as this would invalidate any group consensus as a group would 
 be very easy to create by joining a few nodes in the same space in the network. 


In the current implementation as seen in the diagram below involves several steps. This
proposal involves altering this only slightly.

![bootstrap sequence](http://docs.maidsafe.net/routing/master/routing/bootstrap.png)


## Detailed design

The following outline alters these steps to ensure the joining node does much of the work. This work 
is in itself not enough to prevent group collusion, but is a start in the right direction.  

### Attempt 1: 

1. A node (A) creates a keypair and connects to a group (X).
2. Group (X) then take the closest nodes to this.
3. The resultant hash of these closest nodes yields the target group for the node to join (Y).
4. Group (X) then `GetGroup` (Y) and return this group to the client.
5. Client (A) signs a `JoinGroup` request for (Y) and sends this back to (X).
6. Group (X) then sends this valid request onto group (Y).
7. Group (Y) then hold this request in a join cache (lruCache) of GroupSize
8. Node (A) then generates keypairs until it generates one that would fall within group (Y)
  - The required XorName will comprise a signing key as first part of address and append an 
    encryption key to last part (64 bytes in all)
10. Node (A) then sends a join request for this newly created ID to (Y) but signs this with the
previous key for (A).
10. Group (Y) receive this request and check their cache, if accepted then joining resumes as
normal, This new ID is added to expected nodes and group (Y) wait on `ConnectRequests` from node
(A)'s new ID. 

This process in itself would allow nodes to create many keys in group Y though in a very tight 
address space. That in itself only slows down a collusion attack. 

### Attempt 2: (to be implemented)

1. A node (A) creates a keypair and connects to a group (X).
2. Group (X) then take the closest nodes to this.
3. The resultant hash of these closest nodes yields the target group for the node to join (Y).
4. Group (X) then sends a `JoinRequest` (Y).
5. Group (Y) then calculates the furthest two nodes apart in group (Y)
6. Group (Y) then respond to (A) with `JoinResponse` with the address of the two furthest apart nodes.
7. Group (Y) will set a pending join request for a period of 30 seconds or so.
   - This group will not accept any more requests into this space between nodes during this time.
   - This should not block further attempts to join this group, but not in this space.
8. Node (A) then must create an address that falls between these two nodes and then join (Y).

This process forces new nodes to distribute evenly across groups in a manner that makes location 
type attacks significantly more difficult. 

## Further work

This process alone **may** not prevent collusion attacks completely (dependent on network population 
and churn). It does force distribution across groups. Further steps may require to be taken, such as:

- Every new node connecting to a group forces relocation of the node closest to any other 
  in the group. (splitting close nodes apart and again helping balance).
- Group (Y) maintains a list of all connected groups and uses the furthest two nodes in any group 
  they are all connected to.
- Every new node connecting to a group forces relocation of the oldest node.

There are many such actions routing can take, but combined with the upcoming RFC for `DataChains` 
the options for also forced relocation of nodes is increased, particularly misbehaving nodes should 
be easily spotted and forced out of any group. 

## Drawbacks

This is a partial mechanism to overall group security and only focusses on the joining mechanism. 
The network will require that nodes are ranked and prove themselves prior to becoming full nodes, 
during that process it may prove beneficial to forcibly relocate a node at intervals. If this 
process is chosen, then the node should again create a new key pair and be forced to join a 
random part of the network. 

## Alternatives

The status quo proves relocation capability, but requires indirection to validate an Id belongs 
to a public key. It also can be manipulated by an attacker with relative ease.

## Unresolved questions

TBD
