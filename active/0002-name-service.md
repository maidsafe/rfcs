- Feature Name: Decentralised Naming System (dns)
- Type new feature
- Related components maidsafe_client, maidsafe_vault, maidsafe_dns
- Start Date: 27-06-2015
- Issue number: #23

# Summary

In the current Internet Domains are referenced via a naming system which comprises an indexing mechanism
This indexing mechanism is centralised and controlled, with the ability for countries and ISPs to switch it off
pollute data with advertising data and much worse. The SAFE network on the other had offers a 'free (as in beer)'
mechanism to locate data linked to a name. This proposal outlines a mechanism to look up data related to any name.

This data includes, long name (could be real name is users wish), web site id, blog id, micro-blog id and may be 
extended to include additional information as the network develops. 

# Motivation

In networks people expect to be able to lookup services related to names. In the SAFE network this includes
the ability to retrieve public keys to encrypt data to that id, in cases where privacy is a requirement (.e.
in SAFE all messages are encrypted between identities). The opportunity for people to create and link web sites
and other services is also a motivation for implementing such a system. 

# Detailed design

This is a simple use case for `Unified Structured Data` (see RFC XXXX). In this case we require to set three items

1. The `type_tag` shall be type `4`

2. The `Identity` shall be the `Sha512` of the publicly chosen name (user chooses this name)

3. The `data` field:

```rust
struct dns {
long_name: String
encrytion_key: crypto::encrypt::public_key
//      service, location
HashMap<String, (NameType, u64, bool, bool)> // As Directories are identified by a tuple of 4 parameters (NameType, tag, if private/encrypted, if versioned) it makes sense to have the 4 identifies stored for better scalability.
}

Initially this `HashMap` will likely contain www, blog and micro-blog. Application developers may add services to enhance 
users experiences with new applications and functions via this mechanism. 

```

This is very likely to extend as the services on the network grows. 

To find this information an application will `Hash(Hash(public_name) + type_tag))` and retrieve this packet.

# Drawbacks

None found at this time.

# Alternatives

Alternatives could be clients, passing this information via a series of messages, but this requires both parties
being on-line or able to wait for each other to come on line.

# Unresolved questions

1. Should this struct contain the actual public_name (handle) as this may allow some indexing mechanism, which is
prevented with this design (on purpose).

2. This prevents two identities using the same name, is this really the best way to go. This is more like twitter
than facebook in approach. 
