# Disjoint Groups

- Status: proposed
- Type: feature
- Related components: kademlia_routing_table, routing, safe_vault
- Start Date: 20-07-2016
- Discussion:
- Supersedes: https://github.com/maidsafe/rfcs/blob/master/text/0019-new-kademlia-routing-logic/0019-new-kademlia-routing-logic.md
- Superseded by:


## Summary

Change the definition of the term "close group", and the corresponding routing table logic, such that all the network's groups are disjoint and form a partition of the set of nodes. Each node will thus be a member of exactly one group. The group size, however, will not be fixed anymore.

## Motivation

In the current implementation, each group has exactly `GROUP_SIZE` (8 at the moment) members and each node is a member of multiple groups. This complicates many algorithms and even makes some features unfeasible. Examples of what disjoint groups will facilitate include:

### Disjoint routes

With this design, each group member will know _all_ members of the connected groups. Therefore route `n` can always pass the message to the `n`-th closest (to the recipient) member of the next group. That way, none of the routes can intersect.

### More secure group messages

The current group messages do not provide a high level of protection against attacks on the network, even under the additional assumptions that routes are disjoint and that a quorum of the individual messages need to be intercepted. Assume that an attacker controls 10% of all nodes in the network, and group size is 8 and quorum size is 5. Consider a group message sent via 10 hops.

In that scenario, the chance to intercept any given group request is 71%. With the group-to-group hops proposed here, it will be just 0.43%. Further adjusting the group size and quorum size parameters, this can be reduced to a negligible number, making this kind of attack virtually impossible.

### Banning nodes from the network

The routing tables of all nodes in a group will be identical. This allows the network to ban misbehaving nodes: If a node's group decides to ban the node from the network, it knows exactly whom this node is connected to, and who needs to be notified to make the whole network disconnect from that it.

### Weigthed votes and node evaluation

If group `A` is connected to group `B`, every node in `B` will know every member of `A`, so group `B` can easily reach a consensus regarding the correctness of the individual `A`-members' behaviour. This allows `B` to evaluate the nodes in `A` and give feedback about them - another requirement for banning misbehaving nodes. But it could be taken one step further and used to assign grades, and make a nodes' vote weight for group consensus depend on it: Making voting rights depend on a proof of work is essential for the security of the network.

### Data chains

The current group definition would considerably complicate [data chains](https://github.com/dirvine/data_chain): With disjoint groups, all members of a group can sign the same link block - consising of all the group members -, and are responsible for exactly the same set of data chunks, so they can sign the same data blocks.


## Detailed design

The definition of a close group is modified so that at each point in time, the name space is partitioned into disjoint groups. Each group is defined by a _name prefix_, i.e. a sequence of between 0 and `XOR_NAME_BITS` (currently 256) bits, similar to how IP subnets correspond to IP address prefixes. The group `(p)` consists of all nodes whose name begins with the prefix `p`, and is responsible for all data items whose name begins with `p`. The group thus manages the part of the name space given by the interval `[p00...00, p11...11]`.

When the network is bootstrapped, there is only one group, with the empty prefix `()`, responsible for the whole name space.

* Whenever `(p0)` and `(p1)` satisfy certain group requirements (see below), a group `(p)` splits into `(p0)` and `(p1)`.
* Whenever a group `(p0)` or `(p1)` ceases to satisfy the group requirements, it merges with all its sister groups into the group `(p)` again: All groups that would be a subset of `(p)` are merged back into `(p)`.

These are the only two operations allowed on groups, so it is guaranteed that every address in the network belongs to exactly one group, and that group satisfies the requirements. Note that the second rule is applied as soon as _at least one_ of the groups does not satisfy the requirements anymore. E.g. if there are groups `(111)`, `(1100)` and `(1101)`, and `(111)` does not satisfy the requirements, then even if the other two do, the three groups merge into `(11)`.

The invariant that needs to be satisfied by the routing table is modified accordingly:

1. A node must have its complete group `(p)` in its routing table.
2. It must have every member of every group `(q)` in its routing table, for which `p` and `q` differ in exactly one bit.

If `p` and `q` are not of equal length, that does not count as a difference: A differing bit is one that is defined in _both_ prefixes, but is 1 in one of them and 0 in the other. So e. g. `111`, `1100` and `1101` all differ in exactly one bit from each other.

The groups that differ in the `i`-th bit are the "`i`-th bucket" of `(p)`.

In a balanced network, point 2 just means that the group `(q)` _is_ the `i`-th bucket. But if it is not perfectly balanced, it might be the two groups `(q0)` and `(q1)` or even more groups with longer prefixes, or some prefix of `q` itself.

The requirement is symmetric: I need you in my routing table if and only if you need me in yours!

It also implies the current Kademlia invariant: I am still connected to each of my bucket groups. (The `GROUP_SIZE` nodes closest to my `i`-th bucket address are necessarily in the same group as the bucket address itself, and therefore cannot differ in more than one bit from my own.)

### Group requirements

The group requirements will likely soon become more complex, and involve layers above the routing table in the decision: e.g. a group might require that at least three nodes are confirmed to have stored all the data chunks the group is responsible for. Each group will be able to make the decision to split, and needs to reach consensus on that. As a first step, however, to transition from the current group definition with as small a change as possible, the group requirement should just be:

A group must have at least `GROUP_SIZE` members (that are fully connected i.e. satisfy the routing table invariant).

To avoid repeated splitting and merging due to small fluctuations in group size, the requirement for an actual group split should be slightly stricter than the group requirement itself: e.g. only split a group if both subgroups have at least `GROUP_SIZE + 1` members, so that the next leaving node will not cause a group merge again. The criterion for a group merge will be that there are _fewer than_ `GROUP_SIZE` members in one of the merging groups.

### Message routing

To relay a message from an individual node on a given `route` in a node `n` in group `(p)` for a destination _node_ `d`:

* If `d == n`, handle the message.
* If `d` is in our routing table, relay it directly to `d`.
* If `d` is in our group but no node with that name exists, drop the message. (Log it.)
* Otherwise relay the message to the `route`-th closest entry to `d` in our routing table.

If the destination is a _group authority_ with address `d`:

* If `p` is a prefix of `d`, handle the message and relay it to everyone else in `(p)`.
* Otherwise relay the message to the `route`-th closest entry to `d` in our routing table.

For any groups `A` and `B`, either everyone in `A` is closer to `d` than everyone in `B`, or vice versa. Therefore, the _group_ that we relay the message to doesn't depend on the route. Since everyone in our group knows everyone in the next hop's group, this means that in the next attempt `route + 1`, a _different_ member of `B` will receive the message. Unless there is churn in between the attempts, this guarantees that all routes are disjoint.

As mentioned above, the current group message routing mechanism is not secure. To prevent intercepting group messages, we will instead relay them from group to group, verifying quorum in every step:

Assume a node `n` in group `B` receives a group message signed by someone in group `A`:

* If `A` is not connected to `B`, drop the message.
* If this message has not yet been signed by a quorum of members of `A`, keep it in the cache.
* If it has been signed by a quorum of members of `A`: If we are the recipient, handle it. If not, find the connected group `C` closest to the destination and send the message, signed by `n`, to _every member_ of `C`.

This creates a number of direct messages in each hop that grows quadratically in the group size. To reduce network traffic, the `route`-th node of group `B` could accumulate `A`'s signatures instead, making the message number linear in the group size, at the cost of having effectively two network hops per group hop:

* If `A` is not connected to `B`, drop the message.
* If this message has not yet been signed by a quorum of members of `A`, keep it in the cache.
* If it has been signed by a quorum of members of `A`: If we are the recipient, handle it. If not, find the connected group `C` closest to the destination and send the message, signed by `n`, to the `route`-th closest member of `C`.
* If it has been signed by a quorum of members of `A` and we are the `route`-th closest member of `B`, send the message with `A`'s signatures to every member of `B`.

The quorum cannot be a constant anymore, due to varying group sizes. It needs to be a percentage strictly greater than 50% instead, and in a group of size `n`, a number `x` of nodes will constitute a quorum if `x / n >= QUORUM`.

### Joining nodes

A new node needs to first connect to everyone in its target group `A`. After that, it can drop its proxy node. The other group members can provide the node with their routing tables, which should all be identical. It can then make connections to these other groups, too.

To keep the changes from the current code minimal, the other nodes will just accept the connection for now. This will soon involve another step, though: To be able to trust the new node, all members of all connected groups will need to receive a message signed by the nodes in `A` that confirms that the new node is in fact a new member of `A` now.

### Group split

If a group `(p)` splits, all its nodes stay connected: The new groups are now each other's `i`-th bucket, where `i` is the number of bits in `p`.

The group sends a direct message to all its contacts to inform them about the split. These need to note that the group has split, even if they stay connected to all its members.

The node can then disconnect immediately (no more `ConnectionUnneeded` required) from every group `(q)` for which `q` now differs in more than one bit from its new group prefix.

Finally, a `GroupSplit` event needs to be raised so that safe_vault can react to the change, if necessary.

### Leaving nodes

A leaving node `n` will cause `LostPeer` events in all its connected peers. With the current code, this will cause all nodes to look for a tunnel to `n`. Only if it has really left the network (and not e.g. just lost a single connection), this will fail and everyone will drop it from the routing table.

No further action is necessary, except if it causes a group merge.

As a later change, leaving the network will likely need to be made more explicit: The group will have to agree on its new configuration, and send a signed message about it to all connected groups.

### Group merge

When a new group `(p)` is created by merging, all its members are already connected. They need to exchange their routing tables, merge them and establish the missing connections. Finally, they need to notify all of their peers about the group merge. Each node's new routing table will be the union of all the group members' previous tables.

Then, a `GroupMerge` event is raised so that safe_vault can react to the change and start relocating data.


## Drawbacks

The main drawback is that, compared to the `GROUP_SIZE` constant, groups and routing tables will become larger. Since `GROUP_SIZE` will turn into a minimum instead of an exact size requirement, and groups will usually (if they are balanced) split when they have about `2 * GROUP_SIZE` members, the average group will probably have at least 50% more than `GROUP_SIZE` members. This increases the entries in the routing table and the number of network connections that are simultaneously open. It also increases traffic whenever _each member_ of a group has to take action, e.g. for group messages or data chain signatures.

The group message routing will involve more steps, either doubling the number of actual network hops, or making the number of direct messages per hop increase quadratically in the average group size!

Finally, churn handling for merging groups will be difficult, and cause a lot of traffic. While in a stable or growing network, this should be a rare event, if it _does_ happen, possibly none of the nodes in the merged group will have all of the group's data now. The vault logic will need to handle this situation appropriately and try to restore data replication as soon as possible, so that there is again a number of nodes that have all the group's data.

This is even more difficult if the group requirement is made more strict: the fact that, being responsible for more data now, no node might have the complete group's data anymore, must not lead to yet another merge, which would only make things worse. However, the group requirement will likely only be changed later, as a part of the data chains introduction: With these, the merged group will exchange all data blocks so that every node knows about all of the group's data. Then relocation of the actual data will take place gradually after that.


## Alternatives

* Keep the current group definition. This will require significant changes to data chains and new ideas for other future functionality, like banning nodes.
* Come up with yet another alternative group definition! There might be a way to get the benefits without the drawbacks.
