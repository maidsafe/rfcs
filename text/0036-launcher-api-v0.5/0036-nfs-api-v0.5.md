# NFS API

This is a supporting a document for the parent [RFC](./0036-launcher-api-v0.5.md).
This details the NFS API changes and also the new MOVE/COPY APIs that are getting added.

## Directory

### Create Directory

#### Request

##### Endpoint
```
/nfs/directory
```

##### Method
```
post
```

##### Headers
```
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

##### Body
|Field|Description|
|-----|-----------|
|dirPath| Full directory path as String. Example - /home, /home/photos|
|isPathShared| Boolean value to indicate whether the path is shared from SAFEDrive or from the application directory. Optional. Defaults to false|
|metadata| Optional String. Metadata as UTF-8 String|

```
{
  dirPath: String,
  isPathShared: Boolean,  
  metadata: String
}
```

#### Response

##### On Success

###### Status Code
```
200
```

##### On Failure

###### Status Code
```
401 / 400
```

### Get Directory

#### Request

##### Endpoint
|Field|Description|
|-----|-----------|
|isPathShared| Boolean value to indicate whether the path is shared from SAFEDrive or from the application directory|
|dirPath| Full directory path as String. Example - /home, /home/photos|

```
/nfs/directory/{isPathShared}/{dirPath}
```

##### Method
```
GET
```

##### Headers
```
Authorization: Bearer <TOKEN> // Optional for public directory
```

### Response

#### Headers
```
content-type: application/json
status: 200 Ok
```

#### Body

```javascript
{
    "info" {
      "name": String,
      "creationTime": Integer, // milliseconds            
      "modificationTime": Integer, // milliseconds            
      "isPrivate": Boolean,      
      "metadata": base64 String,
    },
    "subDirectories": [
        {
            "name": String,
            "creationTime": Integer, // milliseconds            
            "modificationTime": Integer, // milliseconds
            "isPrivate": Boolean,            
            "metadata": base64 String
        },
        ...
    ],
    "files": [
        {
            "name": String,
            "size": Integer,                
            "creationTime": Integer, // milliseconds            
            "modificationTime": Integer, // milliseconds
            "metadata": base64 String
        },
        ...
    ]
}
```

### Update Directory Metadata

#### Request

##### End point
```
/nfs/directory/{isPathShared}/{dirPath}
```

##### Method
```
PUT
```

##### Headers
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Body
```
{
    "name": String,
    "metadata": base64 string
}
```

#### Response

##### Headers
```
status: 200 Ok
```

### Move/Copy Directory

API to move or copy the directory from source to a destination.
The source path and destination path must already exists or Bad Request(400) error will
be returned.

#### Request

##### Endpoint
`/nfs/movedir`

##### Method
`POST`

##### Header
```
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

##### Body

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

#### Response

###### Status code
```
200
```

### Delete Directory

#### Request

##### End point
```
/nfs/directory/{isPathShared}/{dirPath}
```

##### Method
```
DELETE
```

##### Headers
```
Authorization: Bearer {TOKEN}
```

##### Response

###### Headers
```
status: 200 ok
```

## File

### Create File

#### Request

##### End point
```
/nfs/file
```

##### Method
```
POST
```

##### Header
```
Content-Type: application/json
Authorization: Bearer {TOKEN}
```

##### Body

```
{
    "filePath": String,
    "isPathShared": Boolean, // Optional - defaults to false
    "metadata": base64 string // Optional
}
```

#### Response

##### Headers
```
Status: 200 Ok
```

### Get File Metadata

#### Request

##### Endpoint
|Field|Description|
|-----|-----------|
|filePath| Full file path. Eg, /home/docs/sample.txt|
|isPathShared| Boolean value to indicate whether the path is shared or private.|

```
/nfs/file/:isPathShared/:filePath
```

##### Header
Required only for private data

```
Authorization: Bearer <TOKEN>
```

##### Method
```
HEAD
```

#### Response

##### Status Code
```
200
```

##### Header
|Field|Description|
|-----|-----------|
|Accept-Ranges| Refers to the range accepted in the Range header|
|Content-Length| Size of the file in bytes|
|Created-On| Created date and time in UTC |
|Last-Modified| Last modified date and time in UTC |
|Metadata| Present only if the metadata is available. Base64 String|

```
Accept-Ranges: bytes
Content-Length: Number
Last-Modified: DATE in UTC
Created-On: DATE in UTC
Metadata: base64 string
```
### Read File

#### Request

##### Endpoint
|Field|Description|
|-----|-----------|
|filePath| Full file path|
|isPathShared| Boolean value to indicate whether the path is shared or private.|

```
/nfs/file/:isPathShared/:filePath
```

##### Header
Required only for private data

```
Authorization: Bearer <TOKEN>
Range: bytes=0- // Optional range for partial read
```

##### Method
```
GET
```

#### Response

##### Status Code
```
200 or 206
```

##### Header
```
Accept-Ranges: bytes
Content-Length: <Length that is requested based on the byte range>
Content-Type: <mime type if available based on the extension or application/octet-stream>
Content-Range: bytes <START>-<END>/<TOTAL>
Last-Modified: DATE in UTC
Created-On: DATE in UTC
Metadata: base64 string
```

##### Body
```
Binary data
```

### Update File Metadata

##### Request

###### End point
|Field|Description|
|-----|-----------|
|filePath| Full file path|
|isPathShared| Boolean value to indicate whether the path is shared or private.|

```
/nfs/file/{isPathShared}/{filePath}/
```

###### Method
```
PUT
```

###### Header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Body
```
{
    "name": "new file name",
    "metadata": base64 String
}
```

##### Response

##### Status code

```
200
```

### Delete File

#### Request

##### End point
```
/nfs/file/{isPathShared}/{filePath}
```

##### Method
```
DELETE
```

##### Headers
```
Authorization: Bearer {TOKEN}
```

#### Response

###### Headers
```
status: 200 Ok
```

### Move/Copy File

API to move or copy the file from source directory to a destination directory.
The source path and destination path must already exists or Bad Request(400) error will
be returned.

#### Request

##### Endpoint
`/nfs/movefile`

##### Method
`POST`

##### Header
```
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

##### Body

|Field|Description|
|-----|-----------|
|srcPath| Source path which has to be copied or moved. eg, `/a/b/c.txt`|
|isSrcPathShared| Optional value. Boolean value to indicate whether the source path is shared or private. Defaults to false|
|destPath| Destination directory path to which the file must be copied or moved. eg, `a/b`|
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

#### Response

##### Status code
```
200
```
