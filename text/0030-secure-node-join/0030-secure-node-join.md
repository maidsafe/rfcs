- Feature Name: Secure node join
- Status: proposed
- Type: enhancement
- Related components: routing
- Start Date: 08-03-2013
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Securely join a node to network

# Motivation

Currently node relocation involves a convoluted mechanism that ends with a non cryptographically
validatable Id. A much more secure mechanism will ensure group membership is easily confirmed as
well as allow a much simpler API, ridding the network of PeerId as well as NodeID. A single NodeId
will be a secure hash of a public key, This also allows encrypted messages to nodeId's on the
network to confirm they are in fact network connected nodes. This is achieved by sending a message
to a group that is encrypted for a single group member. Only the member can read the message and
reply.

# Detailed design


In the current implementation as seen in this sequence diagram XXXXX involves several steps. This
proposal involves altering this only slightly.

1. A node (A) creates a keypair and connects to a group (X).
2. Group (X) then take the closest nodes to this.
3. The resultant hash of these closest nodes yields the target group for the node to join (Y).
4. Group (X) then `GetGroup` (Y) and return this group to the client.
5. Client (A) signs a `JoinGroup` request for (Y) and sends this back to (X).
6. Group (X) then sends this valid request onto group (Y).
7. Group (Y) then hold this request in a join cache (lruCache) of GroupSize
8. Node (A) then generates keypairs until it generates one that would fall within group (Y)
9. Node (A) then sends a join request for this newly created ID to (Y) but signs this with the
previous key for (A).
10. Group (Y) receive this request and check their cache, if accepted then joining resumes as
normal, This new ID is added to expected nodes and group (Y) wait on `ConnectRequests` from node
(A)'s new ID. '

Node (A) will then have and ID that is cryptographically validated and allow the network to have
much more flexibility in handling requests and data for node A.


# Drawbacks

None known at this time.

# Alternatives

The status quo does indeed work, but loses cryptographic guarantees that this scheme offers.

# Unresolved questions

TBD
