- Feature Name: Add `self_encryptor` data IO to `safe_ffi` and `safe_launcher` API
- Type: new feature
- Related components: [self\_encryption](https://github.com/maidsafe/self_encryption),
  [safe\_ffi](https://github.com/maidsafe/safe_ffi), [safe\_launcher](https://github.com/maidsafe/safe_launcher)
- Start Date: 08-03-2016
- RFC PR: 
- Issue number: 

# Summary

Currently NFS is exposed via `safe_ffi` and the `safe_launcher` REST API which uses self encryption under the hood. This
RFC is to provide direct access to `self_encryption` for those that don't need/want full file system structures.

# Motivation

App developers will need to directly write self-encrypted chunks to the network and receive a data map back with
information about the chunks. While the Rust lib can be used directly, by exposing it via the launcher REST service
(which appears to be the recommended app-to-safe communication approach), developers can build their own topologies on
top of chunks instead of conforming to the NFS topology. My use case is for multiple formats of video where I don't need
to store it in the hierarchical, discoverable safe drive area. Other use cases could be developing a git backend to the
safe network. Also since NFS implies the use of structured data for metadata, this could be cheaper in terms of safecoin
for those that don't need that extra metadata.

This is the self-encryption equivalent of [the issue](https://github.com/maidsafe/rfcs/issues/77) for supporting core
data types via the launcher API. 

# Detailed design

This design will center around how the `safe_launcher` HTTP API will look. This obviously implies that the operations
need to be exposed via `safe_ffi`. This should be considered an advanced API and documentation for it should carry a
disclaimer encouraging devs to use NFS if they can.

## Self-Encrypted Data Identifier

In order for data to be accessed, a data map must be provided. This can be seen as an identifier to the data. The data
map will be a base 64'd JSON structure. Several options for passing the data map were considered. The size can get
pretty large (including a ~37% increase by base 64-ing it) which would usually preclude its use in the URL or an HTTP
header. However, including it in the body poses problems because we want to send the contents of the actual data as the
request body (like `PUT /nfs/file/:filePath`) but the way encryption has been implemented prevents the easy use of
something novel like `multipart/mixed` as explained in [RFC 2046](https://tools.ietf.org/html/rfc2046#section-5.1). One
day if the launcher API supports proper TLS instead of whole-body NACL crypto this can be revisited. But for now we
simply decide to do a URL or HTTP header approach and make sure that there is no max HTTP header limit on the
`safe_launcher` API web server.

The data map is either a single <= 3k byte array (for small content) or a collection of chunk details. It will be
serialized to JSON for easy language portability and base64'd for easy URL/header embedding. Akin to JWT, fields are no
more than 3 characters to help with size.

JSON data map for embedded content:

```json
{
  "cnt": "somebase64encodedbytearray"
}
```

Fields:

* `cnt` - Base 64 encoded content (yes, this means that since the whole structure is base 64'd it is technically double
  encoded)

JSON data map for sets of chunks:

```json
[
  {
    "num": 12345,
    "hsh": "somebase64encodedbytearray",
    "phs": "somebase64encodedbytearray",
    "len": 12345
  },
  {
    "num": 12346,
    "hsh": "somebase64encodedbytearray",
    "phs": "somebase64encodedbytearray",
    "len": 12345
  }
]
```

This is an array of chunk details and can have as many chunks as necessary. Fields:

* `num` - Equivalent of `ChunkDetails::chunk_num`
* `hsh` - Equivalent of `ChunkDetails::hash`, base 64 encoded
* `phs` - Equivalent of `ChunkDetails::pre_hash`, base 64 encoded
* `len` - Equivalent of `ChunkDetails::source_size`

It is not believed this identifier needs to be encrypted the way the body and query strings must be encrypted today.
Leaving this unencrypted is no different than the path not being encrypted in `PUT /nfs/file` (again, ideally the
launcher would use TLS instead of rolling its own).

Of course, `safe_ffi` can accept more proper data structures and the decoding/deserialization of the data map can just
be part of the HTTP API.

## HTTP Calls

### GET /data/:dataMapIdentifier?offset=123&length=123

Obtain a self encrypted set of data. The data map identifier may be part of the path or it may be sent in the HTTP
header `Safe-Data-Map`. If they are both present and don't match or if neither are present, a 400 error is returned with
the standard error JSON. This request does not have a body.

If it is not found, it is a 404 with the traditional JSON error returned. Otherwise, the response is a 200 with a normal
(encrypted) HTTP data with content type `application/octet-stream`. Ideally it could be streamed, but that is not part
of this RFC because things like `GET /nfs/file` don't properly chunk/stream the response either. At this point it is the
dev's responsibility to stream the data.

The query parameter `offset` is optional and defaults to 0. The query parameter `length` is optional and defaults as the
full length. Ideally these would be proper HTTP `Range` headers but they are in the query string to be consistent with
`GET /nfs/file` (which arguably should be using proper HTTP semantics here and not the query string).

### POST /data/:dataMapIdentifier?offset=123

Write a self encrypted set of data. If this is editing an existing data piece, the data map identifier may be part of
the path or it may be sent in the HTTP header `Safe-Data-Map`. If they are both present and don't match it is a 400
response with a normal JSON error. If this is creating a new data piece no data map identifier should be present.

Ideally the API would stream the data on the way in to a self-encrypted writer but this RFC does not mandate that since
it is not the case with `PUT /nfs/file` (and the way it is whole-body encrypted precludes doing this properly).

An optional `offset` query param may be provided. It can only be provided when a data map identifier is provided (it is
a 400 error otherwise). If it is not provided but the data map identifier is, it is defaulted to the source size (i.e.
to append) if indeed the offset is based on source size and can be programmatically determined without the caller's
help.

Upon successful write, a 200 is returned. This call should respond with the HTTP header `Safe-Data-Map` with the new
data map identifier. Also, the response body should be the (encrypted) non-base64'd version of the JSON data map
identifier.

The reason POST was chosen is that despite the existing service using a combination of `POST /nfs/file` and
`PUT /nfs/file/:path` to do a create-with-data (which is debatable on whether it is the right way), this is a single
method for creating/updating. Also since the data map changes upon write the URL does not represent an unchanging
RESTful resource URL, POST is more appropriate than PUT.

# Drawbacks

Reasons why not:

1. The self-encrypted API is not stable enough to encourage use by app devs. But they might be linking against
   `self_encryption` anyways so API compatibility concerns persist anyways.
1. The Maidsafe devs do not want people leveraging the low level data persistence over HTTP (and thereby possibly
   avoiding the GPL).

# Alternatives

1. Do TLS properly on the API so we can stream data back and forth instead of whole-body NACL crypto
1. Use proper HTTP `Range` headers instead of the primitive query string approach

# Unresolved questions

1. Should we expose truncate?
1. Should we expose raw encrypt/decrypt just in case devs wanted to encrypt some data for the user for other uses?
1. Is the data map that is returned by an offset-based write the full one?
1. How large is the data map if I write a 2GB file?
1. Is it possible for the self encryption piece to append to the end of data (i.e. auto-set the position to the end)
   without the caller providing the end? Or is the data map with source size enough to derive it?
