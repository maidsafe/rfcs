# Transparent Malice Handling within PARSEC
- Status: agreed
- Type: new feature
- Related components: PARSEC, Routing
- Start Date: 21-09-2018
- Discussion: https://safenetforum.org/t/rfc-50-transparent-malice-handling-within-parsec/25487
- Supersedes: N/A
- Superseded by: N/A

## Summary
[summary]: #summary
In this RFC, we propose methods of identifying and dealing with nodes acting with differing types of malicious intent within a section. When the section identifies and reaches consensus that a particular node is a malicious agent then that node is then ejected from the section.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
[motivation]: #motivation
By design, PARSEC is resilient to a third minus one malicious nodes in a section. Controlling that many nodes would allow the adversary to stall consensus in that section.
Because of sybil attack mitigations from routing, such as node ageing and secure random rellocation, as well as incentives to behave such as farming rate, we know that attempts to control a section will be prohibiteively expensive in a large network. However, the right way to ensure security in a system is to add protections at every possible layer.
This RFC proposes a way to identify a variety of malicious behaviours in PARSEC so that offending nodes can be rejected from a section immediately. This means that the adversary will not only need to burn an enormous amount of resources to be able to misbehave, but also that they will have to synchronise their attack with an excellent precision as the first indication of misbehaving will cause each of their node to be rejected.

## Detailed design
[design]: #design

### Defining Malice
A malicious node is defined as a node that is operating outside of normal node behaviour.

### Handling Malice with PARSEC through transparency
[handling]: #handling
#### Context
Prior to milestone 2 Parsec was unaware of `NetworkEvent` contents. This includes votes for add_peer and remove_peer. It benefits Parsec to be able to view via its own graph when these types of network events become stable from each of its peers' perspectives. That allows us to detect more types of malice, and also to react to malice quicker. By having Parsec able view the add and remove votes, and also by requiring Parsec to cast votes about malicious behaviour, the graph allows us to definitively handle cases which would be difficult or impossible otherwise. Let's take an example where Parsec is unaware of the contents of all `NetworkEvent`s, and instead needs to be told by Routing (via an API method like `Parsec::add_peer(id)`) once a block becomes stable. In the example, we also say that Carol is malicious and sends us a message containing a bunch of gossip events from Bob, even though she hadn't actually been told to add Bob at the point when she recorded a sync request from him. We can't tell from the gossip graph that this is the case. We can't use the fact that `add_peer(Bob)` is or isn't stable for _us_ to deduce whether that's true for Carol too. However, if we can see the actual `add_peer(Bob)` votes in the graph, we can definitively tell when looking at Carol's sync-request event whether that block for add Bob had become stable for her or not (in this case, we're saying it hadn't). That means we're able to say that she shouldn't have accepted any sync requests/responses from Bob before that point, and so it's malicious behaviour.

#### Network Event
We then define `NetworkEvent` as an enum holding the parsec related operations. The causes of malice are defined as a `Malice` enum.

```rust
enum Observation<T: NetworkEvent> {
    Add(PeerId),
    Remove(PeerId),
    Accusation {
        offender: PeerId,
        malice: Malice,
    },
    OpaquePayload(T),
}

enum Malice {
    // the common self_parent of the fork
    Fork(Hash),
    // hash of the malice that was not reported (will be bundled inside same vec)
    Accomplice(Hash),
    // hash of the gossip event containing an invalid accusation
    InvalidAccusation(Hash),
    // hash of the gossip event from a creator that this peer shouldn't trust
    InvalidGossipCreator(Hash),
    // hash of a gossip event for which the other_parent is older than its
    // self_parent's oldest ancestor by the same node
    StaleOtherParent(Hash),
    // hashes of both gossip events by the same creator that carry the same vote
    DuplicateVotes(Hash, Hash),
    // hash of a gossip event which has for cause: Observation::Genesis but shouldn't
    UnexpectedGenesis(Hash),
    // hash of the gossip event that contains a genesis member list that doesn't
    // match consensus
    IncorrectGenesis(Hash),
    // hash of the gossip event that should carry an Observation::Genesis but doesn't
    MissingGenesis(Hash),
    // the gossip event in question
    SelfParentByDifferentCreator(Event),
    // the gossip event in question
    OtherParentBySameCreator(Event),
    ...
}
```
#### On detecting malice
When handling a sync request or response, if an honest node detects malice, it must create a gossip event immediately (after creating a sync event to record the communication that showed malice, but before creating its next sync event) to cast its accusation against the suspicious peer. This will be an `Observation::Accusation` event specifying the type of malice, and as with all observations, will only be acted upon once it has reached consensus among the peers. Each separate instance of malice will be recorded in a single `Accusation` event, so there may be multiple such accusations immediately after the sync event recording receipt of the communication. When we detect a malice that was not flagged by the sender (before its next sync event), we must raise a `Malice(Accomplice(that peer))` accusation to accuse the sender. (Note: according to the event insertion order, the accusation event will be inserted *after* the detection of the malice. This needs to be considered to avoid accusing an innocent sender.)

On seeing a vote for `Malice(...)`, if proof checks out (i.e. we can spot the claimed malice in the graph as well), we must create a gossip event with same observation when we have not done so yet. When we see a `Malice` accusation that was incorrectly flagged, we must raise a `Malice(Liar(PeerId))` accusation against the accuser.

All gossip events, including those being detected as malice, will be added into the graph and be shared with others. The signature of `handle_request` and `handle_response` functions then remains unchanged, as more gossip is just simply added. When Parsec eventually gets consensus on a Malice event, it is given to Routing via `poll()` as with any other `NetworkEvent`.
- Consensus on `Remove` leads to removal from PeerList.
- Consensus on `Add` leads to addition to PeerList.
- Consensus on `Malice` leads to removal from PeerList.

Note that we considered being more expeditive and removing nodes from our PeerList as soon as malice is observed (as opposed to once consensus is reached). We didn't pursue that idea because it could lead to a situation where a fork may be used to give different nodes a temporary disagreement on section membership. We think this probably breaches our proofs as these kinds of forks would be fundamentally different from the forks in static membership that our proofs already consider. By waiting for consensus before acting on Malice, we are certain that the pre-conditions for our proofs in a static network are maintained.

### Malice types
[malicetypes]: #malicetypes

The following categories of malice have been identified:
- Type safety
- Malice that can be proved with the gossip graph
- Malice that can be proved with a single gossip event
- Detectable but not provable only with the gossip graph (needs routing)
- Detectable but not provable (my word against theirs)
- Pure routing handling
- Impossible attacks

Each of the main types of malice have additional sub-strains that must be considered.

#### Type safety
[typesafety]: #typesafety
Here, we list malice that can be handled for free at compile time thanks to type safety.

##### Initial event with a `self_parent`
We can address this issue the same way we dealt with `other_parent`: move the `self_parent` from `Content` into the `Cause` enum so that it is impossible to create an event with Cause initial and non-None self_parent.

#### Malice that can be proved with the gossip graph
[gossipgraph]: #gossipgraph

##### Fork
A peer creates two gossip events with the same `self_parent`. Note that this example is pretty obvious to detect as Alice gossiped both sides of the fork to Bob. There can be more subtle cases where different nodes inform Bob of each side of the fork. We provide the `self_parent` in the accusation in order to reach consensus, despite this being a little harder for a recipient to find the actual forked events (`a_1` and `a_2` in the example below). If we provided the actual forked events in the accusation, then to get accumulation, we would have to make accusations for each possible pair of forks. Given that a malicious node can create many forks of a single parent event, this would be a costly approach.

![Fork](https://s20.postimg.cc/6qi05v1xp/Type_Safety_01_Fork.png)

##### Accomplice
A node fails to report malice that it has observed.

![Accomplice](https://s20.postimg.cc/xobx7n4l9/Type_Safety_02_Accomplice.png)

##### InvalidAccusation
A node incorrectly reports another node misbehaved.

![InvalidAccusation](https://s20.postimg.cc/crfp2xw9p/Type_Safety_03_Invalid_Accusation.png)

##### InvalidGossipCreator
A node reports gossip from another node that isn't in their section. Note: For determining whether a node (say Frank) is in another node's (say Alice's) section, here are the rules:
- if Alice's gossip graph never reached consensus on any event, consider the `BTreeSet` contained in her first observation (`Observation::Genesis<BTreeSet<PeerId>>`) as her section membership list.
- if any event was consensused, the genesis list is the first consensused event, so start from there and consider any subsequent consensused `Add` or `Remove` as an addition or removal of said peer

Note: we discussed the merits of storing the event from an unknown creatorsd(i.e: `bbw_0`) in the gossip graph versus letting routing convince other nodes through backchannels + sending the sync.

The drawback of this approach is a potential added cost in storing the event that we know is malicious, but the advantage is that it is much easier to prove that Alice didn't know `bbw` at the time if we can add her sync message to the graph.

We think that this approach is better. If and when pruning is added, it could potentially deal with the concern of space complexity explosion by malicious nodes.

![InvalidGossipCreator](https://s20.postimg.cc/azmq827rx/Type_Safety_04_Invalid_Gossip_Creator.png)

##### StaleOtherParent
A gossip event with `other_parent` older than first ancestor of `self_parent` by the same node (for instance, Bob can create this situation when they create `b_1`)

![StaleOtherParent](https://s20.postimg.cc/bp5ikel65/Type_Safety_05_Stale_Other_Parent.png)

##### DuplicateVotes
The same node creates more than one gossip event carrying the same vote.
Note: if a node has more than 2 duplicate votes, only report the hashes of the 2 oldest such votes to avoid wasting effort.

![DuplicateVotes](https://s20.postimg.cc/c1wwqlb5p/Type_Safety_06_Duplicate_Votes.png)

##### UnexpectedGenesis
If any event carries an `Observation::Genesis`, but shouldn't. This could be due to any of 2 reasons:
- This node is not a member of the consensused genesis group
- This node is a member of the consensused genesis group but the gossip event's self_parent doesn't have `Cause::Initial`.
Note: This has the side effect of taking care of possible problems such as duplicate votes for different genesis groups

![UnexpectedGenesis](https://s20.postimg.cc/c1wwqlyb1/Type_Safety_07_Unexpected_Genesis.png)

##### IncorrectGenesis
A node creates an event with `Observation::Genesis`. Their observation is in disagreement with my consensus list (either my `Genesis::Observation` or the genesis being consensused if I was added after genesis). For example, here, Bob will accuse `Alice` of `Malice::IncorrectGenesis(a_1)` as her Genesis member list doesn't match his.

![IncorrectGenesis](https://s20.postimg.cc/w9aciwgct/Type_Safety_08_Incorrect_Genesis.png)

##### MissingGenesis
A node that is part of the genesis group creates an event with no `Observation::Genesis` immediately after its initial event. For example, here, Bob will accuse `Alice` of `Malice::MissingGenesis(a_1)` as her Genesis member list doesn't match his.

![MissingGenesis](https://s20.postimg.cc/spoet48i5/Type_Safety_09_Missing_Genesis.png)

#### Malice that can be proved with a single gossip event
[gossipevent]: #gossipevent
These are forms of malice where we don't want to add the gossip event to our graph as it would possibly complicate analysis of the graph significantly. Instead, we will send the entire gossip event that can be used to prove malice in our accusation.

##### SelfParentByDifferentCreator
A malformed gossip event for which the `self_parent` has a different creator from the gossip event in question.

##### OtherParentBySameCreator
A malformed gossip event for which the `other_parent` has the same creator as the gossip event in question.

#### Detectable but not provable only with the gossip graph (needs routing)
[detectablenotprovablegraph]: #detectablenotprovablegraph
We will need to involve routing.

`handle_request` returns a `Result<Response<_>, Error>>`
`handle_response` return a `Result<(), Error>`

```rust
enum GossipError {
    // hash of the gossip event that's got an invalid self_parent
    InvalidSelfParent(Hash),
    // hash of the gossip event that's got an invalid other_parent
    InvalidOtherParent(Hash),
    // hash of the gossip event with a signature that is not its creator's signature
    InvalidSignature(Hash),
    InvalidResponse,
    // they sent more events than they should have
    Spam(Option<Response>),
    // they sent gossip to us before knowing we could handle it
    PrematureGossip
    ...
}

/// On receiving these,
/// routing can communicate this to other nodes (through backchannels)
struct MaliciousMessage {
    kind: GossipError,
    message: SignedMessage,
}
```
##### InvalidSelfParent
The hash of the `self_parent` doesn't point to any known gossip event

##### InvalidOtherParent
The hash of the `other_parent` doesn't point to any known gossip event

##### InvalidSignature
An event is received but the Signature doesn't match the creator. It can't simply be used as proof of malice without the entire request as it would be impossible to distinguish an invalid accusations from a real one

##### InvalidResponse
`handle_response` is called with events such that the latest event doesn't have `Cause::Request` (which means that the author didn't follow the request/response protocol).

The response may not be timely (right after the request)

![InvalidResponse1](https://s20.postimg.cc/vz2tzo4cd/Invalid_Response1.png)

A response could be responding to no request.

![InvalidResponse2](https://s20.postimg.cc/3m7c97awd/Invalid_Response2.png)

##### Duplicate responses
If we receive the same response more than once, we can fail and inform routing (b_2 wouldn't actually be added to the graph in this case)

![DuplicateResponses](https://s20.postimg.cc/lp0f0ez19/Duplicate_Responses.png)

##### Spam
A node sends us info that we know they know we already know Note: in that case, we want for routing to be aware that `Alice` behaved maliciously but we also want to create a sync event and send a `Response` to `Alice` (so we don't look malicious ourselves or lose important data). We achieve this by returning an error that contains the `Response` that we would have returned if there had been no spam.

![Spam](https://s20.postimg.cc/hsn34g171/Spam.png)

##### PrematureGossip
Some node contacts us before we are able to properly handle their gossip. This can be of two forms:
- They send us communication but their latest gossip event betrays that we are not in their routing table (`AddPeer(Us)` was not consensused as far as they know)
    - Note: this covers this particular case:
        - They send us communication where the initial gossip event is followed by an `Event` with `Cause::Response` before an `Event` with `Cause::Request`
            - This allows for events with `Cause::Observation` to be added, but the `Request`/`Response` pattern to be good enough to prove non-spammy behaviour

#### Detectable but not provable (my word against theirs)
[detectablenotprovableword]: #detectablenotprovableword
If a peer can convince himself that another peer is malicious but doesn't have sufficient evidence to convince other nodes that that peer is malicious, PARSEC can still let the routing layer know of the issue. Routing can then call `vote_for` with a `payload` of `UnprovableAccusation`. Unlike some other accusations that can be proven to other peers, we will need to accumulate a super-majority of `UnprovableAccusation` without having the opportunity to echo an accusation on the face of it. If consensus is reached on `UnprovableAccusation(peer)`, it means that the malicious node personally offended a super-majority of other nodes, so it is fair to kick them out (routing's decision).

```rust
enum GossipError {
    // See above for variants that routing can prove
    ...
    // variants that cannot be proven to others
    MissingResponse,
...
}
struct UnprovableAccusation {
    accused: PeerId,
}
```

##### MissingResponse
If I send a request to someone and they come back to me with another response, I know they acted maliciously. It's my word against theirs though, as nothing separates this situation from my hiding some of their gossip events (as I create the sync event that records their response).

#### Pure routing handling

##### Junk
deserialised bytes A message is received which doesn't de-serialize to anything meaningful.

#### Impossible attacks
##### Cyclic graph
We considered the idea of nodes colluding to create a cycle in the graph in an attempt to undermine our algorithms.
We think it is impossible to create such a situation as `Alice` would need to generate a hash for its `other_parent` that depends on its own event hash which is like a proof of work with maximum difficulty (256 bits in our case). Cycle (would need collusion... and infinite computing power).

![Cyclic graph](https://s20.postimg.cc/apf7ot0wd/Impossible_Cyclic.png)

### Notes
Each node keeps a section membership list from each other node's point of view (in peer manager). This allows detection of specific malice that relays messages from peers that the offender shouldn't trust. It is thought that the computation complexity of detecting this malice is constant (assuming this section membership list is a Hash collection with O(1) access).

## Drawbacks
[drawbacks]: #drawbacks
- ?

## Alternatives
[alternatives]: #alternatives
- ?
## Unresolved questions
[unresolved]: #unresolved
- Can we also deal with types of malice that require a stronger synchrony assumption to detect? (For instance not responding or sending disproportionately more responses than requests)
