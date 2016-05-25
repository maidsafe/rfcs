# safe_launcher API v0.5

- Status: proposed
- Type: new feature, enhancement
- Related components: safe_launcher
- Start Date: 25-05-2016)
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

# Summary

Adding new features to the safe_launcher API to improve efficiency and also to
incorporate the standards that were missing in the 0.4 version of the API.

# Motivation

New features to the existing API can improve handling large data volume in an efficiently
using streaming APIs. This can have a significant improvement in the performance
of the launcher and the safe-core ffi for handling large data volume.
Incorporating the standards in the API will also improve the stability of the APIs
and make it easier for third party applications to integrate.

# Detailed design

Categorising the proposals into two sections `Enhancements` and `New Features`.

## Enhancements

### Remove session based encryption between application and launcher

Since the launcher and the integrating applications are running on the local machine,
the session based encryption during the data transfer is a mere over head. Removing the
session based encryption can have a significant improvement in the performance.
However the JWT tokens for validating and authorising the applications would still hold
good. The authorisation process workflow will be the same and the applications
have to pass the JWT token in the `authorization` header for authorised requests.
if the user denies authorisation, Unauthorised (401) status code is returned.

**Authorised requests need no encryption for parameters or even the payload. But must
pass the authorisation token in the request header**

#### Authorisation Request

##### Request

###### Endpoint
`/auth`

###### Method
`POST`

###### Body
```
{
  app: {
    name: String,
    version: String,
    vendor:  String,
    id: String
  },
  permissions: Array[String]
}
```

##### Response

###### Status code
```
200
```

###### Body
```
{
  token: String, // JWT token
  permissions: Array[String]  
}
```

### Remove unnecessary Base64 conversions

At present the APIs exchange data as base64 strings, which is unnecessary while
the raw data can be directly sent over HTTP. The request and response can
directly use the actual content based on the `Content-type`.

For example, the APIs which accept the JSON payload can simply `POST/PUT` the JSON payload
without any encoding or encryption. Like wise the response will also be a plain JSON String.

For the APIs which involves raw data such as the file upload, the binary can be
directly sent instead of encrypting and then encoding to a base64 string.

Removal of base64 encoding has to be handled across all the APIs

### Custom headers naming convention
Rename custom headers to start with `X-` as mentioned in the [RFC](http://www.ietf.org/rfc/rfc2047.txt)

## New Features

### NFS APIs

#### Move/Copy Directory
API to move or copy the directory from source to a destination.
The source path and destination path must already exists or Bad Request(400) error will
be returned.

##### Request

###### Endpoint
`/nfs/movedir`

###### Method
`POST`

###### Header
```
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

###### Body

|Field|Description|
|-----|-----------|
|srcPath| Source directory path which has to be copied or moved|
|isSrcPathShared| Boolean value to indicate whether the source path is shared or private. Defaults to false|
|destPath| Destination directory path to which the source directory must be copied or moved|
|isSrcPathShared| Boolean value to indicate whether the source path is shared or private. Defaults to false|
|action| ENUM value - MOVE or COPY. Defaults to MOVE|

```
{
  "srcPath": String,
  "isSrcPathShared": Boolean,
  "destPath": String,
  "isDestPathShared": Boolean,
  "action": ENUM (MOVE or COPY)
}
```

##### Response

###### Status code
```
200
```

#### Move/Copy File

API to move or copy the file from source directory to a destination directory.
The source path and destination path must already exists or Bad Request(400) error will
be returned.

##### Request

###### Endpoint
`/nfs/movefile`

###### Method
`POST`

###### Header
```
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

###### Body

|Field|Description|
|-----|-----------|
|srcPath| Source path which has to be copied or moved. eg, `/a/b/c.txt`|
|isSrcPathShared| Optional value. Boolean value to indicate whether the source path is shared or private. Defaults to false|
|destPath| Destination path to which the file must be copied or moved. eg, `a/b`|
|isSrcPathShared| Optional value. Boolean value to indicate whether the source path is shared or private. Defaults to false|
|action| Optional value. ENUM value - MOVE or COPY. Defaults to MOVE|

```
{
  "srcPath": String,
  "isSrcPathShared": Boolean,
  "destPath": String,
  "isDestPathShared": Boolean,
  "action": ENUM (MOVE or COPY)
}
```

##### Response

###### Status code
```
200
```

#### File Metadata

##### Request

###### Endpoint
|Field|Description|
|-----|-----------|
|filePath| Full file path. Must be URL encoded|
|isPathShared| Optional Value. Boolean value to indicate whether the path is shared or private. Defaults to false|

```
/nfs/file/:filePath/:isPathShared
```

###### Header
Required only for private data

```
Authorization: Bearer <TOKEN>
```

###### Method
```
HEAD
```

##### Response

###### Status Code
```
200
```

###### Header
|Field|Description|
|-----|-----------|
|Accept-Ranges| Refers to the range accepted in the Range header|
|Content-Length| Size of the file in bytes|
|X-Created-On| created data and time in UTC |
|Last-Modified| Last modified date and time in UTC |
|X-Metadata| Present only if the metadata is available. Metadata as base64 String|

```
Accept-Ranges: bytes
Content-Length: Number
Last-Modified: DATE in UTC
X-Created-On: DATE in UTC
X-Metadata: base64 string
```

### Refactor Read File Response Headers

The get file response of the [NFS](./text/0018-launcher-as-rest-server/nfs_api.md#response-headers-5) API has custom headers. Remove the custom headers because the metadata request (HEAD) can be used
to fetch the file metadata and use the GET file requests only for reading the content of the file.

### Streaming API

Right now the launcher API is not efficient to handle large file sizes. There are
excess chunks being created when a large file is saved. Moreover, this requires the entire
content to be held in the memory and sent to the ffi interface.

Exposing a streaming API can improve this inefficiency along with the changes in the ffi interface,
to support the same.

While creating and reading binary data, the APIs must be able to support streaming (read / write).

Nodejs exposes [Stream API](https://nodejs.org/api/stream.html) for creating a custom Read / Write streams.
The GET APIs must be able to serve the data from the network using a readable stream,
while the PUT/POST APIs must be able to use a writable stream to pipe the received data to the network.

The FFI interface must pass the self_encryption handle to the caller and the caller can use the
handle to read and write the data. After the operation is complete, the caller must call the ffi
api to drop the handle.

#### APIs that must support streaming

GET APIs from [NFS](./text/0018-launcher-as-rest-server/nfs_api.md#read-file)
and [DNS](./text/0018-launcher-as-rest-server/dns_api.md#get-file-unregistered-client-access)
must be able to serve data using the readable streams.

[NFS API to update file content](./text/0018-launcher-as-rest-server/nfs_api.md#update-file-content)
must be able to read the incoming data stream and write the data to the network using a
writable stream.

### Using Content Range Header

Using the `Range` HTTP header can help in removing the offset and length parameters
and drift towards a standard approach for chunked read/write operations.

Example usage,
```
Range: bytes 0-
Range: bytes 0-100
```

If the range header is not specified then the entire file is streamed while reading and
while writing the data is appended from the 0th position.

### Response headers

#### File Read

If the entire file is streamed then the Status code returned will be `200`.
If only a partial section of the file is read then the status code will be `206`.

#### File write

Status code `200` will be returned on success.

### Streaming issue in the Web platform

Streaming over HTTP is out of the box supported in most of the platforms. Similarly,
web browsers also provide support for the streaming data using the default widgets provided.

Could not find an out of the box option for streaming upload of large data. The available
options to write huge files is to user the HTML Form or FormData and send using multipart upload.
The other option was to write data in chunks to the server, that would again be an inefficient
approach in our case.

Thus the NFS file content upload API must be able to support multipart upload. The API
would consider the upload only for one file at a time, i.e, can not upload the file contents
of multiple files at one go. The API will be reading the data only for one file and
close the response accordingly if it is a multipart request.

# Drawbacks

Nil

# Alternatives

Additional API can be added to facilitate the streaming uploads from the web browsers.
It can be a two legged process,

1. The local launcher server can also listen for web socket connections at the same launcher port.

2. The client will call an api (PUT /nfs/file?worker). The API will get the metadata
to locate the file as a part of the request and get the self_encryption handle for writing the file contents
and hold the handle in memory(HashMap<UUID, SE_HANDLE>) associating to a random ID. The random ID
is sent as part of the response.

3. Once the client gets the ID, it can spawn a WebWorker and start streaming the data through the websocket connection.
  ```
  {
    type: 'STREAM',
    id: <Random ID received>,
    data: <Array Buffer>
  }
  ```
  when the data is received the server can validate the ID and write the data to
  network.

# Unresolved questions

Nil
