# safe_launcher API v0.5

- Status: active
- Type: new feature, enhancement
- Related components: safe_launcher
- Start Date: 25-05-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/141
- Supersedes:
- Superseded by:

# Summary

Adding new features to the safe_launcher API to improve efficiency and also to
incorporate the standards that were missing in the 0.4 version of the API.

# Motivation

New features to the existing API can improve handling large data volume efficiently
using streaming APIs. This can have a significant improvement in the performance
of the launcher and the safe-core ffi for handling large data volume.
Incorporating the standards in the API will also improve the stability of the APIs
and make it easier for third party applications to integrate.

# Detailed design

Categorising the proposals into two sections `Enhancements` and `New API Features`.
These features are also based on the suggestion from [cretz](https://forum.safenetwork.io/users/cretz/activity)
in the [forum thread](https://forum.safenetwork.io/t/safe-launcher-dev-issues-suggestions/7890)

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
without base64 encoding or encryption. Like wise the response will also be a plain JSON String (UTF-8).

For the APIs which involves binary data such as the file upload, the binary data
can be directly sent instead of encrypting and then encoding to a base64 string.

**Removal of base64 encoding has to be handled across all the APIs**

### Parameter Validation

The REST API must ensure that the parameters are validated before sending the request
through the FFI. The FFI errors mut be resolved internally and proper error codes must be
sent to caller instead of 500 Status code.

As linked in the [issue](https://github.com/maidsafe/safe_launcher/issues/144), the parameters
must be validated according to needs of the API. For example, the DNS API must also validate the service
name and public name to be alphanumeric and can contain `-`.

### Using Single Client

Launcher uses two separate client instances for handling authorised and unauthorised requests.
Launcher should use only one client instance to do the network related operations.

The NFS API in safe_core must be modified to support this change. The NFS APIs, [uses the
encryption keys](https://github.com/maidsafe/safe_core/blob/master/src/nfs/helper/directory_helper.rs#L159)
based on the client object. Instead the APIs must be modified [as in DNS](https://github.com/maidsafe/safe_core/blob/master/src/dns/dns_operations/mod.rs#L67)
to accept `Option<(&public_key::PublicKey, &secret_key::SecretKey, &nonce::Nonce)`

The FFI can decide whether to pass the keys or not based on the need for encryption/decryption.
If the keys are passed, then the NFS API can use the keys to decrypt/encrypt, else the
data is saved for public read.

### CORS validation

The CORS validation must be based on the request origin for the XHR Requests.
Allow XHR requests only if the `Origin` ends with `.safenet` extension. At present CORS
is enabled for all XHR requests.

### CSP Headers on Web Proxy

Enforcing CSP headers on all responses through the proxy helps to mitigate security threats
on the web clients.

```
Content-Security-Policy : default-src self *.safenet; object-src none; base-uri self; form-action http://api.safenet; frame-ancestors self;
X-Frame-Options : SAMEORIGIN
```

[CSP Level 2](http://content-security-policy.com/) policy `frame-ancestors` is supported only on
chrome and firefox. Adding `X-Frame-Options` headers will act as a fallback for other browsers.

#### Launcher workflow for handling the FFI client handle

When the launcher is started, the unauthorised client is created. This will allow
applications to read public data without the need of user logging in.
Once the user logs in successfully, the unauthorised client handle must be dropped and
the authorised client handle obtained should be used.

### Gulp script for updating error codes (npm run update-error-codes)

FFI interface returns error codes as return value for every method call. The errors must
be looked up based on the error code from the client modules (Core, NFS and DNS).
A gulp script must be integrated to fetch the error codes from the master branch of the `safe_core` and build the [`error_code_lookup.js`](https://github.com/maidsafe/safe_launcher/blob/master/app/server/error_code_lookup.js) file.

The gulp build task will look for the safe_core project on the local machine based on
a path from the gulpfile.
If the `safe_core` project is located, then the error_code_lookup.js file is updated based on the local safe_core
source. Else the `error_code_lookup.js` file wont be updated by the script.

## New API Features

### NFS API

All the existing NFS API have endpoint changes. New APIs for move/copy directory, metadata request are added.
The detail documentation of the NFS API is updated in the [supporting document](./0036-nfs-api-v0.5.md)

### Streaming Support for API

The launcher API can not to handle large file sizes as it requires to read the entire content
in memory to upload the whole file content and also in a chunked upload there are excess data chunks
being created in the network when a large file is saved.

Exposing a streaming API can improve the efficiency to handle larger data upload.
For creating and reading binary data, the APIs must be able to support streaming (read / write).

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
and drift towards a standard approach for partial read/write operations.

Example usage,
```
Range: bytes=0-
Range: bytes=0-100
```

**If the range header is not specified, the entire file is streamed while reading and
the data is appended to the end while writing.**

### Response headers

#### File Read

If the entire file is streamed then the Status code returned will be `200`.
If only a partial section of the file is read, then the status code will be `206`.

Response headers,
```
Accept-Ranges: bytes
Content-Length: <Length that is requested based on the byte range>
Content-Type: <mime type if available based on the extension or application/octet-stream>
Content-Range: bytes <START>-<END>/<TOTAL>
```

#### File write

Status code `200` will be returned on success.

### Streaming issue in the Web platform (Browser based clients)

Streaming over HTTP is out of the box supported in most of the platforms. Similarly,
web browsers also provide support for the streaming data using the default widgets (audio / video controls) provided.

Could not find an out of the box option for streaming upload of large data. The available
options to write huge files is to use the HTML Form or FormData and send using multipart upload.
The other option was to write data in chunks to the server, that again is not an ideal
solution, since the client has to create many short lived connections for uploading the data in smaller chunks.

Thus the NFS file content upload API must be able to support multipart upload. The API
would consider the upload only for one file at a time, i.e, can not upload the file contents
of multiple files at one go. The API will be reading the data only for one file and
close the response accordingly if it is a multipart request.

### Account Status Display

At present there is no means for the user to know the space that has been utilised. Presenting
an UI for displaying the storage stats for the logged in account would be useful. The stats would
be fetched when the launcher starts and the user can check the latest stats by using a refresh option.

A refresh option is provided instead of polling to avoid more network traffic at this time.
Once the messaging is implemented, real time updates for the account status can be implemented.

### Manage Proxy from login page

By default, the proxy is started when the launcher starts. Provide option to disable the proxy before
even logging in to the launcher. Until a dedicated browser for SAFE Network is in place, we would have to
depend on proxy for facilitating web apps.

The settings should be persistent and while starting the launcher next time, the proxy
server starting up should depend on the user's settings.

# Drawbacks

TLS option is not considered in this version.

# Alternatives

### Additional API can be added to facilitate the streaming uploads from the web browsers
1. The local launcher server can also listen for web socket connections at the same launcher port.

2. The client will call an api (PUT /nfs/file/worker/:isPathShared/:filePath/). The API will get the metadata
to locate the file as a part of the request and get the self_encryptor handle for writing the file contents
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

### File Metadata Request (HEAD)

The file metadata request sends the response in in the header fields. As an alternate approach,
instead of the headers the metadata can be sent as a JSON response. [AWS](http://docs.aws.amazon.com/AmazonS3/latest/API/RESTObjectHEAD.html)
uses the headers approach while [GCS](https://cloud.google.com/storage/docs/json_api/v1/objects/get#parameters) uses JSON response.
Since the fields that are sent back is minimal, the preference was for using the headers.

# Unresolved questions

Nil
