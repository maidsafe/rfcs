- Feature Name: Use public key alone to authorise actions 
- Type Enhancement
- Related components routing, maidsafe_types, maidsafe_client, maidsafe_vaults
- Start Date: 23-06-2015
- RFC PR: 
- Issue number: 

# Summary

At the moment client Id packets are create and a hash of these packets used as identity. 
This involves storing such packets prior to the client interacting with the network. 
As such this requires an initial `unauthorised put` as the client is not known to the 
network and cannot be recognised without this. It also means there are many lookups
to link the hash of the id packet with the client taking the action. 
These lookups are costly it is proposed here that they are not required.  

# Motivation

Storing Id packets is a strain on the network. Lookups are a strain and will slow down
network actions. Id packets themselves may be a security issue (can they be hacked), although
highly unlikely, it is impossible to hack if they do not exist! 

This mechanism does not reduce the total number of clients or types of clients as they all exist 
in an address space of 2^256 and app developers can use multiple Id's in very clever ways, such 
as the SAFE network does to distinguish public id's and private id's. Now these and more are possible.
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
