- Feature Name: Reserved name list
- Type enhancement
- Related components maidsafe_client, maidsafe_vault
- Start Date: 27-06-2015
- RFC PR: 
- Issue number: 

# Summary

A very simple RFC to outline and maintain (over time) a known list of `type_tags` for `StructuredData` types.

# Motivation

As with `IANA` type registries or indeed the common well known service ports found in many Operating Systems (OSs)
this list will allow applications that achieve the happy position of being commonplace, or extremely popular to 
register their `type_tag` in this file. This allows othr application developers to choose a non registered tag to 
reduce any confusion or name collisions (although the latter is extremely improbable, given the address space).

# Detailed design

0            --      Login session
1            --      Private Directory
2            --      Public directory
3            --      Shared Directory
4            --      dns entry
5
6
7            --      safecoin
8-10,000     --      Reserved
10,001
2^64         --      Available

The numbers 0-10,000 will be hard coded into vaults for the use of the core network only and will not be likely 
available to application developers to overwrite their structure. 

# Drawbacks

None known

# Alternatives

Alternatively this list could not exist and there could be a free for all approach. 

# Unresolved questions

1. The range 0-10,000 is a subjective one and may not be correct.
