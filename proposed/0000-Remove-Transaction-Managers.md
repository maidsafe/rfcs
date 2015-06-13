- Feature Name: Remove Transaction Managers from network and have network only recognise 2 data types 
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client
- Start Date: 13-06-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Have network only recognise two primary data types, Immutable and Structured. These types will have tag_ids
to allow them to contain several data types that can be used in the network by users of the client interface.
This does mean a change to default behaviour and is, therefore a significant change.

# Motivation

##Why?

The primary goal is two fold, reduce network traffic (by removing an indirection, of looking up a value 
and using that as a key to lookup next) and also to remove complexity (thereby increasing security).

##What cases does it support?

This change supports all use of non immutable data (structured data). This coverall all non content data
on the network and how it is handled. 

##Expected outcome

It is expected this will reduce complexity, code and increase security on the network. 


# Detailed design

The design entails reducing all StructuredData types to a consistent Super Type, therefore it should be able to
be recognised by the network as StructuredData and all such types handled exactly in the same manner. This type 
would be a relatively simple type, this is defined here:

```
struct StructuredData {
type : TagType, // 64 Bytes
name : NameType, // 64 Bytes
data : Vec<u8>,
signature : Signature // signs the fields above // 32 bytes (using e2559 sig)
owner_key : crypto::sign::PublicKey // 32 Bytes
}
```
__Size of raw packet minus data is 192Bytes leaving 320Bytes if restricted to 512 Bytes__

When `Put` on the network this type is StructuredData with a subtype field. The network ignores this subtype 
except for collisions. No two data types with the same name and type can exist on the network. The network will
accept these types if `Put` by a Group and contains a message signed by the owner as indicated. 

These types are stored in a disk based storage mechanism such as `Btree` at the NaeManagers (DataManagers) responsible 
for the area of the network at that `name`. 

These types will be limited to 100kB in size (as Immutable Chunks are also limited to 1Mb) which is for the time being 
a magic number.

If a client requires these be larger than 100kB then the data component will contain a datamap to be able to retrieve 
chunks of the network. 

To update such a type the client will `Put` again (paying for this again) and the network will overwrite the existing 
data element if the request is signed by the previous owner and comes via a group (ClientManagers). This may alter the 
owner to a new owner and that is allowed. 

For private data the data filed will be encrypted (at client discresion), for public data this need not be the case as 
anyone can read that, but only the worked can update it. 


##Security

This code reduction will increase security, but will require that StructuredData, be passed via the Sentinel in routing.
This is also a requirement for current structured Data types in any case to ensure group consensus on those types. The 
security is not relate to tampering with the data (as it is signed), but instead for replay attacks by a bad close
group node acting as a `DataManager`. To prevent this consensus is used. A monotonic function could be considered if the 
client had a notion of the direction of the monotonic output.  


# Drawbacks

This will put a heavier requirement on `Refresh` calls on the network in times of churn, rather than transferring only keys and
versions (which may be small) this will require sending up to 100kB per StructuredData element. If content was always held as
immutable data then it is not transferred at every churn event. 
The client will also have more work as when the StructuredData type is larger than 100kB it then has to self_encrypt the 
remainder and store the datamap in the data filed of the StructuredData. This currently happens in a manner, but every 
time and without calculation of when.

# Alternatives

Status quo is an option and realistic. 

- Another option posed by Qi is that structured data types be directly stored on `ManagedNodes` (`PmidNode`), this would 
reduce churn issues, but may introduce a security concern as we do not trust a `Pmidnode` to not lie or replay and 
it potentially can with data that is mutable and not intrinsically validatable as accurate. 
- Another possibility by Qi is the `PmidManager` (`NodeManager`) stores the data size separate from the immutableData size.  
  

# Unresolved questions

1. size of StructuredData packet, it would be nice if it were perhaps 512Bytes to have the best chance to fit into a single 
UDP packet, although not guaranteed. Means a payload after serialisation of only a few hundred bytes (maybe less)
