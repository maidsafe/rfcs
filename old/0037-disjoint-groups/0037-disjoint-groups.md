# Disjoint Sections

- Status: active
- Type: feature
- Related components: kademlia_routing_table, routing, safe_vault
- Start Date: 20-07-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/161
- Supersedes: https://github.com/maidsafe/rfcs/blob/master/text/0019-new-kademlia-routing-logic/0019-new-kademlia-routing-logic.md
- Superseded by:


## Summary

Introduce the notion of a "section" of the network, in addition to the existing notion of a group, and the corresponding routing table logic, such that the network's sections are disjoint and form a partition of the set of nodes. Each node will thus be a member of exactly one section. The section size, however, will not be fixed, as opposed to the group size.

Groups will continue to be usable, though, and might remain the appropriate notion for safe_vault's tasks.

(This RFC was originally called "Disjoint Groups", but it turns out to be useful to keep both the old and the new kind of "group" functionality in the code for different purposes. The new concept is therefore called a "section", and the `GROUP_SIZE` closest nodes to a given address continue to be called the "close group" of that address.)

## Motivation

In the current implementation, each group has exactly `GROUP_SIZE` (8 at the moment) members and each node is a member of multiple groups. This complicates many algorithms and even makes some features unfeasible. Examples of what disjoint sections will facilitate include:

### Disjoint routes

With this design, each section member will know _all_ members of the connected sections. Therefore route `n` can always pass the message to the `n`-th closest (to the recipient) member of the next section. That way, none of the routes can intersect.

### More secure group messages

The group messages do not provide a high level of protection against attacks on the network, even under the additional assumptions that routes are disjoint and that a quorum of the individual messages need to be intercepted. Assume that an attacker controls 10% of all nodes in the network, and group size is 8 and quorum size is 5. Consider a section message sent via 10 hops.

In that scenario, the chance to intercept any given group request is 71%. With the section-to-section hops proposed here, it will be just 0.43%. Further adjusting the section size and quorum size parameters, this can be reduced to a negligible number, making this kind of attack virtually impossible.

### Banning nodes from the network

The routing tables of all nodes in a section will be identical. This allows the network to ban misbehaving nodes: If a node's section decides to ban it from the network, it knows exactly whom this node is connected to, and who needs to be notified to make the whole network disconnect from it.

### Weighted votes and node evaluation

If section `A` is connected to section `B`, every node in `B` will know every member of `A`, so section `B` can easily reach a consensus regarding the correctness of the individual `A`-members' behaviour. This allows `B` to evaluate the nodes in `A` and give feedback about them - another requirement for banning misbehaving nodes. But it could be taken one step further and used to assign grades, and make a nodes' vote weight for section consensus depend on it: Making voting rights depend on a proof of work is essential for the security of the network.

### Data chains

The current group concept would considerably complicate [data chains](https://github.com/dirvine/data_chain): With disjoint sections, all members of a section can sign the same link block - consisting of all the section members -, and are responsible for exactly the same set of data chunks, so they can sign the same data blocks.


## Detailed design

The definition of a section is chosen so that at each point in time, the name space is partitioned into disjoint sections. That means that every address in the network at every point in time belongs to _exactly one_ section. Each section is defined by a _name prefix_, i.e. a sequence of between 0 and `XOR_NAME_BITS` (currently 256) bits, similar to how IP subnets correspond to IP address prefixes. The section `S(p)` consists of all nodes whose name begins with the prefix `p`. The section manages the part of the name space given by the interval `[p00...00, p11...11]`.

The routing table needs to keep track of not only which peers a node is connected to, but also which sections they belong to. And the network's structure is defined not by the current set of nodes alone, but in addition by the sections which currently exist. An address belongs to _exactly one section_ if and only if _exactly one_ of the address' prefixes is the prefix of a current section. So to define a partition of the name space:

* No two sections must be comparable: If `S(p)` and `S(q)` are different sections, then `p` and `q` cannot be a prefix of each other - they must differ in at least one bit that is defined in both of them.
* Every address must have a prefix that belongs to a section.

For example, these sets of sections form valid partitions of the address space:

* `S(00), S(01), S(10), S(11)`
* `S(0), S(10), S(110), S(1110), S(1111)`
* `S()`

Whereas `S(0), S(00), S(10), S(01)` would not be valid, because `0` and `00` are comparable. And `S(01), S(10), S(11)` would not be valid because an address starting with `00` belongs to none of those sections.

When the network is bootstrapped, there is only one section, with the empty prefix `S()`, responsible for the whole name space.

* Whenever `S(p0)` and `S(p1)` satisfy certain section requirements (see below), a section `S(p)` splits into `S(p0)` and `S(p1)`.
* Whenever a section `S(p0)` or `S(p1)` ceases to satisfy the section requirements, it merges with all its sister sections into the section `S(p)` again: All sections that would be a subset of `S(p)` are merged back into `S(p)`.

These are the only two operations allowed on sections, so it is guaranteed that every address in the network belongs to exactly one section, and that section satisfies the requirements. Note that the second rule is applied as soon as _at least one_ of the sections does not satisfy the requirements anymore. E.g. if there are sections `S(111)`, `S(1100)` and `S(1101)`, and `S(111)` does not satisfy the requirements, then even if the other two do, the three sections merge into `S(11)`.

The invariant that needs to be satisfied by the routing table is modified accordingly:

1. A node must have its complete section `S(p)` in its routing table.
2. It must have every member of every section `S(q)` in its routing table, for which `p` and `q` differ in exactly one bit.

If `p` and `q` are not of equal length, that does not count as a difference: A differing bit is one that is defined in _both_ prefixes, but is `1` in one of them and `0` in the other. So e. g. `111`, `1100` and `1101` all differ in exactly one bit from each other.

The sections that differ in the `i`-th bit are the "`i`-th bucket" of `S(p)`.

In a balanced network, point 2 just means that the section `S(q)` _is_ the `i`-th bucket. But if it is not perfectly balanced, it might be the two sections `S(q0)` and `S(q1)` or even more sections with longer prefixes, or some prefix of `q` itself.

The requirement is symmetric: I need you in my routing table if and only if you need me in yours!

It also implies the current Kademlia invariant: I am still connected to each of my bucket groups. (The `GROUP_SIZE` nodes closest to my `i`-th bucket address are necessarily in the same section as the bucket address itself, and therefore cannot differ in more than one bit from my own.)

As an example, consider the section `S(0101)`. Its `0`th bucket consists of all sections that differ exactly in the `0`th bit from `0101`, i.e. all sections whose prefix is comparable to `1101`. That might be the section `S(11)`, the section `S(1101)` or _all three of the sections_ `S(11010)`, `S(110110)` and `S(110111)`. Similarly, its `1`st bucket consists of all sections whose prefixes are comparable with `0001` (they differ in exactly the `1`st bit from `0101`). If, for example, that happens to be exactly section `S(000)`, then the `1`st bucket must contain _all_ members of `S(000)` - those whose name starts with `0001` _and_ those whose name starts with `0000`.

### Section requirements

The section requirements will likely soon become more complex, and involve layers above the routing table in the decision: e.g. a section might require that at least three nodes are confirmed to have stored all the data chunks the section is responsible for. Each section will be able to make the decision to split, and needs to reach consensus on that. As a first step, however, to transition from the current section definition with as small a change as possible, the section requirement should just be:

A section must have at least `GROUP_SIZE` members (that are fully connected i.e. satisfy the routing table invariant).

To avoid repeated splitting and merging due to small fluctuations in section size, the requirement for an actual section split should be slightly stricter than the section requirement itself: e.g. only split a section if both subsections have at least `GROUP_SIZE + 1` members, so that the next leaving node will not cause a section merge again. The criterion for a section merge will be that there are _fewer than_ `GROUP_SIZE` members in one of the merging sections.

### Message routing

When learning about a change in any neighbouring section `B` (new member, section split or merge), each node in a section `A` signs the sorted new list of members of `B` and sends it to each member of `A`, so every node in `A` has a signature from each member of `A`, for the list of current members of `B`.

A new message variant `HopMessageSignature` is added, containing a hash and a signature. The `HopMessage::signature` field is replaced by `signatures: Vec<Signature>`, and a new field `hop_signatures` is added, containing a list of signed section lists. The message is considered valid if each list in `hop_signatures` has valid signatures from at least a quroum of the entries of the next list, and the `signatures` itself has signatures (signing the message itself) by

* a quorum of the entries of `hop_signatures[0]`, if the source authority is a section, or
* one entry of `hop_signatures[0]`, corresponding to the source, if the source authority is a single node.

If a section sends a message, only the `route`-th node creates the full `HopMessage`, and the others send it a `HopMessageSignature`. The recipient keeps the full message in cache until it has collected a quorum of signatures. Then it continues relaying the message as detailed below. If an individual node sends a message, it sends the full `HopMessage` with only its own signature.

To relay a message on a given `route` in a node `n` in section `S(p)` for a destination `d`:

* Push the signed list of the previous hop's section members on `hop_signatures` and drop the message if it is not valid. Then:
* If `n` is a recipient, verify the signatures and handle the message.
* If `d` is a single node in our routing table, relay it directly to `d`.
* If `d` is a section in our routing table, relay it everyone in `d`.
* Otherwise relay the message to the `route`-th closest entry to `d` in our routing table.

For any sections `A` and `B`, either everyone in `A` is closer to `d` than everyone in `B`, or vice versa. Therefore, the _section_ that we relay the message to doesn't depend on the route. Since everyone in our section knows everyone in the next hop's section, this means that in the next attempt `route + 1`, a _different_ member of `B` will receive the message. Unless there is churn in between the attempts, this guarantees that all routes are disjoint.

Group messages can be sent in the same way since every group is a subset of a section.

### Joining nodes

A new node needs to first connect to everyone in its target section `A`. After that, it can drop its proxy node. The other section members can provide the node with their routing tables, which should all be identical. It can then make connections to these other sections, too.

To keep the changes from the current code minimal, the other nodes will just accept the connection for now. This will soon involve another step, though: To be able to trust the new node, all members of all connected sections will need to receive a message signed by the nodes in `A` that confirms that the new node is in fact a new member of `A` now.

### Section split

If a section `S(p)` splits, all its nodes stay connected: The new sections are now each other's `i`-th bucket, where `i` is the number of bits in `p`.

The section sends a direct `SectionSplit(p)` message to all its contacts to inform them about the split. These need to note that the section has split, even if they stay connected to all its members. They now know that there is no section `S(p)` anymore, but there are sections `S(p0)` and `S(p1)`.

A recipient `S(q)` of `SectionSplit` can then disconnect from one of the new sections, if `q` now differs in more than one bit from the new section prefix.

Finally, a `SectionSplit` event needs to be raised so that safe_vault can react to the change, if necessary.

For example, assume that section `S(01)` just got a new member, the new member is now fully connected and the section is ready to split. Assume that it is currently connected to `S(11)`, `S(000)` and `S(001)`. Every member of `S(01)` now sends a `SectionSplit(01)` to all its contacts. When these messages have accumulated in all the sections, every affected node now knows that there is no section `S(01)` anymore, but there are new sections `S(010)` and `S(011)`.
At this point, `S(000)` will disconnect from `S(011)` (it was connected to its members because they were members of `S(01)` before, but now it can drop them) but keep the connections to `S(010)`. `S(11)` will stay connected to both. And `S(001)` will remain connected to `S(011)` and disconnect from `S(010)`. The two new sections themselves remain connected to each other.

### Leaving nodes

A leaving node `n` will cause `LostPeer` events in all its connected peers. With the current code, this will cause all nodes to look for a tunnel to `n`. Only if it has really left the network (and not e.g. just lost a single connection), this will fail and everyone will drop it from the routing table.

No further action is necessary, except if it causes a section merge.

As a later change, leaving the network will likely need to be made more explicit: The section will have to agree on its new configuration, and send a signed message about it to all connected sections.

### Section merge

A section merge occurs if a section ceases to satisfy the section requirements. It then initiates a merge that will strip exactly the last bit from its prefix, i.e. only the section `S(p0)` or `S(p1)` can initiate a merge into section `S(p)`. The initiating section is then already connected to all members of the new section: Their prefixes are all extensions of `p`, so they differed in at most one bit from `p0` resp. `p1` and therefore were connected to the initiating section.

All new section members then need to exchange their routing tables, merge them and establish the missing connections. Finally, they need to notify all of their peers about the section merge. Each node's new routing table will be the union of all the section members' previous tables. In detail, the message flow is as follows:

* The section initiating the merge has only one more bit than `p` and already knows all constituent sections. It sends a `SectionMerge(p, Vec<(Prefix, Vec<XorName>)>)` section message to all of them (including itself), containing the new section prefix and all its routing table entries (including itself) and _their_ section prefixes.
* Every node accumulating a `SectionMerge` establishes connections to all entries of the list and sends a `SectionMerge(p, Vec<(Prefix, Vec<XorName>)>)` section message to all of them, in the same way.
* Each node that has accumulated all the `SectionMerge` messages from all constituent sections of `S(p)` updates its routing table to reflect the change: It now knows that there exists a section `S(p)` and that all other sections `S(p...)` ceased to exist.

Then, a `SectionMerge` event is raised so that safe_vault can react to the change and start relocating data.

Assume, for example, that section `S(001)` doesn't satisfy the section requirements anymore and needs to merge. Let's say it's currently connected to sections `S(10)`, `S(0110)`, `S(0111)`, `S(0000)` and `S(0001)`.
So `S(001)` now sends a `SectionMerge(00, [...])` to all of its contacs, where the list contains its complete current routing table.
Every node in `S(0000)` and `S(0001)` accumulating this message sends a similar message to all of _its_ contacts, containing _its_ routing table.
When all those messages have accumulated, all sections (the merged one and its contacts) know about the new section composition and about which nodes it needs to connect to. They establish those connections and update their routing tables.
After that, the new section `S(00)` will be connected to `S(10)`, `S(0110)` and `S(0111)`, but in addition to all sections with a prefix starting with `010`, let's say `S(0100)` and `S(0101)`.
How did it know about these two sections and their members? It received the information from the `SectionMerge` messages from `S(0000)` and `S(0001)`, because _they_ were already connected to those sections respectively, before the merge.


## Drawbacks

The main drawback is that, compared to the `GROUP_SIZE` constant, sections and routing tables will become larger. Since `GROUP_SIZE` will turn into a minimum instead of an exact size requirement, and sections will usually (if they are balanced) split when they have about `2 * GROUP_SIZE` members, the average section will probably have at least 50% more than `GROUP_SIZE` members. This increases the entries in the routing table and the number of network connections that are simultaneously open. It also increases traffic whenever _each member_ of a section has to take action, e.g. for section messages or data chain signatures.

Finally, churn handling for merging sections will be difficult, and cause considerable traffic.


## Alternatives

* Keep just the current group definition. This will require significant changes to data chains and new ideas for other future functionality, like banning nodes.
* Come up with yet another alternative group definition! There might be a way to get the benefits without the drawbacks.
