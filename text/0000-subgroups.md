# Title-cased Subgroups

- Status: proposed
- Related components: routing, disjoint groups
- Start Date: 2016-11-04
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this) 

## Summary

Allow sub-prefixes within a group to be managed by a sub-set of group nodes which
themselves satisfy group requirements.

## Motivation

With the current disjoint groups proposal, nodes dropping from a group may
cause a group merge to occur, which suddenly causes all members of the merged
group to have to manage all data of the new larger group, increasing workload
(in terms of responses, data storage and synchronisation).

Worse, the increased work may cause nodes to drop out (e.g. if they cannot handle
all the new data they are expected to store — we don't currently have a way of letting
nodes contribute in this case). This may cause another merge, and could trigger
a cascade trying to merge the entire network. (This RFC is not a complete solution
to this issue.)

## Detailed design

Currently, a group has one prefix denoting addresses (`XorName`s) which its nodes
store data for. This RFC would change this, such that:

1.  There would still be one prefix denoting all addresses managed by the group
2.  The group may have any number of sub-prefixes, matching its root prefix
3.  Each prefix must match enough nodes in the group to themselves satisfy group
    requirements

### Prefixes

Assignation of prefixes to nodes should remain fairly simple.

On merge, all nodes keep their old prefixes but also get the new group prefix.

On split, nodes lose any prefixes which are not contained in their new group
prefix.

When a node joins, it takes all prefixes currently used by the group; further,
new prefixes matching the node's address may be created if enough nodes in
the group match the new prefix to satisfy group requirements. All matching nodes
get any new prefixes created.

When a node leaves, if any of its prefixes no longer satisfy group requirements,
they are dropped by all nodes.

### Responsibilities

Nodes are responsible for all addresses matching the group prefix which do not
match a prefix not matching the node's own name.

Example: group has prefixes `00, 001, 0011`. Then nodes matching `0011` are
responsible for all addresses matching `00`. Nodes matching `0010` are
responsible for addresses matching `0010` but not `0011`. Nodes matching
`000` are responsible only for addresses matching `000`.

## Advantages

When a merge is required, nodes within the smaller half (which lost consensus)
are no longer required to grab all data matching the group they merge with,
which reduces network load.

Futher, if a cascade occurs, the address space which nodes need to cover does
not grow exponentially. For example, given groups `000, 001, 010, 011`, if group
`000` fails, firstly a merge will occur producing `00, 010, 011`. If another merge
occurs, these groups merge to form `0`, however, this group has sub-prefixes `001`,
`01`, `010` and `011` which are themselves valid groups, thus no nodes under `0`
need to cover _all_ of this group's address space. (Without this RFC, all nodes
under `0` would be expected to cover all addresses under `0`. If this proved
impossible for enough of the nodes, they could drop out causing another merge,
which could cascade to take down the entire network.)

## Drawbacks

Routing is _already_ complicated. Further, this doesn't entirely solve node-loss
problems, though it does reduce implied work-load and chance of cascade failure.

## Extensions

### Assigned responsibilities

During a merge, this proposal still requires all members of one of the old
groups to assume responsibility over at least part of the new address space.
If some of these nodes were close to capacity, they may now be overwhelmed.
Example: `00` merges with `01` because `00` no longer had enough nodes; now
all of `01` are responsible for all of `0`.

An extension might be to assign only _some_ nodes responsibility over an address
space not covered by a sub-prefix; for example, in the above merge, allocate only
enough nodes of `01` to `00` to satisfy the group requirements. This could be
done on an opt-out basis: choose nodes randomly, but let them decline, and only
force if no other option exists. It should not be done on a volunteer or
resource/performance count basis since this would allow dishonest nodes to
grab extra responsibilities.

### Group size

The current *minimum group size* requirement only looks at one side of an issue.
Possibly requirements could be made more specific by defining minumum numbers
of nodes for several different purposes:

*   the minimum required to create a new (sub)group or split
*   the minimum required to prevent merging
*   the minimum required for data storage, before neighbouring nodes are recruited
    to replicate data

## Alternatives

### Frozen addresses

Another option for dealing with prefixes (or sub-prefixes) without enough nodes
would be to _freeze_ the address space. _Get_ requests for immutable data
(which verifies itself) could still succeed assuming some node answers the request,
but _put_ requests and _get_ requests for mutable data would fail.

This has the advantage of preventing merge-cascade-failure of the network, but
two disadvantages: temporary service failure and a chance that data does not get
replicated to new nodes before all nodes are lost.

## Unresolved questions

What parts of the design are still to be done?
