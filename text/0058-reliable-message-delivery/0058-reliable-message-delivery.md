# Reliable Message Delivery

- Status: proposed
- Type: enhancement
- Related components: DHT, CRUST
- Start Date: 23-05-2019
- Discussion:
- Supersedes: None
- Superseded by: None

## Summary

This RFC presents a method for relaying messages in the network such that every message will be successfully delivered with a very high probability, equal to 1 under the assumption that no section in the network is compromised. This is achieved by choosing a set of relay nodes that should contain at least one correct node at every hop.

## Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

## Motivation

There are two main aspects of message delivery in the network: reliability and security:

- Reliability: how likely it is that a message will reach its destination?
- Security: if we receive a message, can we be sure that the source of the message is what it claims to be?

This RFC attempts to address the reliability issue by proposing a method that guarantees message delivery with probability close to 1 (exactly 1 if the assumptions are satisfied). The security part is addressed in [another RFC](https://github.com/maidsafe/rfcs/blob/master/text/0056-secure-message-delivery/0056-secure-message-delivery.md).

This RFC DOES NOT attempt to tackle network spam as that is a separate issue and not restricted to message passing alone. Spam is an issue that may be intra-section as well as inter-section. This paper is focussed on inter-section messaging.

## Detailed design

### Assumptions

1. Every section in the network consists of two types of nodes:
   - Elders: their responsibilities are network control, messaging and consensus layer.
   - Adults: their responsibilities are the storage and maintenance of data.
2. Every section contains strictly less than a third of Byzantine faulty (ie. unresponsive or malicious) Elders.

Further, we assume that a node being correct - that is, not being faulty - implies that it has all the data necessary to pass any message it receives to a next hop. An implementation of this RFC must take special care to satisfy this assumption. Ways to achieve that will be discussed below.

### General mechanism of delivery

The general outline of the delivery mechanism proposed here is as follows:

1. A subset of nodes in section X decide to send a message. (This can be, for example, a single Elder or Adult, or a whole section. It can also be a client connected to section X.)
2. This subset signs the message and passes it to a Delivery Group within X. X becomes the first hop section for the message.
3. In every hop section, every node in the Delivery Group executes the following once a single copy of the message is received:
	* if the current section contains the destination, broadcasts the message to the nodes belonging to the destination, otherwise:
	* chooses a neighbouring section closest to the message's destination - the next hop section,
	* calculates the Delivery Group within the next hop section,
	* passes the message to every node in the calculated Delivery Group

Choosing the Delivery Group is the crux of the method presented in this RFC.

### Delivery Group

The Delivery Group is a subset of the Elders of the section that contains at least a third of the total number of Elders in the section. This can be chosen eg. as the third of the Elders that are closest to the message hash, or closest to the message destination, but other ways are also possible. The choice should be pseudorandom and determined by the message, so that different messages will usually be relayed by different Delivery Groups, and so that every node can choose the same Delivery Group in any given section.

Since we assume that less than a third of Elders are faulty in any section, the Delivery Group will contain at least one correct Elder. The assumption that correct nodes have all information necessary to pass the message to the next hop then guarantees successful delivery to the destination.

Note that the size of a Delivery Group can also be expressed another way: if we assume that there can be at most `f` faulty Elders in a section, the size of a Delivery Group in that section will be `f+1`. This also means that a section won't be able to correctly handle more than `|DG|-1` faulty nodes, where `|DG|` is the size of a Delivery Group.

### Complexity and reliability analysis

A hop X with delivery group `DGX` passing the message to the next hop Y with delivery group `DGY` will generate up to `|DGX|*|DGY|` messages, where `|A|` is the number of members of A.

If the number of Elders per section is constant and equals `N`, this means `|DG| = ceil(N/3)` and `|DGX|*|DGY| = ceil(N/3)²` messages per hop, so in general the scaling is quadratic.

Since we require `f` to be at least 2 due to considerations regarding handling churn and possible malice at the same time, this implies `|DG| ≥ 3`. Hence, we intend to start with `|DG| = 3` in the initial implementation in order to minimise overhead - which means that `N` should be either 7, 8 or 9 - and increase it if we discover that this isn't enough.

In a poorly connected network, the scaling may potentially be super-quadratic. In such conditions, different nodes in one hop section could choose different Delivery Groups in the next hop section, leading to a larger number of nodes receiving and relaying messages, thus increasing the number of messages. This would, however, tend to self-correct in hop sections in which members agree about the Delivery Group in the next hop section. The upside of this is high reliability in the presence of merges, splits and other disruptions.

As mentioned above, reliability is 100% as long as the assumptions are satisfied. Therefore, any failure of the mechanism will necessarily be caused by a failure to satisfy the assumptions. The probability of failure is thus less than the probability of the assumptions not being satisfied - which must be ensured to be as low as possible by means outside the scope of this RFC.

### Connectivity

This proposal will give the reader some insight into connectivity, but it may not be obvious. Here we are saying that in a section a node (due to PARSEC) must connect to at least 1/3 of its peers. This proposal assumes that these nodes will be able to message 1/3 of each of its neighbour section's Elders as well. As we are saying that 1/3 of a sections Elders will include at least 1 honest node then we can ensure in code that a node will either terminate or request relocation in cases it cannot connect to at least 1/3 of its section Elders AND each of its neighbour section Elders. This can be enforced via PARSEC as well if we considered that we should. In this case each neighbour section's Elders will vote for nodes to be off-line when they cannot message that node. As the PARSEC Quorum is more than 2/3 of Elders in a section, we can see that if more than 2/3 of nodes agree they cannot message such a node, then this section can message the neighbour section to let it know there is an invalid node in their section. It should be noted, however, that an honest node that can connect to >1/3 of its own section SHOULD be able to connect to >1/3 of each neighbour section's Elders.

Recent tests with NAT traversal techniques have shown we can achieve >70% connectivity, this proposal assumes 33% at least.

### Satisfying the assumptions

The two most important assumptions, on which the mechanism depends, are:
1. that less than a third of the Elders in any given section are faulty, and
2. that all non-faulty Elders in a section will be able to relay the message.

Upholding the first assumption is within the scope of Node Ageing - it is expected to provide a very high probability that no section becomes compromised even during a coordinated attack.

The second assumption has been partially discussed in the Connectivity section. By using PARSEC, nodes will ensure that they know who their neighbour sections are, what are their members, and that they agree on this information between each other. They will also ensure that nodes who can't connect to enough of their neighbours are forcibly disconnected from the network.

PARSEC doesn't guarantee that the Elders will always agree on the Delivery Group in the next hop exactly - some of them might be lagging a bit at any given moment. However, if we assume that every Elder chooses `N/3` nodes only from among those to which it is connected, a disagreement will only result in more nodes in the next hop receiving the message than intended. The reliability won't be lost, then, just the cost in terms of the number of messages will be slightly higher in this particular hop.

There is still a potential issue during splits. When our neighbour (let's call it Y) splits, one of the two resulting sections (cal them Y0 and Y1) might not be our neighbour anymore. Say Y0 is our neighbour after the split, and Y1 isn't. With Node Ageing in place, some of the Elders of Y will end up in Y0, and some will end up in Y1. In an edge case scenario, all Elders from Y will end up in Y1. Y0 will then have new Elders, but it will take some time for them to connect to us. If the Elders of Y1 disconnect right after the split, there will be a period when we aren't connected to anyone in the address space that was managed by Y, thus breaking our assumption.

In order to prevent this issue from occurring, Routing will have to ensure that every section's Elders are connected to at least 1/3 of Elders in each of its neighbouring sections at all times - so Elders in a section that split won't be able to just immediately disconnect from their former neighbours and break their ability to relay messages.

## Drawbacks

This RFC increases the burden on the Network in terms of the number of messages being passed between nodes. It seems to be an acceptable increase, however.

## Alternatives

An alternative is to use message routes and selecting a single node to route a message to the next section via an Ack mechanism. This is generally less reliable and possibly slower than this proposal, due to the necessity of waiting for timeouts.

## Unresolved questions

What size of Elder group is secure? This is critical to prevent an exponential increase in per hop messages.
