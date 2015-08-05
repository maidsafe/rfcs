- Feature Name: Handle larger file sizes.
- Type new feature
- Related components self_encryption
- Start Date: 05-08-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

The purpose of this RFC is to amend the storage strategy for immutable data, data from here on, either destined for or retrieved from the network. The data in question, associated with a DataMap going through the self encryption process, must be maintained in some form, it is this part of the process we are concerned with here.

# Motivation

A significant portion of network traffic is anticipated to result from the storage/retrieval of data coming through self encryption. It is therefore necessary to have robust, secure and efficient methods in place for handling the data arriving at any connected device. Tied to this are the parameters associated with the devices themselves, for our purpose mainly storage and memory capacities. By introducing a memory map we can increase the size of files that can be handled by self_encryption.

# Detailed design

Currently all data associated with a DataMap going through self encryption is held unencrypted in memory. A close call on a SelfEncryption, SE, object initiates encryption and storage of the data via the Storage object passed on creation. In the current iteration it is proposed to maintain a vector of data per file up to a 50Mb limit in memory, while for files that grow larger than this to swap over to using an anonymous memory map up to 1Gb on 32-bit systems and 10Gb on 64-bit systems across all architectures. No noticeable difference should occur for file sizes less than the 50Mb limit already handled in the current implementation, and throughput is expected to be limited on a system by system basis to the number of files that can be open at any given time, clearly this number decreases with increasing file size.    

# Drawbacks

The 1Gb limit on file size for 32-bit systems is restrictive, however, this will be addressed in some future RFC.

# Alternatives

Allow unrestricted file sizes across devices by introducing a cache backed sliding window algorithm. The sliding window here containing NMb of in memory data per file, for some architecture dependent integer N. 

# Unresolved questions

None foreseen.
