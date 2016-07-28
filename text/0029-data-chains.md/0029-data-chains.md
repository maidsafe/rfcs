# Data Chains

- Status: proposed
- Type: new feature
- Related components: (data, routing, vaults)
- Start Date: 08-03-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

[DataChain]s are a container that allows large blocks of data to be maintained. These blocks can be
validated by a node on a network, close to the data name, to contain valid data that was guaranteed 
to have been correctly stored onto the network. Code for this RFC is available on [github]

# Definitions used

- Decentralised network, A peer to peer network in xor space, using Kadmelia type addressing.
- Hash, a cryptographic one way function that produces a fixed length representation of any input.
- Immutable data, a data type that has a name == hash of it's contents (it is immutable as changing
  the contents creates a new piece of immutable data).
- Structured data, a data type that has a fixed name, but mutable contents.
- GROUP_SIZE, the number of nodes surrounding a network address.
- Chain consensus, the fact that majority number of signatories exist in the next link (`DataBlock` as
  described below) that also exist in the previous block.
- Churn event, a change in the group, either by a node leaving or a node joining.

# Abstract

A mechanism to lock data descriptors in containers that may be held on a decentralised network.
Such structures are cryptographically secured in lock step using a consensus of cryptographic
signatures. These signatures are of a certain size (group size) with a majority required to be 
considered valid (much like N of P sharing). In a decentralised network that has secured groups,
these signatures are those closest to the holder of a [DataChain]. The implementation linked at
[github] provides a mechanism to hold data as well as the [DataChain] of descriptors. 

When a [DataChain] begins, the first item is likely a `link`. This is a block that uses the
identity of a close group on the network. This `link` has an associated proof that is the
`PublicKey` and a corresponding signature for each node. The `Signature` is the signed `link`
block.  On each `churn` event a new link is created and again signed by all members of the
close_group. This link is the nodes close group as known by all members of that close_group. The
link is the xor result of that close_group. The first link in a network may be referred to as the 
*Genesis* block. 

Data block entries are signed by an ever changing majority of pre-existing nodes.  As the chain
grows, this rolling majority of different signatories can be cryptographically confirmed (via
`links`).  This process continues to the very top of the chain which will contain entries signed by
a majority of the current close group of nodes. This current group of nodes can then
cryptographically validate the entire chain and every data element referred to within it.


An example of a [DataChain] may look like this. 

![Chain](https://github.com/dirvine/data_chain/blob/master/docs/datachain_diagram.png)

The `links` maintain group consensus and the data elements should individually validate all data
blocks though the group consensus provided by the preceding `link`.

As groups change and the network grows, or indeed shrinks, many chains held by various nodes will
have a common element. This allows such chains to be cross referenced in order to build a complete
picture of data from the start of the network. In essence, this chain of verifiable data elements
provides a provable sequence of data validity and also the sequence of such data appearing on the
network. It is assumed that a later project using graph analysis can provide analytics that may be
subjected to deep learning algorithms that will improve network security and efficiency.

It is through this basic recognition of chained majority agreements that assures the ability for a
[DataChain] to be validated and therefore allows data to be republished.

The design described below will show a system where node capabilities are amortised across a
network, providing a balance of resources that can be mixed evenly across a network of nodes with
varying capabilities, from mass persistent data storage to node with very little, transient data
storage.

# Motivation

In a fully decentralised network there are many problems to solve, two of these issues can be
thought of as:

1. Handling the transfer of large amounts of data to replicant nodes on each churn event.

2. Allowing data to be **republished** in a secure manner.

Point 2 actually encompasses two large issues in itself. The ability to start a node and make it's
data available is obviously required where we have large amounts of data to maintain. Another large
advantage is the ability for such a network to recover from a full system outage (full network
collapse, worldwide power outage etc.).

# Detailed design

## Data covered by a data chain

This proposal is aimed at protecting data by confirming the nodes on the network that were the 
closest to the data at that point in time. This data will have a common number of leading bits 
corresponding to the part of the network they were close to.

[DataChain]s can be validated by a majority of the current nodes close peers. As a chain will be 
transferable (with the data) it will not have an identifier of any particular address. 
Instead the identifiers for the groups will appear somewhat arbitrary. Acceptance of a [DataChain]
by a node will require that the current close nodes in a group have all signed the chain. 

What concerns us in this design is that at least all group members agree on something that they can 
sign to attest to this group having existed on the network. To achieve this we again use `xor` and 
as described below the identifier for links is merely the xor of all group members in relation to 
individual nodes and not any data item itself.

## [BlockIdentifier]

A [BlockIdentifier] is simple enumeration that represents a `Data` item such as  (`structuredData`
or `ImmutableData`).

The other type that can be represented in the `enum` is a `Link`. A `Link` represents a valid group
of nodes that are close to a point in the Xor address space. This point changes with respect to
changing nodes around any address. The representation of the link address in the chain is the Xor 
of all the current close group members of the current node. All close group members will recognise 
the group of this node and this node will also know the close group of all of it's close nodes.

The [BlockIdentifier] that represents a data item contains the hash of that data item. This allows
the [DataChain] to hold identifiers to data that can validate the data itself. This allows the data
to be republished whilst being certain that data was created on the network itself.

To ensure there are no extension attacks possible the data size should also be maintained along
with any other identifying fields deemed required by the implementation. Additionally an HMAC can
be used to further secure the data in question.

## [Block]

A [Block] is made up of a [BlockIdentifier] and a vector of `PublicKey` and `Signature`.This vector
is known as the [Proof]. Each [Proof] tuple can be used to verify the signature is that of the
[BlockIdentifier] and that the `PublicKey` is the one used to sign this.

A link [Block] has the same [Proof] vector. This [Block] type is the glue that holds the chain together
and provides the link of proofs right up until the current group can be identified. It is this
pattern that allows a series of links to be cryptographically secured. As each link is only valid if
signed by all previous members minus 1 of the previous (valid) link then a detectable series is 
calculable.

[Block]s that have data as their [BlockIdentifer] part are merely slotted into the appropriate gap
between links. A block of data is validated in the same manner as the connections between links.

The last valid link can also be tested to contain the current close group (minus 1). In this
case the chain is valid right to the last link. This phenomenon allows all blocks to be shown to be
valid. As a new node then a new link will be created that will contain all of the current close 
group.

## [NodeBlock]

A [NodeBlock] consists of the [BlockIdentifier] and a [Proof]. Nodes will create these
and send them as messages to group members when the network mutates. This will require that for
every `Put` `Delete` or `Post` a new [BlockIdentifier] for that data item is created and sent to
all group members. The [Proof] is this nodes `PublicKey` and `Signature`, allowing the receiving node
to call the [DataChain]'s `fn add_nodeblock()` to try and add this to the data chain.

In times of network churn a node will create a separate `LinkDescriptor` to create the
[BlockIdentifier] for this [NodeBlock]. This `LinkDescriptor` is created by calling the
[create_link_descriptor()] method and passing the close_group **to that node** as the input. Each
node in the group will do the same and send the [NodeBlock] to that node.

This continual updating of the chain also provides a history of part of the network, both in terms 
of data and also groups. Each block will contain a list of the nodes that have been seen on the 
network as the chain evolved.

## [DataChain]

The chain itself is a very simple vector of [Block]s. The [API] of the [DataChain] allows for
splitting, merging and validating chains. This allows chains to be exchanged and validated between
nodes. If a chain can be proven to be owned (by calling the chain validate_ownership
function) by a receiving node then it is considered fully valid.

An interesting aspect though is the ability to "validate in history". This means that even if a
chain cannot be proven to be able to be fully published to a group (as there are not enough
remaining members of the group existing) it may still be queried with a few more conditions.

1. The current receiving node, did exist in the chain and has previously signed a block. Even
though others do not remain this node does believe the chain, but cannot prove it to anyone else.

2. A chain may contain an older link that is validate-able as there is a common link in a current
held chain and the published one. The published chain may hold data after this point that cannot be
validated, however the data up to the point of a common link (a link that holds enough common nodes
to provide a majority against a link we already know in our own chain) can be proven valid. This
phenomenon allows even older data than we know to be collected and increase the length of the
current chain. This allows the adding of "history" to an existing chain.

## Routing requirements

1. A node address will be a cryptographic signing key.

2. A node will attempt to join a previous group with the last known key. It will not though, join
the routing table at that stage. Routing will ask the upper layer (vaults in this case) if that
node is acceptable. While this process is taking place this joining node will be added to a list of
nodes attempting to join. If vaults agree the node is OK then routing will add this node to the
routing table.

3. If vaults reject a node, then it will follow the normal joining process (secure join)

4. Routing must punish nodes ASAP on failure to transmit a Link [NodeBlock] on a churn event. Links
   will validate on majority, but routing will require to maintain security of the chain by ensuring
   all nodes participate effectively. These messages should be high priority.

## Vault requirements

1. A vault will allow majority - 1 nodes to join via the mechanism above.

2. On receiving a join request for a node (from routing), vaults will request the nodes [DataChain]

3. If this nodes [DataChain] is longer than an existing majority - 1 nodes, then nodes query the
joining node for data from the chain and then it is allowed to join.

4. All nodes that can hold a lot of data will try and build their data chain from existing nodes
holding such data (`Archive Nodes`). This data is transferred with the lowest priority.

5. On a churn even a node that is off line for a period may find on restart an existing node did
build a chain and now this restarting node has to join another group to begin the process again of
building a data chain.

6. Nodes will choose the sender of the data on `Get` requests. New nodes will only be expected 
to have data that has appeared since they joined (each node knows this via it's own data chain"). 
New nodes can and will try (if they have resources) to`Get`data from the group. When nodes have 
this data they can request full membership of the group. At that time they can be chosen to respond
to any`Get`request, thereby earning safecoin or rewards. Nodes may then continue to ask for data 
from archive nodes that are outwith the current group data. This may allow them to restart as an 
archive node, maximising their reward time as restarts are much faster since data does not need
relocated.

7. On start a new node will ask 1 member for the group for the chain and all members for the genesis
block only. This allows that node to verify the chain is current and is traverses back to genesis
Ok. If there is doubt over chain validity, other nodes may be asked for the `BlockIdentifiers` only
, should any block be missing then the node that sent this (signed) will be reported to the group
and this action will mean that node is expelled, immediately.

8. A node on startup may request the genesis block from any group and store this locally. 


Nodes will build their chains to become more valuable to the network and therefore earn more
safecoin. This process will encourage high capability nodes to spread evenly across the network.

Lower capability nodes will not attempt to build data history and will therefore have less earning
potential. This is perfectly valid and possibly a requirement of such a network, to allow nodes of 
varying capability (cpu/bandwidth/storage etc.) to exist.

# Additional observations

## Group size

Whilst it was thought that a [DataChain] did not require the use of a magic number, there is a 
requirement at this time for it to know the group size used in the network for group consensus. This 
is unfortunate and hopefully will be factored out. The use of group size though, is required on 
groups splitting and the chain progressing. As this happens a link will potentially lose majority.
In this case the data chain needs to us another factor to decide quorum has been met and this size 
is the group size figure. 

It is hoped that this can be eradicated by a more sophisticated checkpointing mechanism where both 
sides of a split can sign the split link. This would be identifiable as the split happens at a common
leading bits agreement of a number of the group. At this time using a naive algorithm though may 
introduce unwanted and potentially insecure side effects. 

## Archive nodes

Nodes that hold the longest [DataChain]s may be considered to be archive nodes. Such nodes will be
responsible for maintaining all network data for specific areas of the network address range. There
will be less than group_size/2 archive nodes per group. These more reliable nodes and will have a 
vote weight higher than a less capable node within a group. There will still require to be a majority
of group members who agree on votes though, regardless of these high weighted nodes. This is to 
prevent attacks where nodes lasting for long periods in a group cannot collude via some out of 
band method such as publishing ID's on a website and soliciting other nodes in the group to collude 
and attack that group. 

### Archive node [Datachain] length

The length of the [DataChain] should be as long as possible. Although a node may not require to hold
data outwith it's current close group. It is prudent such nodes hold as much of the Chain as
possible as this all allow quicker rebuild of a network on complete outage. Nodes may keep such
structures and associated data in a container that prunes older blocks to make space for new blocks
as new blocks appear (FIFO or first in first out).

#### Additional requirements of Archive nodes

All nodes in a group will build on their [DataChain], whether an Archive node or simply attempting
to become an archive node. Small nodes with little resources though may find it difficult to create
a [DataChain] of any significance. In these cases these smaller less capable nodes will receive
limited rewards as they do not have the ability to respond to many data retrieval requests, if any
at all. These small nodes though are still beneficial to the network to provide connectivity and
lower level consensus at the routing level.

A non archive node can request old data from existing archive nodes in a group, but the rate should
be limited in cases where there are already three such nodes in a group. These messages will be the
lowest priority messages in the group. Thereby any attacker will require to become an archive node
and this will take time, unless the group falls below (group_size / 2) - 1 archive nodes
in which case the priority is increased on such relocation messages.

## Chained chains

As chains grow and nodes hold longer chains across many disparate groups, there will be commonalities
on `DataBlocks` held. Such links across chains has not as yet been fully analysed, however, it is
speculated that the ability to cross reference will enable a fuller picture of network data to be
built up.

### Structured data first version

To strengthen validity of mutable data (`StructuredData`) the first version (version 0) may be
maintained in the chain. This will show age of such data, which may be particularly useful in types
of mutable data that do not change ownership or indeed where network created elements (such as any
currency) can be further validated.

###Network "difficulty"

The distance of the furthest group member to a nodes own ID is regarded as network difficulty. In
small networks this will wildly fluctuate. This value must be written to the nodes configuration
file, in case of SAFE this is the vault configuration file.

## Network restart / mass segmentation

The process described above will mean that decentralised network, far from potentially losing data
on restart should recover with a very high degree of certainty.

If a node restarts or in cases of massive churn there will be a significant reduction in network
difficulty. This reduction will mean that any nodes joining `again` should be accepted, regardless
of chain length.

If a restart has been detected, any node recognised in the last link of the chain will be allowed
entry again.

# Further work 

The current implementation of [DataChain]s is secure, but can be made extremely more efficient over
time. Merkle tree's, checkpoints and more such as use an xored checkpoint that contains not only
a signed checkpoint, but a checkpoint that can evaluate when all contained blocks are available 
by xoring back to all zero's. This RFC does not attempt to include any such efficient mechanism,
instead this is left for further design improvements and RFC's.

# Drawbacks

- In very small networks (less than approx 3000) network difficulty is a fluctuating number, this can
probably not be prevented, but may allow unwanted data or in fact prevent valid data from being
refreshed.
- This pattern is at it's earliest of stages and will require significant testing to ensure integrity of data as well as safety.
- Chain merging and data integrity checking is not well defined in this RFC and will require further analysis during implementation.

# Alternatives

None as of yet

# Unresolved questions

Not initially required, but should be considered in near future.

- Effective handling of removed blocks from the chain. (A holder can remove blocks but not add them) 
- Effective checkpoints of chains to reduce size.
- Store efficiently on disk (disk based key value store of [DataChain])
- Calculate vote weights and ensure collusion is not possible in a group.

[github]: https://github.com/dirvine/data_chain
[Block]: https://dirvine.github.io/data_chain/master/data_chain/chain/block/struct.Block.html
[NodeBlock]: https://dirvine.github.io/data_chain/master/data_chain/chain/node_block/struct.NodeBlock.html
[DataChain]: https://dirvine.github.io/data_chain/master/data_chain/index.html
[Proof]: https://dirvine.github.io/data_chain/master/data_chain/chain/node_block/struct.Proof.html
[BlockIdentifier]: https://dirvine.github.io/data_chain/master/data_chain/enum.BlockIdentifier.html
[create_link_descriptor()]: https://dirvine.github.io/data_chain/master/data_chain/chain/node_block/fn.create_link_descriptor.html
