- Feature Name: Use public key alone to authorise actions
- Type Enhancement
- Related components routing, maidsafe_types, maidsafe_client, maidsafe_vault
- Start Date: 23-06-2015
- Issue number: #22

# Summary

At the moment client Id packets are created and a hash of these packets used as identity.
This is stored on the network as a `key` (ie the hash) / `value` (the public key) pair.
These packets have to be available prior to the client storing data onto the network.

As the network must confirm a client has paid the network, either by providing resource
or via a network token (i.e. safecoin). Then it needs to look up the clients public key
and confirm the signature of requests come from that client. This is done by querying
the client ID (Hash), then looking up the ID packet and downloading it to get the public key.

As such this requires an initial `unauthorised put` as the client is not known to the
network and cannot be recognised without this. This means essentially the network has to
allow unlited `Put` of client Id packets, thereby exposing a risk of wasted network usage.

This proposal removes all of this indirection and instead allows the client to be recognised
by the public key included in messages or data types. such messages and data types will be
signed by the `secret key` that is paired with this public key. As only the owner should have
access to this `secret key` then it can be assumed the message or request to mutate data is
indeed valid in a cryptographically secure manner.

# Motivation

Storing Id packets is a strain on the network. Lookups are a strain and will slow down
network actions. Id packets themselves may be a security issue (can they be hacked), although
highly unlikely, it is impossible to hack if they do not exist!

This mechanism does not reduce the total number of clients or types of clients as they all exist
in an address space of 2^256 and app developers can use multiple Id's in very clever ways, such
as the SAFE network does not distinguish between public id's and private id's. Now these and more are possible.
Id's could be created to share data types (as co-owners) and allow shares access and write capability
between applications and groups.

# Detailed design

As clients `Put` data or messages on the network (i.e. ask to create) they send such requests via the
ClientManager type persona (MaidManager in the case of data). These requests are signed and include the
clients `PublicKey`. This public key can be allocated, or matched to an existing,to a client account.
The ClientManager may then take any action, such as authorising the `Put`, taking a balance etc. and all against the key.

If a client wishes to amend any data (structured data) then this is already signed by that client and contains the
`PublicKey` The client will sign the request to alter the data (by overwriting with a new valid one) and again
the public key of the signed message is included in the message, confirming what key made the request. This key
can then be confirmed to be one of the owners of the `StructuredData` element and if multiple owners
then the signatures of the data element being uploaded is checked against the list of signatures of the new chunk
using the owners keys of the last chunk.

Clients may identify themselves using the public key as their node address. Signing the request to join the network
at that address. This is a different size key from routing nodes though, which is a benefit as it distinguishes these
connections more clearly. This does require a change to the routing API to accommodate 32 byte node identities.

Clients themselves no longer will require to store Id's packets on the network (although they can via StructuredData).
`PublicId` types are no longer required in the maidsafe_types library (maidsafe_client exclusive now).

# Drawbacks

To be identified

# Alternatives

Alternatively the status quo could be left in place

# Unresolved questions

To be resolved during evaluation.
