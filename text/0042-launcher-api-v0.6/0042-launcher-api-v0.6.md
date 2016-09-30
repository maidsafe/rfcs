# SAFE Launcher API v0.6

- Status: active
- Type: new feature, enhancement
- Related components: safe_launcher, safe_core
- Start Date: 01-09-2016
- Discussion: https://forum.safedev.org/t/rfc-42-safe-launcher-api-v0-6/95

## Summary

Expose low level API from launcher to allow applications to allow thrid party applications
to access the building blocks of the SAFE Network.

## Motivation

Exposing low level apis allow dynamic applications to be built on SAFE Network.
Using the StructuredData API third party applications can build and manage their
own data structures to fit their needs.

## Detailed design

Expose low level building blocks via launcher for third party applications, which
will allow building their own data structures. Moreover, dynamic applications can be
built on using the appendable data api.

The low level apis use the FFI interface for low level API as detailed in the
[safe_core low level RFC](https://github.com/maidsafe/rfcs/blob/master/text/0041-low-level-api/0041-low-level-api.md)

APIs for,
- [Structured Data](./api/structured_data.md)
- [Immutable Data](./api/immutable_data.md)
- [Appendable Data](./api/appendable_data.md)
- [Cipher Opts](./api/cipher_opts.md)

are exposed from the launcher through the REST interface.

### Permission

Application must request for `LOW_LEVEL_API` access permission to invoke the low level api to store or
read encrypted data.
Since low level apis can be used to create data in the network, it might be possible where
the data created by the apps cannot be deleted by the user again to retrieve the lost space.
Thus, it makes it important to request user for the permission to access low level apis.

### Unauthorised access

Unauthorised access is granted for reading public data using low level api by default.


### Handle Id

The low level apis return Handle-Id corresponding to the data type that is being worked with.
For example, the structured data will return a DataIdentifier-handle-Id using this Handle-Id,
the operations can be performed on the structured data.

It becomes the applications responsibility to drop the handle after the usage.

### Limitation

Ability to work with versioned data is exposed only from low level APIs. Launcher's
NFS API doesn't support versioning yet.

## Drawbacks

If the handle id is not cleaned up properly it can lead to memory leak. safe_core
can purge if the number of handles goes above a certain threshold.

## Alternatives

Nil

## Unresolved questions

Nil
