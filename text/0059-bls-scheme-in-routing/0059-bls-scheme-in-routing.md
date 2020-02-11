# Boneh-Lynn-Shacham scheme in Routing

- Status: active
- Type: enhancement
- Related components: routing, PARSEC
- Start Date: 06-06-2019
- Discussion: https://safenetforum.org/t/rfc-59-boneh-lynn-shacham-scheme-in-routing/28858
- Supersedes: None
- Superseded by: None

## Summary

This RFC describes a proposal to integrate Boneh-Lynn-Shacham cryptographic scheme into Routing in order to enable taking advantage of its threshold-cryptographic capabilities.

## Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

## Motivation

Routing has multiple applications in which signing a piece of data by multiple nodes is required. This is realised currently by collecting multiple signatures from the nodes and checking the number of signatures against a predefined quorum. This means, however, that when such a signed piece of data needs to be transmitted somewhere else in the network, all the signatures need to be transmitted as well, along with the proof that each of the corresponding public keys belongs to a legitimate node.

Using a threshold cryptography scheme will provide the same functionality, but with a greatly reduced overhead. Instead of multiple signatures, only one signature can be attached to a piece of data, corresponding to a single aggregate public key belonging to a section as a whole. Then, instead of a proof for every member's public key, only a proof for the single section's public key will be enough.

Another nice feature of threshold signature is that the keys used for signing are changed every time the set of members changes. This means that votes cast for an event that happened when the set of members was `A` cannot be reused for an event that happened when the set of members was `B`. Thus, we become more resilient against replay attacks and rogue key attacks.

A threshold cryptography scheme will also be a crucial component of migrating PARSEC to use a common coin.

## Assumptions

We assume that the network is divided into sections, with each section running its own PARSEC instance. In particular, we don't assume anything about whether all the nodes in a section participate in PARSEC, or just some of them (the Elders), so the design should be applicable both before and after the implementation of Node Ageing.

However, since most of the design will only concern the nodes participating in PARSEC, we will call them "Elders" for the sake of brevity. We can just assume that before the implementation of Node Ageing, all section members can be considered Elders.

Also, like in RFCs 56 and 58, we assume no merges. This assumption may be lifted in the future and the proposal expanded to accommodate merges as well.

## Detailed design

### General overview

1. All Elders hold a Bonneh-Lynn-Shacham (BLS) keypair, in addition to the existing ED25519 keypair.
2. The BLS keypairs held by the Elders are shares of an aggregate section keypair with threshold `t` (which means that `t+1` shares would be required to construct a full signature).
3. The BLS keypairs are regenerated every time the set of members of the section changes, or when the section prefix changes. The keys are generated with a Distributed Key Generation (DKG) algorithm.
4. Whenever the section needs to collectively sign a piece of data, nodes input their signature shares as votes into PARSEC. When enough signature shares reach consensus, any node that reaches this point in PARSEC will be able to construct a full section signature and send it to the destination.

### Elders' state

As mentioned above, every Elder will need to keep additional state: a `SecretKeyShare` and a `PublicKeySet` (`PublicKeySet` being a type from the `threshold_crypto` crate that represents the set of public keys used in the scheme - it can be used to retrieve both individual nodes' public keys, as well as the aggregate section's public key).

The exact place in the code that will store this state remains an implementation detail. It will probably eventually be stored inside PARSEC as a part of the common coin mechanism.

### Distributed Key Generation

DKG shall be an important part of implementing the BLS scheme in Routing, as it will be the mechanism through which the Elders will be obtaining their keys. Its most important feature is the lack of a trusted dealer - such a dealer always has access to all the shares at some point and thus could single-handedly act on behalf of the section if it was malicious. Using a DKG algorithm prevents this vulnerability. We will use the DKG algorithm presented in [POA Network's HBBFT repository](https://github.com/poanetwork/hbbft/issues/47\#issuecomment-394422248).

The DKG algorithm is executed by two types of nodes, which we will call "participants" and "observers". The participants end up with both a `SecretKeyShare` and the `PublicKeySet`; the observers can only obtain the `PublicKeySet`.

Let us denote the maximum number of faulty nodes in a section by `f`. From the properties of PARSEC, `f` MUST be less than a third of the Elders. Let us denote the number of Elders by `N`, then `N > 3f`. Actually, in order to have the highest tolerance possible, we choose `N = 3f + 1`.

In order for the DKG algorithm to finish, a node must collect at least `t + 1` valid `DkgAck`s per `DkgPart` for at least `t + 1` `DkgPart`s and be certain that other nodes will also have at least `t + 1` valid `DkgAck`s per `DkgPart` at this point, too. Since a `DkgAck` message consists of parts encrypted with different nodes' keys, a single node can never know if parts encrypted for different nodes are valid. Hence, the only way to be sure that every node will have at least `t + 1` valid `DkgAck`s among a common set is for the set to contain `f + t + 1` messages, as at most `f` of them can be invalid. Since every participant can only send one `DkgAck` per `DkgPart` and only participants can send `DkgAck`s, this also means that DKG requires at least `f + t + 1` active participants.

However, `f` faulty nodes can just not send any messages at all. Thus, the algorithm has to be able to terminate once `N - f` messages are collected. This means that `f + t + 1` must be less than or equal to `N - f`, or: `t ≤ N - 2f - 1`, which, together with our choice of `N = 3f + 1`, implies `t ≤ f`. `t < f` is unsafe, because then the faulty nodes could create valid signatures by themselves, so we choose `t = f`.

The above is a change relative to the pre-BLS behaviour. Before, we required more than 2/3 of the section to sign a message before it was valid. `t = f` means that more than 1/3 of the section will be enough after implementing BLS.

Since the algorithm is synchronous in nature, all messages that pertain to it MUST be sent as PARSEC observations. PARSEC will then ensure a common order of these messages, which will guarantee that all participants and observers will end up with correct data.

The outline of the algorithm is as follows:

1. Every participant publishes a `DkgPart` message as a PARSEC observation.
2. Whenever a `DkgPart` message reaches consensus, participants publish a `DkgAck` message as a PARSEC observation.
3. Once `f + t + 1` `DkgAck` messages per `DkgPart` for at least `t + 1` `DkgPart`s reach consensus, the algorithm will finish. Participants will then be able to generate their `SecretKeyShare`s, and both participants and observers will be able to calculate the `PublicKeySet`.

In the last point, it is important that all the nodes use the same set of `t + 1` `DkgPart`s. This is guaranteed by PARSEC - nodes will just use the first `t + 1` ones that got `f + t + 1` valid `DkgAck`s, and due to PARSEC providing a common ordering, they will be the same messages for every participant.

`DkgPart` and `DkgAck` messages both contain payloads encrypted with other nodes' public keys. These public keys don't have to be BLS keys - any encryption algorithm can be used, and we intend to use the ED25519 keys for this purpose, if possible.

#### The set of DKG participants

Since the participants are the nodes that obtain `SecretKeyShare`s, the set of participants MUST be the updated set of the members of the section. That is, if a new node becomes an Elder, the set of participants MUST be the set of the old Elders plus the new one.

The nodes that become Elders SHOULD be able to actively participate in the algorithm, but this is not required - as long as there are at least `f + t + 1` active Elders among the old set of Elders, the algorithm will finish correctly. However, we won't have such a guarantee once Node Ageing is in place - then, during a split, the set of Elders of one of the resulting sections might have no nodes in common with the set of Elders of the splitting section. This means that once Node Ageing is implemented, the set of participants MUST be the new set of Elders. Satisfying this requirement will require modifications to PARSEC, as currently nodes that are not Elders are not allowed to gossip nor vote (create observations), and these will be necessary to enable their participation in DKG.

#### Trusted dealer - a clarification

Even though the DKG algorithm itself doesn't require a trusted dealer, the first node in the network is effectively a trusted dealer for itself. Until the modifications that allow joining nodes to participate are in place, it will also effectively be a trusted dealer when the second node joins. However, once the network gains more nodes and there are at least three of them, the new keys won't be generated single-handedly by anyone and the need to trust the first node will have disappeared.

### Updating the keys

Whenever the DKG is complete and a new set of keys is generated, the Elders SHALL use their old keys to sign the new aggregate public key - in case of splits, both of the new public keys. This is a requirement of RFC 56. The signature shares SHALL be input into PARSEC, and only once enough shares reach consensus for the signature to be constructed, will the Elders discard their old BLS keys.

The signature along with the new section's public key SHALL be appended to the `SectionProofChain` in the shared state (see RFC 56).

### Creating signatures

Whenever a need arises for the section to collectively sign a piece of data, the Elders SHALL input their signature shares into PARSEC, and the signature will be constructed once enough shares reach consensus.

### Impact on PARSEC

The BLS keys will become a crucial component of PARSEC's common coin. As such, PARSEC will need to be aware of the BLS keys assigned to nodes internally. This suggests that it may be beneficial to implement the BLS keys initially already as a PARSEC component.

The main changes introduced by the implementation of BLS as a PARSEC component would be:
1. Two new internally handled observation types: `DkgPart` and `DkgAck`. These would carry the corresponding DKG messages.
2. Modifications to the flow of voters set mutations. Currently, the voters set is being mutated once `Add`, `Remove` or `Accusation` reach consensus. With BLS, consensus on such an observation would only kickstart the DKG and add the observation itself to a set of pending mutations. Multiple DKG instances could be run in parallel, if another mutation reached consensus before the last DKG finished. Then, once a DKG finishes, all the pending mutations up to the one that started the finished DKG could be applied to the voters set, and DKG instances older than the finished one could be discarded.
3. Modifications to the way gossip is handled. At the moment, only nodes that have been added to the set of members are allowed to send gossip to other nodes or create observations - this will have to be changed in order to accommodate the participation of to-be Elders in the DKG. Note that this shall only enable participation from the nodes that are going to become Elders, not from all Adults.

#### Handling splits

Since PARSEC should be routing-agnostic, we can't just code split-specific logic in it. However, splits differ from other cases, so they will need to be handled in a special way. PARSEC will have to provide means for routing to be able to handle them properly.

Handling splits presents two main challenges:
- Dealing with two separate DKG instances started by the same event (a node joining),
- Making it possible for non-Elders (the future Elders of the sections after the split) to participate in gossip.

The main idea here is to provide a way of triggering DKG instances externally - Routing could then detect split conditions and start relevant DKGs. We present two possible alternatives of doing that. The final decision how to proceed will be made later.

##### Option 1 - opaque DKGs

One way would be to enable running DKGs in a way completely opaque to PARSEC. On a split condition:
1. Routing would create 2 DKG instances in itself.
2. All `DkgPart`s and `DkgAck`s would be input into PARSEC as opaque payloads.
3. When an opaque `DkgPart` or `DkgAck` reaches consensus, it would be input into the relevant DKG instance.
4. If a non-elder wanted to input its `DkgPart`/`DkgAck`, it would do so via an Elder - or multiple Elders, to make sure that at least one will be honest.
5. Routing would detect that a DKG finished and get the result.

Advantages: less modifications to PARSEC required.

Disadvantages: need to pass messages from Adults via Elders, need to expose the DKG from PARSEC to Routing or to duplicate its functionality in Routing.

##### Option 2 - transparent external DKGs

Another way would be to just tell PARSEC to run a DKG and get a result out of it.
1. Routing would vote for a new observation `DkgForPeers<BTreeSet<PublicId>>`
2. PARSEC would start the DKG once this gets consensus; the peers listed in the observation would be added as valid sources of observations/gossip.
3. PARSEC would run the DKGs just like for `Add`s/`Remove`s.
4. Once such a DKG completes, PARSEC would output a block with a DKG result (`PublicKeySet` and an optional `SecretKeyShare`).
5. Routing would get the results via `poll()`.

Advantages: more consistent handling of DKGs both for `Add`s/`Remove`s and for splits.
Disadvantages: more invasive to PARSEC - a new transparent observation required.

## Drawbacks

A potentially large effort required to implement all the necessary elements within PARSEC.

## Alternatives

No other designs have been considered.

## Unresolved questions

None at the moment.
