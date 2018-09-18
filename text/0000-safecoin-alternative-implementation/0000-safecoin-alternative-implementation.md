# Safecoin Alternative Implementation

- Status: proposed
- Type: new feature
- Related components: SAFE Vault, SAFE Client Libs
- Start Date: 15-09-2018
- Discussion:
- Supersedes: N.A.
- Superseded by: N.A.

## Summary

Implementation details of safecoin; the currency of the SAFE Network.  This RFC describes a different approach to implementing safecoin compared to that proposed in [RFC #0012].



## Conventions

- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).



## Motivation

The motivation for implementing safecoin is well described in [RFC #0012].  The motivation for this particular RFC is to present an alternative approach which has several advantages over the other:

* Simpler
* Fairer for farmers
* Faster completion of transactions
* Less overhead for the network to manage (less data, smaller account structures, fewer messages)
* Coin divisibility can be handled trivially
* Doesn't require deletion of data (hence adhering to Network Fundamentals) when recycling coins
* Simpler to implement pre-farmed state for network startup



## Detailed design

### Overview

The crux of this proposal is that safecoin don't exist as units of data representing individual coins (or parts of coins), but rather only balances of safecoin exist.  These balances will be in the form of values in client-owned `CoinAccount`s and `PaymentAccount`s and as section-wide values representing the sections' farmed totals.

First, let's define the form of these balances:

```rust
struct Coin {
    units: u32,
    parts: u32,
}
```

The `units` field will represent whole safecoins, and since the defined upper limit of issuable safecoin is `2^32`, this need be no bigger than a `u32`.

The `parts` field represents the number of `2^-32`-th parts of a single safecoin.  Since the maximum value of a `u32` is `(2^32)-1`, `parts` will always represent less than a single safecoin.

To represent client-owned accounts for holding safecoin, the following will be used:

```rust
struct CoinAccount {
    id: PublicSigningKey,
    balance: Coin,
}
```

As with other network addressable entities, the `id` will represent the account's address on the network.  It will be managed by the close group of Vaults for that address, using a new persona: CoinManager.  The ID is also an asymmetric public signing key.

A single client can create many `CoinAccount`s if desired, and there is no need to associate any of these with its main MAID account.  (In this context, MAID stands for MaidSafe Anonymous ID, and any user wishing to send requests to the network which require payment must first create a MAID account.  This is not a `CoinAccount`, but rather a record held by the network used to hold e.g. the MAID public key and the public keys of authorised apps).  For example, a user may want to run multiple farmer Vaults and allocate a different `CoinAccount` for each to receive the farming rewards.

When a client creates a new `CoinAccount` on the network, the request need only specify the `id`, since the initial balance will always be 0.  It is up to the client to create and manage the key pairs of all `CoinAccount`s it owns.  These could be stored on the network as part of its encrypted MAID account, or managed by a standalone wallet application.

To pay for network resources (e.g. putting data) a `PaymentAccount` must exist where the account's ID is the same as the MAID's public key:

```rust
struct PaymentAccount {
    details: CoinAccount,
    app_rules: BTreeMap<PublicSigningKey, AppRules>,
}

enum AppRules {
    Unrestricted,
    SpendLimit {
        remaining: Coin,
    }
}
```

The details of the `PaymentAccount` will be explained below at "[Paying for Network Resources](#paying-for-network-resources)".  Until we reach that point, just consider this as a wrapped `CoinAccount` and when `CoinAccount` is mentioned it means either of these types.



### Farming

Each section of the network will be responsible for a proportion of the total issuable safecoin equal to the proportion of the network address space it manages.  By "responsible for", we mean responsible for managing the amount of farmed coin from that address space by ensuring it doesn't exceed the allotted amount or drop below zero.  For instance, at the start when the network has a single section covering the full address space, that section will be responsible for all `2^32` safecoin.  When the section splits, each half will be responsible for `2^31` safecoin.

This means that for a section with Prefix length `n`, it will be responsible for `2^(32-n)` safecoin.

Each section's CoinManagers will maintain a record `farmed` (of type `Coin`) of the amount of safecoin farmed at that section.  Any changes to this total will come as a result of an action having passed through Parsec in order to ensure that all managers maintain an eventually consistent record.

The section's `farmed` value will never be allowed to exceed the amount of coins for which that section is responsible.  Ideally no section should ever get close to "running out" of farmable coins; getting the farming rate algorithm correct should ensure that.  However, in the case that a section has farmed all of its coins, it will stop issuing any more until the `farmed` value reduces again.

The actual algorithm defining the farming rate is somewhat orthogonal to this RFC and worthy of its own RFC, so the details will be omitted, but it would seem fairly critical that it ensures sections don't become fully farmed or fully unfarmed.

One benefit of providing coin divisibility is that on a successful farming attempt, it is trivial to pay all farmers in that section rather than the lottery approach taken in the alternative proposal whereby only one farmer is paid at a time, and only if the chosen coin is unfarmed.  Indeed, they could be paid proportional to the value they have provided to the network.  They could also be paid smaller amounts more frequently to encourage new Vault owners to continue to run their Vaults.

To send a payment, the CoinManagers will send a `Credit` message to the destination `CoinAccount`'s section (the destination address defining the particular `CoinAccount::id`):

```rust
struct Credit {
    amount: Coin,
    transaction_id: Option<u64>,
}
```

(The `transaction_id` will be explained in the "[Transferring to a Different `CoinAccount`](#transferring-to-a-different-coinaccount)" section below, but when sending farming rewards, it will always be `None`.)

The CoinManager persona will be responsible for managing all aspects of farming within their section.  This will include among other things:

* calculating the farming rate
* maintaining a mapping of Vault to `CoinAccount::id` for each Vault in their section (i.e. where to pay successful farming attempts)
* sending `Credit`s to these `CoinAccount`s when a farming attempt is successful, and increasing the `farmed` total for the section
* receiving `Credit`s from other sections and updating the corresponding `CoinAccount`s

When a section `X` splits into `X0` and `X1`, each new section will start with half of `X`'s final `farmed` value.  If `farmed.parts` is odd, rather than rounding, `X1` will be allocated the extra 1 part.

When two sections merge, the new section's `farmed` value will be the total of their final `farmed` values.

When handling a received `Credit`, if the specified `CoinAccount` doesn't exist, the coin will be recycled by decreasing that destination section's `farmed` value by the specified `Credit::amount`.  It would perhaps seem more intuitive to return a failure message to the source section, since that's the `farmed` value which was increased, and hence it seems fairer to recycle the coin back into that source section.  However, handling this would involve more traffic and more code, and such "unfairness" is likely to become fair overall when applied equally across all sections.

When a farming vault first starts, the user must specify an associated `CoinAccount::id`.  This will be persisted by the network when the vault gets relocated.  The client can supply a new ID if such a request is signed by the old ID.  If no initial ID is provided, that vault will never earn farmed coin (its proportion will never be sent and will remain unfarmed at the source).



### Other Rewards

CoinManagers will also be responsible for paying other rewards, i.e. those for application developers and maintainer developers.  The details of how these actual `CoinAccount`s are managed are left for a future RFC, but the IDs of these accounts will be known to the section's `CoinManger`s so that rewards can be paid into them using `Credit` messages.

As with farming, when a reward is paid, the section from which it is paid will increase its `farmed` total accordingly.



### Paying for Network Resources

When a client wants to consume network resources which requires payment (e.g. when putting data), it will send such a request to its MaidManagers (as is currently implemented).  However, before approving this request, the MaidManager Vaults will deduct appropriate payment from the `PaymentAccount` with the same ID as the MAID.  Since the client's MAID account and `PaymentAccount` have the same ID, each vault which is a MaidManager will also be a CoinManager for that account; so no extra network messages will be needed to take the payment.

Furthermore, to handle the case where a client sends multiple such requests, but doesn't have sufficient balance to cover them all, the receiving Vaults will pass the requests through Parsec in order to get an agreed order of these requests.  In that way, there will be eventual agreement amongst them about which transactions are valid and which aren't.

As per the existing Vault implementation, once approved, the MaidManagers will forward the request to the appropriate DataManagers.  This will continue to be the case, but the request will also now include a `Coin` field specifying the amount which the client paid.

When handling these requests from MaidManagers, the DataManager Vaults decide whether the request can be actioned or not.  If it can, these Vaults will have their CoinManager personas deduct the paid amount from their section's `farmed` total, effectively recycling the coin and making it farmable again.  If the request can't be actioned, the DataManagers send a failure response back to the MaidManagers (as per the existing implementation).  On receipt of such a failure message, the client's CoinManagers will refund the amount to its `PaymentAccount`.

The MaidManagers currently already handle a single MAID account being associated with multiple "app authentication keys", allowing different applications different rights with regards to data mutation.  This notion will be extended to include spend limits on the Client's `PaymentAccount` from which it will make payments.

When an app makes a payable request, the client's `PaymentAccount` will be checked to see if the app has the required balance available to it.  If there is an entry corresponding to that app's keys in `PaymentAccount::app_rules` then:

* for `AppRules::Unrestricted`, the app has permission to use the full balance of `PaymentAccount::details::balance`, and that balance is reduced accordingly
* for `AppRules::SpendLimit { remaining: Coin }`, the app has permission to use only `remaining` from `PaymentAccount::details::balance`.  If there is enough in both, then both values are reduced accordingly.

These rules can be extended if required to increase the flexibility of managing spend-limits for apps.



### Transferring to a Different `CoinAccount`

(Note, when mentioning `CoinAccount` from this point onwards, we mean both `CoinAccount` and `PaymentAccount` unless otherwise indicated.)

When a client wants to transfer some coins from a `CoinAccount` it owns to some other one it may or may not own, it will send a `CoinTransfer` to the CoinManagers of its source `CoinAccount` (the address defining the particular source `CoinAccount::id`):

```rust
struct CoinTransfer {
    destination: PublicSigningKey,
    credit: Credit,
}
```

The `Credit::transaction_id` is an optional value which can be agreed beforehand by the owners of the two `CoinAccount`s and can be used by either to confirm that the transfer has been completed (details below).

As with paying for network resources, before acting upon such client requests, the CoinManagers must pass the requests through the same instance of Parsec in order to ensure that they agree which requests to accept or reject.

On receipt of a `CoinTransfer` request (after passing out of Parsec), the CoinManagers will deduct the amount specified in `credit` from the source account.  If the account's balance doesn't permit this, the request will be silently dropped.

These source CoinManagers will then send the `Credit` to the destination CoinManagers, where it will be run through Parsec to cover the case where that owner is concurrently trying to spend coin from that `CoinAccount`.

If the account to be credited doesn't exist, the destination CoinManagers will recycle the coin by deducting the value from their section's `farmed` value.  The client will receive no notification that the transaction failed.  (We could look to refund the source `CoinAccount` in such cases, but this would require more effort by the network.  Well-designed client applications should be able to reduce the risk of accidental loss of coins in this way to zero.)

Each `CoinAccount` will have an associated fixed-length FIFO queue for holding the most recent `transaction_id`s and `amount`s of `Credit`s made to that account.  When crediting the destination `CoinAccount`, the CoinManagers will push the `transaction_id` and `amount` onto that queue if the `transaction_id` is not `None`.  At this stage, the transaction is complete.

Any client will be able to query for the existence of such a completed transaction by sending a `GetTransaction` message to the destination `CoinAccount`'s section (the destination address defining the particular `CoinAccount::id`) and expecting to receive a `Transaction` in response:

```rust
struct GetTransaction {
    transaction_id: u64,
}

enum Transaction {
    Success {
        value: Coin,
        transaction_id: u64,
    },
    NoSuchTransaction,
    NoSuchCoinAccount,
}
```

A CoinManager receiving a `GetTransaction` will always return a `Transaction`.  If the specified `CoinAccount` doesn't exist, `NoSuchCoinAccount` is returned.  If the `CoinAccount` does exist, but the requested `transaction_id` doesn't exist in its FIFO queue, `NoSuchTransaction` is returned.  Otherwise the appropriate `Success` value is returned.  As this message is signed by the Vaults comprising the CoinManagers, a `Transaction::Success` can be used as proof by any client, (in particular the sender) that the transaction was successful.

This has the benefit of requiring neither the sender nor the receiver to stay connected while the payment is being processed, as would be the case for example if the sender needed to receive a response in order to prove payment had been made.  It also serves to further anonymise the sender, as the recipient will likely not be aware of the actual `CoinAccount` used by the sender to credit its own; only the `transaction_id` is visible.



## Drawbacks

None noted at the moment, although some may be added as they are identified during the RFC review process.



## Alternatives

The original proposal for a safecoin implementation is detailed in [RFC #0012].



## Unresolved questions

* We may need to handle the case where a section is receiving payment (e.g. for a Put request), but its `farmed` amount is less than the payment, meaning the coins can't all be recycled by that section.  This would seem to imply a failure of the farming rate, but potentially we'd need to cover that situation, e.g. by having that section forward a specific new type of message to a neighbour section which _can_ handle recycling that amount of coins.

* If payment rates weren't variable from section to section, we could omit the `Coin` field from the requests which MaidManagers send to DataManagers.

* Instead of `GetTransaction` and `Transaction`, we could possibly use push notifications to notify the sending and receiving clients of a completed transaction.

* It's unclear at the moment how to discourage spamming the network with `CoinTransfer` requests for tiny amounts without doing something like charging a minimal amount for handling transfers.  It's also similarly unclear how to discourage the creation of excessive numbers `CoinAccount`s by a malicious client.  This could potentially be handled by the proxy nodes.

* The original proposal suggested allowing new MAID accounts to be given a small amount of safecoin for free to be used exclusively for paying for network resources.  This may still be required, and would probably require special handling of such `CoinAccount`s.  However, with the approach described in this RFC, pre-farming may be enough to obviate the need for such special handling.

* Using Parsec should ensure that a malicious proxy can't replay a client's `CoinTransfer` or other chargeable request, since Parsec observations need to be unique.  However should this not be the case (e.g. in light of Parsec pruning), then an alternative mechanism would need to be found to prevent this attack.  Examples could be requiring clients to send requests via a quorum of proxy nodes, or to specify a sequence/version number which would be held in the `CoinAccount`.



[RFC #0012]: https://github.com/maidsafe/rfcs/blob/master/text/0012-safecoin-implementation/0012-safecoin-implementation.md
