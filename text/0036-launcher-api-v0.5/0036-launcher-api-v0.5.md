# Title-cased Name of the Feature

- Status: proposed
- Type: new feature, enhancement
- Related components: safe_launcher
- Start Date: 25-05-2016)
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

# Summary

Adding new features to the safe_launcher API to make it more developer friendly and also to
incorporate the standards that were missing in the 0.4 version of the API.

# Motivation

New features to the existing API can improve handling large data volume in an efficient manner.
This can have a significant improvement in the performance of the launcher and the safe-core ffi
for handling large data volume. Incorporating the standards in the API will also improve the
stability of the APIs and make it easier for third party applications to integrate.

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

###### On Success

####### Status code
`200`

####### Body
```
{
  token: String, // JWT token
  permissions: Array[String]  
}
```

###### On Failure

####### Status code
`401`

### Remove unnecessary Base64 conversions

At present the APIs exchange data as base64 strings, which is unnecessary while
the raw data can be directly sent over HTTP. The request and response can
directly use the actual content based on the `Content-type`.

For example, the APIs which accept the JSON payload can simply `POST/PUT` the JSON payload
without any encoding or encryption. Like wise the response will also be a plain JSON String.

For the APIs which involves raw data such as the file upload, the binary can be
directly sent instead of encrypting and then encoding to a base64 string.

Removal of base64 encoding has to be handled across all the APIs

### Renaming custom headers

Rename custom headers to start with `X-` as mentioned in the [RFC](http://www.ietf.org/rfc/rfc2047.txt).
At present the custom headers used in the [NFS](https://maidsafe.readme.io/docs/nfs-get-file#section-response)
and [DNS](https://maidsafe.readme.io/docs/dns-get-file-unauth#section-response) while fetching a file does
not follow the standards. And any custom header used should follow the naming convention.

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
|srcPath| Source path which has to be copied or moved|
|isSrcPathShared| Boolean value to indicate whether the source path is shared or private. Defaults to false|
|destPath| Destination path to which the source directory must be copied or moved|
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
|destPath| Destination path to which the source directory must be copied or moved|
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
|X-Created-On| created data time in UTC |
|Last-Modified| last modified date in UTC |

```
Accept-Ranges: bytes
Content-Length: Number
Last-Modified: DATE in UTC
X-Created-On: DATE in UTC
```

### Streaming API

### Using Content Range Header

#### Response headers (206)


# Drawbacks

Why should we *not* do this?

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
