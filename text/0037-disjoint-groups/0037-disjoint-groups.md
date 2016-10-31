# Disjoint Groups

- Status: active
- Type: feature
- Related components: kademlia_routing_table, routing, safe_vault
- Start Date: 20-07-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/161
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

The routing tables of all nodes in a group will be identical. This allows the network to ban misbehaving nodes: If a node's group decides to ban it from the network, it knows exactly whom this node is connected to, and who needs to be notified to make the whole network disconnect from it.

### Weighted votes and node evaluation

If group `A` is connected to group `B`, every node in `B` will know every member of `A`, so group `B` can easily reach a consensus regarding the correctness of the individual `A`-members' behaviour. This allows `B` to evaluate the nodes in `A` and give feedback about them - another requirement for banning misbehaving nodes. But it could be taken one step further and used to assign grades, and make a nodes' vote weight for group consensus depend on it: Making voting rights depend on a proof of work is essential for the security of the network.

### Data chains

The current group definition would considerably complicate [data chains](https://github.com/dirvine/data_chain): With disjoint groups, all members of a group can sign the same link block - consising of all the group members -, and are responsible for exactly the same set of data chunks, so they can sign the same data blocks.


## Detailed design

The definition of a close group is modified so that at each point in time, the name space is partitioned into disjoint groups. That means that every address in the network at every point in time belongs to _exactly one_ group. Each group is defined by a _name prefix_, i.e. a sequence of between 0 and `XOR_NAME_BITS` (currently 256) bits, similar to how IP subnets correspond to IP address prefixes. The group `G(p)` consists of all nodes whose name begins with the prefix `p`, and is responsible for all data items whose name begins with `p`. The group thus manages the part of the name space given by the interval `[p00...00, p11...11]`.

The routing table needs to keep track of not only which peers a node is connected to, but also how they are grouped. And the network's structure is defined not by the current set of nodes alone, but in addition by the groups which currently exist. An address belongs to _exactly one group_ if and only if _exactly one_ of the address' prefixes is the prefix of a current group. So to define a partition of the name space:

* No two groups must be comparable: If `G(p)` and `G(q)` are different groups, then `p` and `q` cannot be a prefix of each other - they must differ in at least one bit that is defined in both of them.
* Every address must have a prefix that belongs to a group.

For example, these sets of groups form valid partitions of the address space:

* `G(00), G(01), G(10), G(11)`
* `G(0), G(10), G(110), G(1110), G(1111)`
* `G()`

Whereas `G(0), G(00), G(10), G(01)` would not be valid, because `0` and `00` are comparable. And `G(01), G(10), G(11)` would not be valid because an address starting with `00` belongs to none of those groups.

When the network is bootstrapped, there is only one group, with the empty prefix `G()`, responsible for the whole name space.

* Whenever `G(p0)` and `G(p1)` satisfy certain group requirements (see below), a group `G(p)` splits into `G(p0)` and `G(p1)`.
* Whenever a group `G(p0)` or `G(p1)` ceases to satisfy the group requirements, it merges with all its sister groups into the group `G(p)` again: All groups that would be a subset of `G(p)` are merged back into `G(p)`.

These are the only two operations allowed on groups, so it is guaranteed that every address in the network belongs to exactly one group, and that group satisfies the requirements. Note that the second rule is applied as soon as _at least one_ of the groups does not satisfy the requirements anymore. E.g. if there are groups `G(111)`, `G(1100)` and `G(1101)`, and `G(111)` does not satisfy the requirements, then even if the other two do, the three groups merge into `G(11)`.

The invariant that needs to be satisfied by the routing table is modified accordingly:

1. A node must have its complete group `G(p)` in its routing table.
2. It must have every member of every group `G(q)` in its routing table, for which `p` and `q` differ in exactly one bit.

If `p` and `q` are not of equal length, that does not count as a difference: A differing bit is one that is defined in _both_ prefixes, but is `1` in one of them and `0` in the other. So e. g. `111`, `1100` and `1101` all differ in exactly one bit from each other.

The groups that differ in the `i`-th bit are the "`i`-th bucket" of `G(p)`.

In a balanced network, point 2 just means that the group `G(q)` _is_ the `i`-th bucket. But if it is not perfectly balanced, it might be the two groups `G(q0)` and `G(q1)` or even more groups with longer prefixes, or some prefix of `q` itself.

The requirement is symmetric: I need you in my routing table if and only if you need me in yours!

It also implies the current Kademlia invariant: I am still connected to each of my bucket groups. (The `GROUP_SIZE` nodes closest to my `i`-th bucket address are necessarily in the same group as the bucket address itself, and therefore cannot differ in more than one bit from my own.)

As an example, consider the group `G(0101)`. Its `0`th bucket consists of all groups that differ exactly in the `0`th bit from `0101`, i.e. all groups whose prefix is comparable to `1101`. That might be the group `G(11)`, the group `G(1101)` or _all three of the groups_ `G(11010)`, `G(110110)` and `G(110111)`. Similarly, its `1`st bucket consists of all groups whose prefixes are comparable with `0001` (they differ in exactly the `1`st bit from `0101`). If, for example, that happens to be exactly group `G(000)`, then the `1`st bucket must contain _all_ members of `G(000)` - those whose name starts with `0001` _and_ those whose name starts with `0000`.

### Group requirements

The group requirements will likely soon become more complex, and involve layers above the routing table in the decision: e.g. a group might require that at least three nodes are confirmed to have stored all the data chunks the group is responsible for. Each group will be able to make the decision to split, and needs to reach consensus on that. As a first step, however, to transition from the current group definition with as small a change as possible, the group requirement should just be:

A group must have at least `GROUP_SIZE` members (that are fully connected i.e. satisfy the routing table invariant).

To avoid repeated splitting and merging due to small fluctuations in group size, the requirement for an actual group split should be slightly stricter than the group requirement itself: e.g. only split a group if both subgroups have at least `GROUP_SIZE + 1` members, so that the next leaving node will not cause a group merge again. The criterion for a group merge will be that there are _fewer than_ `GROUP_SIZE` members in one of the merging groups.

### Message routing

When learning about a change in any neighbouring group `B` (new member, group split or merge), each node in a group `A` signs the sorted new list of members of `B` and sends it to each member of `A`, so every node in `A` has a signature from each member of `A`, for the list of current members of `B`.

A new message variant `HopMessageSignature` is added, containing a hash and a signature. The `HopMessage::signature` field is replaced by `signatures: Vec<Signature>`, and a new field `hop_signatures` is added, containing a list of signed group lists. The message is considered valid if each list in `hop_signatures` has valid signatures from at least a quroum of the entries of the next list, and the `signatures` itself has signatures (signing the message itself) by

* a quorum of the entries of `hop_signatures[0]`, if the source authority is a group, or
* one entry of `hop_signatures[0]`, corresponding to the source, if the source authority is a single node.

If a group sends a message, only the `route`-th node sends the full `HopMessage`, and the others only a `HopMessageSignature`. The recipient keeps the full message in cache until it has collected a quorum of signatures. Then it continues relaying the message as detailed below. If an individual node sends a message, it sends the full `HopMessage` with only its own signature.

To relay a message on a given `route` in a node `n` in group `G(p)` for a destination `d`:

* Push the signed list of the previous hop's group members on `hop_signatures` and drop the message if it is not valid. Then:
* If `n` is a recipient, verify the signatures and handle the message.
* If `d` is a single node in our routing table, relay it directly to `d`.
* If `d` is a group in our routing table, relay it everyone in `d`.
* Otherwise relay the message to the `route`-th closest entry to `d` in our routing table.

For any groups `A` and `B`, either everyone in `A` is closer to `d` than everyone in `B`, or vice versa. Therefore, the _group_ that we relay the message to doesn't depend on the route. Since everyone in our group knows everyone in the next hop's group, this means that in the next attempt `route + 1`, a _different_ member of `B` will receive the message. Unless there is churn in between the attempts, this guarantees that all routes are disjoint.

### Joining nodes

A new node needs to first connect to everyone in its target group `A`. After that, it can drop its proxy node. The other group members can provide the node with their routing tables, which should all be identical. It can then make connections to these other groups, too.

To keep the changes from the current code minimal, the other nodes will just accept the connection for now. This will soon involve another step, though: To be able to trust the new node, all members of all connected groups will need to receive a message signed by the nodes in `A` that confirms that the new node is in fact a new member of `A` now.

### Group split

If a group `G(p)` splits, all its nodes stay connected: The new groups are now each other's `i`-th bucket, where `i` is the number of bits in `p`.

The group sends a direct `GroupSplit(p)` message to all its contacts to inform them about the split. These need to note that the group has split, even if they stay connected to all its members. They now know that there is no group `G(p)` anymore, but there are groups `G(p0)` and `G(p1)`.

A recipient `G(q)` of `GroupSplit` can then disconnect from one of the new groups, if `q` now differs in more than one bit from the new group prefix.

Finally, a `GroupSplit` event needs to be raised so that safe_vault can react to the change, if necessary.

For example, assume that group `G(01)` just got a new member, the new member is now fully connected and the group is ready to split. Assume that it is currently connected to `G(11)`, `G(000)` and `G(001)`. Every member of `G(01)` now sends a `GroupSplit(01)` to all its contacts. When these messages have accumulated in all the groups, every affected node now knows that there is no group `G(01)` anymore, but there are new groups `G(010)` and `G(011)`.
At this point, `G(000)` will disconnect from `G(011)` (it was connected to its members because they were members of `G(01)` before, but now it can drop them) but keep the connections to `G(010)`. `G(11)` will stay connected to both. And `G(001)` will remain connected to `G(011)` and disconnect from `G(010)`. The two new groups themselves remain connected to each other.

### Leaving nodes

A leaving node `n` will cause `LostPeer` events in all its connected peers. With the current code, this will cause all nodes to look for a tunnel to `n`. Only if it has really left the network (and not e.g. just lost a single connection), this will fail and everyone will drop it from the routing table.

No further action is necessary, except if it causes a group merge.

As a later change, leaving the network will likely need to be made more explicit: The group will have to agree on its new configuration, and send a signed message about it to all connected groups.

### Group merge

A group merge occurs if a group ceases to satisfy the group requirements. It then initiates a merge that will strip exactly the last bit from its prefix, i.e. only the group `G(p0)` or `G(p1)` can initiate a merge into group `G(p)`. The initiating group is then already connected to all members of the new group: Their prefixes are all extensions of `p`, so they differed in at most one bit from `p0` resp. `p1` and therefore were connected to the initiating group.

All new group members then need to exchange their routing tables, merge them and establish the missing connections. Finally, they need to notify all of their peers about the group merge. Each node's new routing table will be the union of all the group members' previous tables. In detail, the message flow is as follows:

* The group initiating the merge has only one more bit than `p` and already knows all constituent groups. It sends a `GroupMerge(p, Vec<(Prefix, Vec<XorName>)>)` group message to all of them (including itself), containing the new group prefix and all its routing table entries (including itself) and _their_ group prefixes.
* Every node accumulating a `GroupMerge` establishes connections to all entries of the list and sends a `GroupMerge(p, Vec<(Prefix, Vec<XorName>)>)` group message to all of them, in the same way.
* Each node that has accumulated all the `GroupMerge` messages from all constituent groups of `G(p)` updates its routing table to reflect the change: It now knows that there exists a group `G(p)` and that all other groups `G(p...)` ceased to exist.

Then, a `GroupMerge` event is raised so that safe_vault can react to the change and start relocating data.

Assume, for example, that group `G(001)` doesn't satisfy the group requirements anymore and needs to merge. Let's say it's currently connected to groups `G(10)`, `G(0110)`, `G(0111)`, `G(0000)` and `G(0001)`.
So `G(001)` now sends a `GroupMerge(00, [...])` to all of its contacs, where the list contains its complete current routing table.
Every node in `G(0000)` and `G(0001)` accumulating this message sends a similar message to all of _its_ contacts, containing _its_ routing table.
When all those messages have accumulated, all groups (the merged one and its contacts) know about the new group composition and about which nodes it needs to connect to. They establish those connections and update their routing tables.
After that, the new group `G(00)` will be connected to `G(10)`, `G(0110)` and `G(0111)`, but in addition to all groups with a prefix starting with `010`, let's say `G(0100)` and `G(0101)`.
How did it know about these two groups and their members? It received the information from the `GroupMerge` messages from `G(0000)` and `G(0001)`, because _they_ were already connected to those groups respectively, before the merge.


## Drawbacks

The main drawback is that, compared to the `GROUP_SIZE` constant, groups and routing tables will become larger. Since `GROUP_SIZE` will turn into a minimum instead of an exact size requirement, and groups will usually (if they are balanced) split when they have about `2 * GROUP_SIZE` members, the average group will probably have at least 50% more than `GROUP_SIZE` members. This increases the entries in the routing table and the number of network connections that are simultaneously open. It also increases traffic whenever _each member_ of a group has to take action, e.g. for group messages or data chain signatures.

Finally, churn handling for merging groups will be difficult, and cause a lot of traffic. While in a stable or growing network, this should be a rare event, if it _does_ happen, possibly none of the nodes in the merged group will have all of the group's data now. The vault logic will need to handle this situation appropriately and try to restore data replication as soon as possible, so that there is again a number of nodes that have all the group's data.

This is even more difficult if the group requirement is made more strict: the fact that, being responsible for more data now, no node might have the complete group's data anymore, must not lead to yet another merge, which would only make things worse. However, the group requirement will likely only be changed later, as a part of the data chains introduction: With these, the merged group will exchange all data blocks so that every node knows about all of the group's data. Then relocation of the actual data will take place gradually after that.


## Alternatives

* Keep the current group definition. This will require significant changes to data chains and new ideas for other future functionality, like banning nodes.
* Come up with yet another alternative group definition! There might be a way to get the benefits without the drawbacks.
