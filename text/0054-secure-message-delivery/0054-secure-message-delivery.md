# Secure Message Delivery

    Status: proposed
    Type: enhancement
    Related components: DHT, Routing, Vault
    Start Date: 04-05-2019
    Discussion:
    Supersedes: None
    Superseded by: None

## Summary

This RFC describes how the network can be certain that a message received at a destination was validly sent from a trusted section in another part of the network. The scope of such messages includes safecoin credits, changes in infrastructure and more, The combination of [reliable message delivery](https://github.com/maidsafe/pre-rfc/blob/master/dht/reliable-message-delivery.md) and this RFC enables the network to transmit messages with reliability and efficiency.

## Conventions

    The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

## Assumptions

- We assume sections prove themselves via the use of a BLS public key.
- We assume for now that the merges will not occur in the network. Handling merges can be added as an extension later, but assuming their absence simplifies a part of the design.
- Existing group Authorities such as `NaeManager` and so on will all fold into a single type, `Elders`, consisting of the Elders of the section. These Elders will, for now, manage all nodes in a section.
- We assume Elders are removed when not participating in the network, this participation is not answering heartbeats alone as the network does not reward that, it requires Elders to participate in PARSEC and decision making, failure there must be penalised. We do not pay nodes to answer heartbeats alone.

## Motivation

The correct functioning of the Network relies strongly on various decisions being made collectively by group authorities. There are situations in which nodes in some part of the Network need to be made aware of a decision made by other nodes, which they are not directly in contact with. In order to prevent malicious actors from impersonating nodes from an a priori unknown part of the network, a mechanism for authenticating message source to the message destination is required.

The Network workload being reduced and the users experience being enhanced is an essential goal. Simplicity as well as efficiency is essential to launching quickly and correctly. The network cannot afford complex mechanisms to deliver messages, nor can it afford to slow down message delivery, even by increasing size of messages or checking/deserialising at every hop. The network must deliver all messages quickly and securely, this RFC focusses on those goals.

## Detailed design

### Signatures

The most basic mechanism for proving the origin of a message is a cryptographic signature. We will be employing two types of signatures, depending on whether a message is signed by a single node, or a group Authority.

Each node in the network holds an ED25519 keypair. The public component of this keypair also functions as an identifier for the node. Whenever a node wants to prove its authorship of the message, it will sign it with the private component of this keypair.

Each Elder SHALL also hold a BLS keypair share. These shares will be a part of an aggregate keypair representing a whole section. Private key shares will allow for generating signature shares, which will then be combined into a single collective signature for a message originating in a group Authority. Such a signature - represented in pseudocode below as the `BLS::Signature` type - can be verified using a corresponding aggregate public key, which will be represented as the `BLS::PublicKey` type.

These BLS keypairs SHALL NOT be tied to the identities of nodes. Whenever the set of the Elders in a section changes, a new set of key shares will have to be generated.

### Shared state

In order to be able to prove their section, and to trust proofs from other sections, nodes will have to keep some state. This state will be shared between nodes in a section and only modified as a result of consensus via PARSEC, hence we will call it "shared state".

The state needed to satisfy the requirements stated above consists of three elements:

- A history of the node's own section.
- Known public keys of other sections.
- Points in own section's history that other sections are known to trust.

The history of own section SHALL be formed of a chain of `SectionProofBlock`s:

```rust
struct SectionProofBlock {
  key: BLS::PublicKey,
  sig: BLS::Signature
}
```

Every time the set of Elders changes, a new set of BLS keypairs will be generated, which will be shares of a new section's `BLS::PublicKey`. In order to maintain trust, this new public key will be signed using the old key shares, thus forming a signature that will be a part of the new `SectionProofBlock` alongside the new public key.

If we want to handle merges as well, this will need to be extended to a full Directed Acyclic Graph of structures similar to a `SectionProofBlock`.

Known public keys of other sections is simply a map of `Prefix` → `BLS::PublicKey`.

The last part is also a map keyed by `Prefix`es, but holding references into the chain of our `SectionProofBlock`s.

### Section Proof Chains

A list (a `Vec` in the Rust terms) of `SectionProofBlock`s, in which every block's public key correctly validates the signature of the next block, will be called a `SectionProofChain`.

We can define a naive type for holding a chain of section proof blocks. It is recommended that a chain type be created that ensures the invariant described above is maintained.

```rust
type SectionProofChain = Vec<SectionProofBlock>
```

Such a chain can be considered a proof that the last public key in the chain is a legitimate public key of the section that corresponded to any of the public keys earlier in the chain. Thus, an Authority that trusts a single public key corresponding to a section, can be convinced by that section to trust their newer public key via a `SectionProofChain` starting at the trusted key and ending at the updated key.

### Secure communication

When a group Authority sends a message to another Authority, then the format of this message is:

```rust
struct SecureMessage {
  proof : SectionProofChain,
  first_prefix: Option<Prefix>,
  last_prefix: Prefix,
  signature: BLS::Signature,
  message: Vec[u8]
}
```

The chain attached as `proof` is the fragment of section's history that starts at the key the recipient trusts according to our shared state, and ends at our current `BLS::PublicKey`.

Whenever such a message is received by a group Authority, it SHALL respond with an acknowledgement message, in order to update the sender's knowledge of the recipient's trust for sender's keys.

The sender prefixes corresponding to the first and last entry in the chain are included in the message in order to allow the recipient to realise that the sender changed its prefix, if such a change happened. If no change happened, `first_prefix` is set to `None`, and `last_prefix` is assumed to correspond to both the first and the last prefix.

If the message originates from a single node, the message will still get signed by the section's aggregate public key - in order to achieve that, the source node will have to input it into PARSEC, and other nodes will sign it once it gets consensused.

### Updating the shared state

As mentioned before, the own history component of the shared state SHALL be updated every time the section's set of Elders changes. This can only happen as a result of the PARSEC consensus, hence updating this part of the state will also be tied to PARSEC consensus.

Known public keys of other sections SHALL be updated any time a section receives a message from another section with a valid `SectionProofChain`. The nodes of the receiving section will attempt to validate the `SectionProofChain` using the previously stored public key for the sending section. If such validation is successful, it will cast a PARSEC vote for updating the section's key to the latest one from the chain. When such votes reach consensus, the component of the shared state will be updated.

It may happen that the sender prefix in the received message will not correspond to any of our prefixes in our map of knowledge of other sections. In such a case, the prefix will be a descendant of one of the prefixes that we are storing. We will thus store the new public key for the descendant prefix, and use the public key we stored for the old prefix to initialise our knowledge of all other prefixes required to cover the address space that was covered by the old prefix.

Example: we store public key X for section 00. We receive a message with `first_prefix` = 00 and `last_prefix` = 001 with a chain that correctly validates when using the key X, and that proves a public key Y. We store Y as the public key for 001, and X as the public key for 000 - 000 and 001 together cover the same address space as 00, so this is all that is needed.

The last component of the shared state - the other sections' knowledge of us - shall be updated only once we receive an acknowledgement response to a message we sent. This is to make sure that even if some messages are received in a different order than the one they were sent, they will still validate correctly - we won't assume that the recipient's knowledge changed until we receive a confirmation.

### Shared state pruning

If the map of other sections' knowledge of our section covers the whole address space, and all of our keys known by other sections are later than some public key X from our history, we can prune our history up to the public key X - older keys will no longer be necessary for any communications.

### Messages from single authorities

If the message originates from a single node, issues might arise if the node is lagging and the receiving section already got updated regarding the knowledge of the sender's section. To prevent such issues, messages from single nodes SHALL be first input into PARSEC and only sent when they reach consensus.

### Remote sections and neighbour sections

In this RFC we make no distinction between close sections (neighbours) authority and remote sections. They all work in the same manner as far as section proofs go. However neighbours connect to each other and this requires neighbours know **who** makes up the Elder group in addition to recognising the **aggregate public key** of an elder group. The other difference for neighbours is that we push notifications of membership changes to them. We must also push to them the **diff** that caused that change, i.e. the ED25519 public key and quic-p2p connections info of elder removed and the same for the Elder added that forced the change in our `SectionProofBlock`. Note the use of ED25519 is still used for individual node identities and BLS for section authority.

### Section splits

Before a section splits, it must sign the new public keys for both siblings, in order to maintain continuity of the history chains. This means that we can only split once we have signed (and thus trust) the keys for both siblings.

Assume that the last section public key before the split was X, and it was used to sign the siblings' keys X0 and X1. Assume that we will belong to the sibling X0 after the split. Once we split, we will initialise the public key of our new sibling to X1, and their knowledge of us to X0, as they must trust X0 by now, too. This way we make sure that further communications between us and our sibling may be cryptographically validated.

### Future Enhancements

A simple enhancement would be to include section health in an `SectionProofBlock`, this would take the form of a 32 bit integer to represent the age of the section Elders and an 8 bit integer to represent the number of nodes in a section. For efficiency these would only be updated at each `SectionProofBlock` update as they would be signed by the sections Elders. With this in place the global network health can be determined and in cases where a section is in need to nodes for instance it can request a node from a healthy section. This pattern could be more dynamic by the group signing **any** changes to it's section and updating this health figure, but that is a little more complex and would require an incrementing version number as well to prevent malice.

Another possible enhancement could be made in the communications scheme - we suggest sending an acknowledgement response to every message in order to update the sender's knowledge of which keys we trust, but this can be optimised in the future (for example, by not responding to messages that don't result in any changes to our trust).

## Drawbacks

The main drawback is the necessity of maintaining state that refers to every single section in the network. However, we expect that the memory requirements will be manageable, so this is not a big drawback.

This approach also seems potentially more vulnerable to spam than the alternative per-hop verification mentioned below - as in that alternative, the first hop will already reject an incorrect message, whereas in this proposal the message will make it all the way to the destination, only to be rejected there. Also, detecting the source of spam would be significantly harder.

## Alternatives

An alternative would be to make every hop prove the previous hop, which would mean that each section would only need to maintain state referring to its neighbours. This, however, requires more work related to validation of messages from the nodes on the Network.

## Unresolved questions

None at the moment.
