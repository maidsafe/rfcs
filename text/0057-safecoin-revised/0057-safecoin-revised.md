# Safecoin Revised

- Status: implemented
- Type: new feature
- Related components: SAFE Vault, Routing, SAFE Client Libs
- Start Date: 01-05-2019
- Discussion: https://safenetforum.org/t/rfc-57-safecoin-revised/28660
- Supersedes:
    - [RFC 0005: Balance Network Resources][rfc05]
    - [RFC 0012: Safecoin Implementation][rfc12]
    - [RFC 0051: Safecoin Alternative Implementation][rfc51]
- Superseded by: N.A.


## Summary

This RFC combines the three previous RFCs as indicated above. It describes the link between resource constrained sections and the desire to add new nodes. As safecoin is the "oil" of the network it may appear to couple certain aspects of the network and in many ways does, but this is a critical aspect of the network that combines many aspects into a cohesive system. It should be noted that a client's identity on the network is represented by a `BLS::PublicKey` which allows multisig capabilities and which is probably essential for any currency to be secured and usable. Such a public key requires an amount of safecoin to be associated with it to be of any use. This relationship is represented by a `CoinBalance`.

In addition we recognise that writes to the network will be slowed down as `CoinBalance`s are checked for payment, but this is both acceptable and also important for the network to not be spammed without payment. Writes are slow, but reads will be as fast as possible as caching and no request for payment is involved. There will also be replay attack prevention for transfer of coins.


## Conventions

- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).


## Motivation

SAFE cannot operate without data and data cannot be maintained without incentivising resource providers (Farmers). This RFC defines the relations between farmers, clients who store or mutate data and consumers who drive usage of the network. This critical component has received significant input from the community over the years and this RFC combines much of that feedback and brings the previous RFCs into a single document, albeit with minimal changes.


## Detailed design

### Overview

As in [RFC 0051][rfc51], this proposal is that safecoin doesn't exist as units of data representing individual coins (or parts of coins), but rather only balances of safecoin exist on the network. These balances will be in the form of values in `CoinBalance`s and as section-wide values representing the sections' farmed totals.

```rust
struct Coins(u64);
```

The inner value will represent a count of nano-safecoins, and will never exceed the upper limit of issuable safecoin (i.e. 2^32 safecoin, or equivalently 2^32 * 10^9 nano-safecoins).

To associate a (client-owned) public key with its safecoin balance, the following will be used:

```rust
struct CoinBalance {
    owner: BLS::PublicKey,
    value: Coins,
}
```

A `CoinBalance`'s address on the network will be that of its `owner` field; i.e. its public key's address. As with other network addressable entities, it will be managed by the elders of the section covering this address. `CoinBalance`s won't be encrypted since the vaults need to be able to manipulate them (i.e. when payment is made to or from them), but it's proposed they are get-able only by the owner(s) which can sign a GET request with the corresponding BLS secret key.


### Safecoin transfer

To transfer safecoin from one `CoinBalance` to another we send a signed RPC to the Elders of the sending node. The Elders then Vote the transaction through PARSEC. The sender's balance is only checked once the vote is polled out of PARSEC, as there may have been further payments awaiting confirmation while this current payment was becoming valid. An error is returned to the Client if the payment failed due to insufficient funds.

When a vote for a payment is valid, and funds are available, all Elders reduce the balance from `CoinBalance` corresponding to the sender's BLS id, and they send an RPC (including the original client request and signature) to the remote group to increase by the `Credit` amount the balance in the `CoinBalance` corresponding to the destination's BLS id.

```rust
struct Credit {
    amount: Coins,
    transaction_id: Uuid,
}

struct CoinTransfer {
    destination: BLS::PublicKey,
    credit: Credit,
}

send(from: BLS::PublicKey, to: CoinTransfer, signature: Signature)
```

where the signature is the source `CoinBalance`'s BLS signature of the `CoinTransfer`.

```rust
get_transaction(coin_balance_id: BLS::PublicKey, transaction_id: Uuid)

enum Transaction {
    /// The associated `CoinBalance` was successfully credited with this `Credit`.
    Success(Credit),
    /// This transaction is not known by the associated `CoinBalance`.  This could be because it was
    /// never known, or is no longer known.
    NoSuchTransaction,
    /// The requested `CoinBalance` doesn't exist.
    NoSuchCoinBalance,
}
```

Receiving a `CoinTransfer` is a single PARSEC vote to increase a `CoinBalance`'s value. Once the vote is returned by PARSEC having reached consensus, the Elders try to credit the indicated `CoinBalance`. If it doesn't exist, the source balance will be refunded by sending a new `CoinTransfer` back to the source for the same amount.

Each `CoinBalance` will have an associated fixed-length FIFO queue named `credits` for holding the most recent `Credit`s paid into it. When crediting the destination `CoinBalance`, the Elders will push the `Credit` onto that queue. At this stage, the transaction is complete.

Any client will be able to query for the existence of a transaction by sending a `get_transaction` message. Elders receiving such a request should respond with an appropriate `Transaction` after searching the `credits` of the indicated `CoinBalance`. If the specified `CoinBalance` doesn't exist, `NoSuchCoinBalance` is returned. If the `CoinBalance` does exist, but the requested `transaction_id` doesn't exist, `NoSuchTransaction` is returned. Otherwise `Success(Credit)` is returned. As this message is signed by the Elders, a `Transaction` can be used as proof by any client, (in particular the sender) that the transaction was successful.

Given that these `get_transaction` requests are acted upon by the Elders immediately (i.e. they're not voted through PARSEC), it's possible that Clients will not receive any response as the Elders may not have consensus about the `Transaction` variant to send. In such a case, the clients will simply retry repeatedly with a delay until they do get a response. The Elders will eventually reach consensus, so this polling is guaranteed to not go on indefinitely.

This has the benefit of requiring neither the sender nor the receiver to stay connected while the payment is being processed, as would be the case for example if the sender needed to receive a response in order to prove payment had been made. It also serves to further anonymise the sender, as the recipient will likely not be aware of the actual `CoinBalance` used by the sender to credit its own; only the `transaction_id` is visible.

It should be noted that by providing a unique `transaction_id` for every coin transfer, it renders such transfers safe from replay attacks, since PARSEC will disallow multiple votes by a single peer for the same observation.


### `CoinBalance` creation

We create a `CoinBalance` by generating an asymmetric key pair. We will use BLS keys. The secret key stays local to the client.

```rust
create_balance(from: BLS::PublicKey, new_balance_owner: BLS::PublicKey, to: CoinTransfer, signature: Signature)
```

The `from` field indicates the `CoinBalance` paying for this new `CoinBalance` to be created. The owner of the paying `CoinBalance` could be ourself or a friend who is creating a `CoinBalance` for us. As this does not require us to provide a secret key then it is a safe operation for a friend to create a `CoinBalance` for us. Once we have a `CoinBalance` then we can go on to create an account. This is described below. It may appear that we duplicate the `new_balance_owner` field here as this data also exists as `to.destination` as described in the previous section. The reason for this is that here we will create a `CoinBalance` if it does not exist, whereas in a normal safecoin transfer then an error in the `to` field will be detected and the payment returned to the user who perhaps made a mistake in the `to` field. In the case of `CoinBalance` creation, Elders MUST confirm both of these `to` public keys are equal, otherwise this is an error and should be detected at the source Elder section who will not process the transaction and will return an error to the sender.

It is worth noting that this allows people to maintain hardware wallets or similar where some of their `SecretKeys` are held entirely in a hardware wallet.

**Note:** Safe Client Libs should make available a human readable `PublicKey`. [z-base32 encoding](http://philzimmermann.com/docs/human-oriented-base-32-encoding.txt) seems to be a good choice as it is case insensitive, as opposed to other case-sensitive encodings like base58btc or base64url, and it was designed to be easier for human use permuting the alphabet so that the easier characters are the ones that occur more frequently.


### Account creation

Only when a Client has a `CoinBalance` with funds then it can create an account. The account will hold the client's secret keys and any other data such as directory identifiers for their own data. Account creation is described below and is a "blob" of data held by Elders closest to the `PublicKey` of the account packet. As the user already has a `CoinBalance` with funds, this packet is created by paying directly from the `CoinBalance` created above.

This is not an appendable data packet as it is managed by the Network and its contents can be anything up to 1MB. The charge to store is 1MB, but any updates are free of charge.

To create the account packet we choose two pieces of random data (passwords) - the first gives us the location of the login packet and the second is our encrypt/decrypt key.

To secure these packets we do not wish anyone to download them, so we protect them by

1. Running the password through [PBKDF2](http://en.wikipedia.org/wiki/PBKDF2) to provide a 256bit password.
2. Encrypting the account packet with AES-SIV.
3. Creating the hash(password) and seed a [PRNG](http://en.wikipedia.org/wiki/Pseudorandom_number_generator) in order to create a signing keypair.
4. Prepending the public key to our encrypted account packet.
5. Signing the account packet with the secret key and prepending that signature.
6. Storing the account packet at the address provided by the location password.

To download the account packet, the Client creates the same keypair as in 3 above and signs the request for the account packet to be downloaded using the secret key. On downloading we confirm the data is still valid by checking the signature before decrypting with the password from point 2 above. This does not protect the packet from Network level nodes' snooping, but does protect against mass download attempts by bad actors.


### Farming

We will update [RFC 0012][rfc12] where we alter the calculations on sacrificial chunks to that of relocated chunks. When a node cannot store a chunk due to being at capacity, then the chunk is stored on the next closest node. This is a relocated chunk and Elders should keep a note of relocated chunks. As this is handled via PARSEC then it SHOULD NOT present any issue. Should a node pretend to store chunks, it puts itself in danger of losing all Age. For this reason when a node is asked to send a chunk to a requester and does not then the Elders should vote to kill the node. This means the node cannot rejoin the network.

This RFC also updates the original [RFC 0005][rfc05]. As in RFC 0012, as these relocated chunks increase then so will farming reward. However as we have a full node now in this section, we now should add additional capacity and add a new node into the section.

We also update the terminology of both of these RFCs and substitute `ClientManager`, `DataManager` and `CoinManger` with `Elder`.

Each section of the network will be responsible for a proportion of the total issuable safecoin equal to the proportion of the network address space it manages. By "responsible for", we mean responsible for managing the amount of farmed coin from that address space by ensuring it doesn't exceed the allotted amount or drop below zero. For instance, at the start when the network has a single section covering the full address space, that section will be responsible for all `2^32` safecoin. When the section splits, each half will be responsible for `2^31` safecoin.

This means that for a section with Prefix length `n`, it will be responsible for `2^(32-n)` safecoin.

Each section's Elders will maintain a record `farmed` (of type `Coins`) of the amount of safecoin farmed at that section. Any changes to this total will come as a result of an action having passed through PARSEC in order to ensure that all managers maintain an eventually consistent record.

The section's `farmed` value will never be allowed to exceed the amount of coins for which that section is responsible. In the case that a section has farmed all of its coins, it will stop issuing any more until the `farmed` value reduces again.

To send a payment, the Elders will send a `Credit` message to the destination `CoinBalance`'s section (the destination address defining the particular `CoinBalance::owner`).

The Elder group will be responsible for managing all aspects of farming within their section. This will include among other things:

* calculating the StoreCost
* maintaining a mapping of Vault to `CoinBalance::owner` for each Vault in their section (i.e. where to pay successful farming attempts)
* sending `Credit`s to these `CoinBalance`s when a farming attempt is successful, and increasing the `farmed` total for the section
* receiving `Credit`s from other sections and updating the corresponding `CoinBalance`s

When a section `X` splits into `X0` and `X1`, each new section will start with half of `X`'s final `farmed` value. If `farmed.parts` is odd, rather than rounding, `X1` will be allocated the extra 1 part.

When two sections merge, the new section's `farmed` value will be the total of their final `farmed` values.

When handling a received `Credit`, if the specified `CoinBalance` doesn't exist, the coin will be recycled by decreasing that destination section's `farmed` value by the specified `Credit::amount`. It would perhaps seem more intuitive to return a failure message to the source section, since that's the `farmed` value which was increased, and hence it seems fairer to recycle the coin back into that source section. However, handling this would involve more traffic and more code, and such "unfairness" is likely to become fair overall when applied equally across all sections. Since the behaviour in this scenario of a non existing destination `CoinBalance` is different between a farming reward payment and a normal safecoin transfer, it's proposed to have a second type of RPC for the reward payment, e.g. `reward()` with the exact same function signature as the `send()` function.

When a farming vault first starts, the user must specify an associated `CoinBalance::owner`. This will be persisted by the network when the vault gets relocated. The client can supply a new public key representing a different `CoinBalance` for receiving farming rewards if such a request is signed by the current `owner`'s corresponding secret key. If no initial `CoinBalance::owner` is provided, that vault will never earn farmed coin (its proportion will never be sent and will remain unfarmed at the source).


### Establishing StoreCost

We'll make the following definitions related to the numbers of nodes within a single section:

* `N` = total number of nodes
* `F` = number of currently full nodes (those whose last Put request failed because they're full)
* `G` = number of good nodes = `N - F`

We want to reduce the cost to store (and hence also the farming reward) when the number of good nodes increases, and also when the proportion of full nodes decreases. To that end, we'll use the following formula:

```
StoreCost = 1/G + F/N
```

This formula will very likely need to be refined as we gain a better understanding of how the network is being used, and how the StoreCost affects it.


### Farm reward calculation

The farming reward calculation is also a simple affair, but must mitigate against specific attacks on the network. These include, but are not limited to:

1. Continual Get against known data on a vault
2. Attempted targeting of farm rewards

For now, we will use an algorithm which would eventually deplete all farmable coin, but which is simple to implement while we gather further data from testnets.

When a client pays to store or mutate data, the payment will be immediately be divided amongst farmers. Furthermore the amount paid will be matched by the client's section (previously called the MaidManagers) by increasing the section's `farmed` total accordingly. This will yield a reward which is 2 * payment amount.

Such a reward will be divided as follows:
* If the data has an associated App Developer's public key, their `CoinBalance` is awarded 10% of the total
* The Core Developers of the SAFE Network are awarded 5% of the total
* The remaining amount (85 or 95%) is divided amongst the vaults in that section, each being awarded a share proportional to its age. (This might need adjusted later, e.g. to bias rewards towards or away from Adults)
```
single_node_age = if no associated CoinBalance::owner { 0 }
                  else if flagged as full { node's age/2 }
                  else { node's age }
total_age = sum of each vault's single_node_age
reward_proportion = single_node_age / total_age
```

This means that if any Elder or Adult doesn't have an associated `CoinBalance::owner`, their share will remain unfarmed (i.e. it will be deducted from the section's `farmed` total). It also means that vaults which are full are only receiving half of what they'd otherwise earn.

To avoid making many frequent small payments, we will buffer rewards until the total amount paid by clients since the last rewards were actually paid exceeds 1 safecoin, or until the section splits or merges.


### Payment address

1. Farmers -> Registers an optional (if set by user) `CoinBalance::owner` on vault creation
2. App Developer -> App developers will include their `CoinBalance::owner` in any `Put` request
3. Core development -> Initially every node will be aware of a hard coded `CoinBalance::owner` for core development. This will likely lead to a multi-sign `CoinBalance::owner`.


### Section health

The farming rewards are designed to incentivise "healthy" sections - ones which have enough storage capacity, but not excessive amounts. As well as this routing will provide mechanisms to help.

Each section will aim to maintain a minimum ratio of 50% good nodes - ones which aren't full. As vaults are added, removed, or flagged full, if that ratio of good nodes drops or remains below 50%, vaults will ask routing to add a new node to the section.

From the perspective of routing itself, sections will always split if both the resulting new sections will have at least 100 nodes each.


### Future enhancements

* When a node joins a section as an `Infant`, or it's relocated as an `Adult` or `Elder`, we then must confirm the node is acceptable to the Section. We'll continue to use the `resource_proof` crate for now to get new nodes to do some dummy work before being permitted to join a section. In future, this will be replaced by passing the new node the actual data for which it will become responsible once joined. If it fails to store it or fails to respond to Gets for it, the node will be punished.
* This is in addition to the routing layer's section size of 100. Routing will accept new nodes quickly to get to the level of 100 new nodes, however each new node will be tested as above. Therefore accepting a node to a section will revolve around 100 nodes capable of storing data, full nodes will not be counted as part of the section recommended size of 100. It is recommended that as sections grow to 200 members (regardless of storage capability) they will split.
* We also expect the StoreCost algorithm to be modified in the future, according to further work and observed data from upcoming testnets.
* Once the network supports push-notifications/messaging to clients, we will replace the `credits` FIFO queue and not require clients to poll for transaction results.


## Drawbacks

The FIFO queues for holding recent `Credit`s are prone to being attacked by a malicious user who sends a flood of micro-transactions to the given `CoinBalance` in order to quickly push out valid entries. There are options to make this mechanism more robust, but since we expect to replace this once push-notifications/messaging to clients is implemented, it's not worth the extra complexity for now.


## Alternatives

As stated in [RFC 0051][rfc51] an alternative is to hold each safecoin as a data type itself, a physical coin, however that pattern makes transfers very expensive and also limits the ability of the network to use micro payments of less than a single safecoin.


## Unresolved questions

1. We might want to allow users to first join the network by starting and running a vault for a while in order to accrue enough safecoin to be able to create a client account packet. This would need to be detailed further, since we currently require payment for the creation of a new `CoinBalance` instance, whereas here the vault would need to create one without being able to pay for its creation.
1. We need to look in a lot more detail what changes to PARSEC are required to support our use case here.



[rfc05]: https://github.com/maidsafe/rfcs/blob/master/text/0005-balance-network-resources/0005-balance-network-resources.md "RFC 0005: Balance Network Resources"
[rfc12]: https://github.com/maidsafe/rfcs/blob/master/text/0012-safecoin-implementation/0012-safecoin-implementation.md "RFC 0012: Safecoin Implementation"
[rfc51]: https://github.com/maidsafe/rfcs/blob/master/text/0051-safecoin-alternative-implementation/0051-safecoin-alternative-implementation.md "RFC 0051: Safecoin Alternative Implementation"
