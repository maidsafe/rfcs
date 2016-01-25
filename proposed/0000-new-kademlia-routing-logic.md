- Feature Name: new_kademlia_routing_logic
- Type enhancement
- Related components kademlia_routing_table, routing
- Start Date: 23-01-2016
- RFC PR:
- Issue number:


# Summary

The current mechanism implemented in the [kademlia_routing_table][1] and
[routing][2] crates has several weaknesses that lead to potential problems
affecting security, performance and even basic functionality.

[1]: https://github.com/maidsafe/kademlia_routing_table
[2]: https://github.com/maidsafe/routing


# Motivation

In the current implementation,

* routing table entries are symmetric, i. e. I can't be in your table if you are
  not in mine,
* the routing crate does not add enough connections to a new nodes' table to
  ensure the [invariant required for Kademlia](http://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)
  (3rd paragraph in "System description"),
* the `kademlia_routing_table::RoutingTable`'s methods to choose desired entries
  do not ensure the invariant, even if presented with enough potential entries,
* not even the original Kademlia invariant is strong enough to guarantee that
  messages reach every member of a target's close group, or that every node
  knows whether it is a member of that group,
* the parallelism feature makes the upper bound for the number of hop messages
  that are sent for a single message grow linearly with the network size,
  instead of logarithmically,
* the parallelism feature causes hop messages to be sent to messages outside the
  target's bucket, which does not reduce the estimate for the remaining number
  of hops to the target.

Some of these issues have already been demonstrated in tests: Both the routing
crate's ci_test and deployed test networks have shown that the routing tables
are insufficiently filled, and
[tests for the kademlia_routing_table](https://github.com/afck/kademlia_routing_table/blob/network_tests/src/lib.rs#L1308)
crate show that even under ideal conditions, messages do not always reach their
target and the number of nodes that consider themselves members of a particular
close group can exceed 100 - orders of magnitude above the current group size of
8.

Of course a peer-to-peer network has to deal with lots of unpredictable
problems, like malicious nodes, simultaneously joining or leaving nodes,
connection failures, etc., so no absolute guarantees can be made. However,
we can expect to have proof that at least in the most common scenarios (e. g.
one node leaves, then there is some time to adapt to the change, then a message
is sent) the routing mechanism *does* make the guarantees that its users rely
on.

Instead of detailing how and where the above points fail, this is a proposal to
make some changes to the routing logic, together with a rigorous argument why
this will make the following guarantees in common scenarios:

1. The number of nodes in the network with `node.is_close(target) == true` is
   exactly `GROUP_SIZE` for each target address.
2. Each node in a given address' close group is connected to each other node in
   that group.
3. Each message reaches every member of the destination's close group after at
   most 512 hops.
4. The number of total hop messages created for each message is at most
   `PARALLELISM * 512`.
5. For each node there are at most 512 * `GROUP_SIZE` other nodes in the network
   for which it can obtain the IP address, at any point in time.

**TODO**: We should add more guarantees here, e. g.: If `PARALLELISM - 1` nodes
suddenly go offline and routing tables haven't been updated yet, messages still
reach their destination. Or: If two nodes are unable to directly connect to each
other, ... etc.


# Detailed design

The network will attempt to always satisfy the following invariant:


## The invariant

In every node, every bucket is maximally filled, and every node knows its close
group. That means:

Whenever a bucket has less than `BUCKET_SIZE` entries, it contains *all nodes in
the network* that have the corresponding bucket distance. And the closest
`GROUP_SIZE - 1` nodes are always in a node's routing table.


## Changes to the routing logic

In kademlia_routing_table:

* The constants are changed and a test asserts that `BUCKET_SIZE == GROUP_SIZE`
  and `BUCKET_SIZE >= PARALLELISM`. (Possibly remove `BUCKET_SIZE`.)
* The `add_node` and `want_to_add` methods are modified so that we always
  add/want a node if its bucket does not yet have `BUCKET_SIZE` entries.
* The `target_nodes` function is modified (and used in routing accordingly) so
  that only the source node of a message sends `PARALLELISM` copies of it.
* Apart from that, each node relays every copy of the message that it receives,
  and up to `PARALLELISM` copies.
* As an exception to the previous point: Any message sent to a group that the
  current node belongs to is relayed to the `GROUP_SIZE - 1` nodes closest to
  the target.
* The `is_close` method returns whether there are fewer than `GROUP_SIZE` nodes
  in the routing table which are closer to the target than `self`.

In routing:

* I am allowed to connect to you (and learn about your IP address) if and only
  if you are in the close group of one of my *bucket address*es: an address that
  differs from mine in exactly one bit, (independent of whether you want me in
  your routing table).
* Whenever a node joins or leaves, all routing tables are updated so that the
  invariant is satisfied. (See below for details.)

See the appendix below for proofs that the invariant and these changes will
imply the required guarantees.


## Node insertion

TBD


## Node removal

TBD


# Drawbacks

The expected routing table size for a network with 2<sup>n</sup> nodes will be
about `GROUP_SIZE * n`. Each of these entries currently corresponds to at least
one open connection and two threads. It would therefore be desirable to reduce
the routing table size.


# Alternatives

TBD


# Unresolved questions

Although it can be proved that the closest node to the destination is reached by
a message, there is no proof yet that *every* member of the close group will be
reached.


# Appendix: Proofs

We prove that whenever the invariant holds, the desired properties are
guaranteed.

Let `bd(x, y)` be the bucket distance between two addresses `x` and `y`, `x ^ y`
the XOR distance, and `bi(x, y) = 512 - bd(x, y)` the bucket index.
Equivalently, the `bi(x, y)`-th bit is the first (most significant) one where
`x` and `y` disagree.


### Lemma 1

`y` is XOR-closer to `x` than `z` if and only if `y` agrees with `x` in the most
significant bit where `y` disagrees with `z`. That is, the following are
equivalent:

* `x ^ y < x ^ z`
* `x` and `y` agree in the `bi(y, z)`-th bit.
* `x` and `z` disagree in the `bi(y, z)`-th bit.

**Proof:**
`x ^ y < x ^ z` means that in the most significant bit where `x ^ y` and
`x ^ z` disagree, `x ^ y` has a 0. But `x ^ y` and `x ^ z` disagree in the same
bits as `y` and `z`, i. e. they first disagree in the `bi(y, z)`-th bit.
Since `x ^ y` has a 0 there, that means that `x` and `y` agree there.
Similarly, since `x ^ z` is 1 in that place, `x` and `z` disagree there.


### Lemma 2

If `n` is in the close group to `d`, it has every node `m` in its routing table
which is *even closer* to `d`.

**Proof:**
By Lemma 1, `m` is closer to `d` if and only if `d` and `n` agree in the
`bi(m, n)`-th digit, i. e. if `m` would belong in a bucket `i` of `n` such that
`d ^ n` has a `0` in the `i`-th position. In other words, the nodes closer to
`d` than `n` are exactly those which belong in such a bucket. If there were
`GROUP_SIZE` of them, `n` wouldn't be close to `d`, so there are less than
`GROUP_SIZE` of them, which means none of these buckets is full. Therefore
`m` is actually in one of those buckets of `n` because of the invariant.


### Property 1

If a node is among the `GROUP_SIZE` closest nodes in the network, it cannot have
`GROUP_SIZE` other, closer nodes in its routing table. Therefore `is_close`
returns `true`.

For the converse, assume there are `GROUP_SIZE` nodes that are closer to the
target `t` than our node's address `n`. That such a node `c` is closer to `t`
means `t ^ c < t ^ n`, which by Lemma 1 is equivalent to `c` belonging in the
`i`-th bucket of `n` for some `i` such that `n` and `t` disagree in the `i`-th
bit. Since by the invariant each such bucket contains either *all* nodes with
that bucket distance or `GROUP_SIZE` such nodes, the routing table then has at
least `GROUP_SIZE` such entries `c`.


### Property 2

Let `n` and `m` be in the close group of `d`. Witout loss of generality assume
that `n` is closer to `d`. Then by Lemma 2, `m` has `n` in its routing table.
Therefore, the two are connected.


### Property 3

Thanks to property 2, we only have to show that the message reaches the node
closest to the destination `d`, as from there it will directly be sent to all
the other close nodes.

For that, we show that every node `n` that is not closest to `d` increases the
number of *good* leading bits in the next hop:

A bit `i` is *good* for `n` if either `n ^ x` is 0 in that bit, or `n`'s `i`-th
bucket is empty.

Let the current node `n` not be closest to the destination `d`, and assume that
the first `i` bits are good. The message is sent to the entry `m` in `n`'s
routing table which is closest to `d`, so we need to prove that `m` has at least
`i + 1` good leading bits.

Since `m` minimizes the distance to `d` among the table entries, it must be in
the first nonempty bucket `k` of `n` such that `n` and `d` disagree in the
`k`-th bit. Thus `k`-th is the first bit that is not good for `n`, that is,
`k = i + 1`. Since `m` and `d` agree there, it is, however, good for `m`.

If we can show that the first `i` bits are also still good for `m`, then it
follows that `m` has in fact at least `i + 1` good leading bits:

But `m` and `n` agree in the first `i` bits. So wherever `n ^ x` has a 0,
`m ^ x` also does. Also, the nodes with bucket distance `j` for every `j <= i`
are the same, so whenever `n`'s `j`-th bucket is empty, the invariant implies
that there are no nodes with that bucket distance in the network and therefore
`m`'s `j`-th bucket must also be empty.


### Property 4

This follows immediately from Property 2, since only `PARALLELISM` different
messages are created by the sender.


### Property 5

There are `512` bucket addresses, and each of them has only `GROUP_SIZE` close
nodes.
