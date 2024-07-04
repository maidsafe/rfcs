# NFS API

This is a supporting document for the parent [RFC](./0036-launcher-api-v0.5.md).
This details the NFS API changes and also the new MOVE/COPY APIs that are getting added.

The `isPathShared` boolean variable is changed to an enum representation (app / drive).
This varaible can be called as `rootPath` with values `app / drive`. This will
make the variable name self explanatory.


### SAFE_Drive and Application Directory

`SAFE_Drive` directory is created by default for every account. Applications can not access
SAFE_Drive directory unless the user grants the permission at the time of authorisation.

SAFE_Drive is meant for private storage for the account. Files/Folders can be accessed by applications
from SAFE_Drive only if the `SAFE_DRIVE_ACCESS` permission is granted.

Likewise when an application is authorised for the first time, a root directory for the
application is created and mapped to the account. The applications can store and retrieve data only
from the app's root folder. In case if the app needs to access SAFE_Drive, the app must authorise
with the `SAFE_DRIVE_ACCESS` permission and the user must grant the permission.

`SAFE_Drive` is a special folder, which can be accessed by applications for sharing
data.

For example, a camera app can store images on the SAFE_Drive. While another image editor
application, can read the images from the SAFE_Drive.

## Directory

### Create Directory

#### Request

##### Endpoint
|Field|Description|
|-----|-----------|
|rootPath| Enum Value (app/drive).|
|dirPath| Full directory path as String. Example - /home, /home/photos|

```
/nfs/directory/{rootPath}/{dirPath}
```

Example endpoint URL: `/nfs/directory/drive/home/music`


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
|metadata| Optional String. Metadata as base64 String|
|isPrivate| Optional Boolean value to indicate whether the folder should be private or readable by all. Defaults to true|

```
{
  metadata: String,
  isPrivate: Boolean
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
|rootPath| Enum Value (app/drive).|
|dirPath| Full directory path as String. Example - /home, /home/photos|

```
/nfs/directory/{rootPath}/{dirPath}
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
/nfs/directory/{rootPath}/{dirPath}
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
|srcRootPath| Enum value - app/drive. Defaults to app|
|destPath| Destination directory path to which the source directory must be copied or moved|
|destRootPath| Enum value - app/drive. Defaults to app|
|action| ENUM value - MOVE or COPY. Defaults to MOVE|

```
{
  "srcPath": String,
  "srcRootPath": Boolean,
  "destPath": String,
  "destRootPath": Boolean,
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
/nfs/directory/{rootPath}/{dirPath}
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
/nfs/file/{rootPath}/{filePath}
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
```
/nfs/file/:rootPath/:filePath
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
```
/nfs/file/:rootPath/:filePath
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
/nfs/file/metadata/{rootPath}/{filePath}/
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
/nfs/file/{rootPath}/{filePath}
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
|srcPath| Source path which has to be copied or moved. eg, /a/b/c.txt|
|srcRootPath| Enum value - app/drive. Defaults to app|
|destPath| Destination directory path to which the file must be copied or moved. eg, a/b|
|destRootPath| Enum value - app/drive. Defaults to app|
|action| ENUM value - MOVE or COPY. Defaults to MOVE|

```
{
  "srcPath": String,
  "srcRootPath": Boolean,
  "destPath": String,
  "destRootPath": Boolean,
  "action": ENUM (MOVE or COPY)
}
```
#### Response

##### Status code
```
200
```
