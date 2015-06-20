- Feature Name: Remove Transaction Managers from network and have network only recognise 2 Structured data sub-types 
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client, sentinel
- Start Date: 13-06-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Have network only recognise two primary data types, Immutable and Structured. These types will have tag_ids
to allow them to contain several data types that can be used in the network by users of the client interface.
This does mean a change to default behaviour and is, therefore a significant change. ImmutableData has already two sub-types (Backup and Sacrificial). StructuredData will have two sub types, `fixed` and `transferrable`. This proposal should simplify the sentinel and interfaces from routing to users of routing as there will be no need to pass down type information (i.e. how to get the name or owner etc.). These types can actually be defined in the routing library, allowing users of the library to use the `type_tag` to create their own types and actions on those types. 

# Motivation

##Why?

The primary goal is two fold, reduce network traffic (by removing an indirection, of looking up a value 
and using that as a key to lookup next) and also to remove complexity (thereby increasing security).

Another facet of this proposal is extendability. In networks such as SAFE for isntance, client app developers can define their own types (say of the `fix` protocol for financial transactions) and instanciate this type on the network. For users creaatign their own network they may whitelist or blacklist types and type_id's as they wish, but the possibility would exist for network builders (of new networks) to allow extensability of types.  

##What cases does it support?

This change supports all use of non immutable data (structured data). This covers all all non `content only` data
on the network and how it is handled. 

###Data storage and retrieval

ImmutableData is fixed self validating non mutable chunks. These require StructuredData types to manipulate information. These structured Data types may then create a global application acting on a key value store with very high degrees of availablity and security (i.e. create network scale apps). Such apps could easily include medical condition analysis linked with genomic and protiomic sequencing to advance health based knowledge on a global scale. This proposal allows such systems to certainly be prototyped and tested with a high degree of flexability. 

###New protocols

As these types are self validating and may contain different information, such as new protocols, `rdf`/`owl` data types then the limit of new data types and ability to link such data is extremely scalable. Such protocols could indeed easily encompass token based systems (a form of 'crypto-currency'), linked data, natural language learning databases, pre-compilation units, distributed version control systems (git like) etc.

###Compute

Such a scheme would allow global computation types, possibly a Domain Specific Language (DSL) would define operator types to allow combination of functions. These could be made monotonic and allow out of order processing of programs (disorderly programming) which in itself presents an area that may prove to be well aligned with decentralised 'intelligence' efforts. Linked with 'zk-snarks' to aleviate any 'halting problem' type issues then a global turing complete programming enviroment that acts on semantic ('owl' / 'json-ld' etc.) data is a potential outcome. 

##Expected outcome

It is expected this will reduce complexity, code and increase security on the network, whilst allowing a greater degree of flexability. 

# Detailed design

The design entails reducing all StructuredData types to two sub-types, therefore it should be able to
be recognised by the network as StructuredData and all such sub-types handled exactly in the same manner. the sub types are defined here:

##FixedStructuredData

```
struct FixedStructuredData {
type : TagType, // 64 Bytes
data : mut Vec<u8>, // in many cases this is encrypted
owner_keys : vec<crypto::sign::PublicKey> // n * 32 Bytes (where n is number of owners)
version : mut u64, // incrementing (deterministic) version number
signature : mut Vec<Signature> // signs the fields above // 32 bytes (using e2559 sig)
}
```
__Size of raw packet minus data is 192Bytes leaving 320Bytes if restricted to 512 Bytes__

Fixed (immutable fields) 
- type
- owner_keys

##Validation 

- To confirm name (storage location on network) we SHA512(TagType + owner_keys (concatanted))
- To validate data we confirm signature using hash of (tag_type + version) as nonce. 
- To confirm sender of any `Put` (store or overwrite) then we check the signature of sender using same mechanism. For multiple senders we confirm at least 50% of owners have signed the request for `Put`

When `Put` on the network this type is `FixedStructuredData` with a subtype field. The network ignores this subtype except for collisions. No two data types with the same name and type can exist on the network. 

##TransferableStructuredData

```
struct TransferableStructuredData {
type : TagType, // 4 Bytes ?
Identifier : NameType // 64Bytes
data : mut Vec<u8>, // in many cases this is encrypted
owner_keys : mut vec<crypto::sign::PublicKey> // n * 32 Bytes (where n is number of owners)
version : mut u64, // incrementing (deterministic) version number
signature : mut Vec<Signature> // signs the fields above // 32 bytes (using e2559 sig)
}
```
__Size of raw packet minus data is 192Bytes leaving 320Bytes if restricted to 512 Bytes__

Fixed (immutable fields) 
- type
- identifier

##Validation 

- To confirm name (storage location on network) we SHA512(TagType + Identifier)
- To validate data we confirm signature using hash of (tag_type + version) as nonce. 
- To confirm sender of any `Put` (store or overwrite) then we check the signature of sender using same mechanism. For multiple senders we confirm at least 50% of owners have signed the request for `Put`

When `Put` on the network this type is FixedStructuredData with a subtype field. The network ignores this subtype 
except for collisions. No two data types with the same name and type can exist on the network. 

These types are stored in a disk based storage mechanism such as `Btree` at the NaeManagers (DataManagers) responsible for the area of the network at that `name`. 

## common features 

These types will be limited to 100kB in size (as Immutable Chunks are also limited to 1Mb) which is for the time being a magic number.

If a client requires these be larger than 100kB then the data component will contain a (optionally encrypted) datamap to be able to retrieve chunks of the network. 

The network will accept these types if `Put` by a Group and contains a message signed by at least 50% of owners as indicated. For avoidance of doubt 2 owners would require at least 1 have signed, 4 owners would require at least 2 etc. for majority control use an odd number of owners. Any `Put` must obey the mutability rules of these types.
To update such a type the client will `Put` again (paying for this again) and the network will overwrite the existing data element if the request is signed by the owner and comes via a group (ClientManagers). 

For private data the data filed will be encrypted (at client discresion), for public data this need not be the case as anyone can read that, but only the owner can update it. 

##Security

### Replay attack avoidance

The inclusion of the version number will provide resistance to replay attacks.

### GetKey removal

The removal and validation of client keys is also a significant reduction in complexity and means instead of lookups to get keys, these keys are included as part of the data. This makes the data self validating and reduces security risks from sparticus type attacks. It also removes ability for any key replacement attack on data.

# Drawbacks

This will put a heavier requirement on `Refresh` calls on the network in times of churn, rather than transferring only keys and versions (which may be small) this will require sending up to 100kB per StructuredData element. If content was always held as immutable data then it is not transferred at every churn event.
The client will also have more work as when the StructuredData type is larger than 100kB it then has to self_encrypt the remainder and store the datamap in the data filed of the StructuredData. This currently happens in a manner, but every time and without calculation of when.

# Alternatives

Status quo is an option and realistic. 

- Another option posed by Qi is that structured data types be directly stored on `ManagedNodes` (`PmidNode`), this would reduce churn issues, but may introduce a security concern as we do not trust a `Pmidnode` to not lie or replay and it potentially can with data that is mutable and not intrinsically validatable as accurate. 
- Another possibility by Qi is the `PmidManager` (`NodeManager`) stores the data size separate from the immutableData size.  
  

# Unresolved questions

1. size of StructuredData packet, it would be nice if it were perhaps 512Bytes to have the best chance to fit into a single UDP packet, although not guaranteed. Means a payload after serialisation of only a few hundred bytes (maybe less)
2. Version conflics or out of order updates, will upper layers handle this via a wait condition?
