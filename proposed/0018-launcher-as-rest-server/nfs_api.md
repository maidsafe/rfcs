## NFS

### General Parameter Specification

Common parameters used,

- isVersioned is used to denote whether the Directory is to to be versioned or not.  
- isPrivate is used to indicate whether the Directory is private and encrypted.
  If not, then the Directory is exposed as public and unencrypted.
- dirPath corresponds to the root path. The last token in the path will be interpreted as the name of
  directory to be used for the operation. e.g. "/path/to/a/new_directory".  
- isPathShared denotes whether the path of the directory is shared from SAFE Drive or from the application
  root directory. Optional parameter. Default value will be `false`.
- metadata - refers to the user metadata that can be saved along with the Directory or File
- filePath corresponds to the path to the file. The last token in the path will be interpreted as the name of
  file to be used for the operation. e.g. "/path/to/a/directory/my_file.txt".  

### Unregistered Client Access
Get Directory and File APIs are exposed for Unregistered client access.

### Authorised Request
- The Authorisation token must be passed in the request header.
- Authorised requests should encrypt the entire url path, using the symmetric encryption key.
- The body of the http request should also be encrypted using the symmetric key.

For example,
```
GET http:\\api.safenet\{encrypted_path_along_with_the_query_params}
```

### Directory

#### Create Directory

Creates a directory in the network

#### Request

##### End point
```
/v1/nfs/directory
```

##### Method
```
POST
```

##### Request header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Request Body

```
{    
    "dirPath": String,
    "isPathShared": Boolean, // Optional. Default value - false    
    "isPrivate": Boolean, // Optional. Default value - true
    "isVersioned": Boolean, // Optional. Default value - false
    "metadata": base64 String // Optional field. Any additional metadata. to be passed as base64 string
}
```

#### Response on Success

##### Response headers
```
status: 202 Accepted
```

#### Get Directory

##### Request

dirPath must be url-encoded.

##### End point
```
/v1/nfs/directory/{dirPath}/{isPathShared}
```

##### Method
```
GET
```

##### Request header
```
Authorization: Bearer {TOKEN} // Optional
```

#### Response on Success

##### Response headers
```
content-type: application/json
status: 200 Ok
```

##### Response body
```
{
    "info" {
            "name": String,
            "creationTime": Integer, // milliseconds            
            "modificationTime": Integer, // milliseconds            
            "isPrivate": Boolean,
            "isVersioned": Boolean,
            "metadata": base64 String,
    },
    "subDirectories": [
        {
            "name": String,
            "creationTime": Integer, // milliseconds            
            "modificationTime": Integer, // milliseconds
            "isPrivate": Boolean,
            "isVersioned": Boolean,
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

#### Delete Directory

#### Request

##### End point
dirPath must be url encoded
```
/v1/nfs/directory/{dirPath}/{isPathShared}
```

##### Method
```
DELETE
```

##### Request header
```
Authorization: Bearer {TOKEN}
```

#### Response on Success

##### Response headers
```
status: 202 Accepted
```

#### Update Directory

Renaming a directory or updating its metadata can be performed. The request payload must have
at least one key value pair.

#### Request

##### End point
```
/v1/nfs/directory/{dirPath}/{isPathShared}
```

##### Method
```
PUT
```

##### Request header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Request Body
```
{
    "name": String,
    "metadata": base64 string
}
```

#### Response on Success

##### Response headers
```
status: 202 Accepted
```

### File

#### Create File

##### Request

###### End point
```
/v1/nfs/file
```

###### Method
```
POST
```

###### Request header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

###### Request Body

```
{
    "filePath": String,
    "isPathShared": Boolean, // Optional
    "metadata": base64 string // Optional
}
```

##### Response on Success

###### Response headers
```
status: 202 Accepted
```

#### Read File

##### Request
filePath must be url encoded

###### End point
```
/v1/nfs/file/{filePath}/{isPathShared}
```

##### Optional parameters
```
offset - Default value is 0. This indicates the starting position from where to read the file
length - Reads the entire content from the offset by default. If length is mentioned it will read only the
         content from offset to the size specified
```

###### Method
```
GET
```

###### Request header
```
Authorization: Bearer {TOKEN}
```

##### Response on Success

###### Response headers
```
Content-Type: application/json
Content-Length: 3000
file-name: file_name
file-created-time: 1452447851949 // time in milliseconds
file-modified-time: 1452447851949 // time in milliseconds
file-metadata: base64 string
status: 200 Ok
```

###### Response Body
```
File content as bytes. encrypted only for authorised requests
```

#### Update File Metadata

##### Request

###### End point
```
/v1/nfs/file/metadata/{filePath}/{isPathShared}
```

###### Method
```
PUT
```

###### Request header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Request Body
```
{
    "name": "new file name",
    "metadata": base64 String
}
```

##### Response on Success

###### Response headers
```
status: 202 Accepted
```

#### Update File Content

##### Request

###### End point
```
/v1/nfs/file/{filePath}/{isPathShared}
```

###### Optional parameter
```
offset - starting position from where the data has to modified
```

###### Method
```
PUT
```

###### Request header
```
Authorization: Bearer {TOKEN}
```

###### Request Body
```
Encrypted file content as base64 String
```

##### Response on Success

###### Response headers
```
status: 202 Accepted
```

#### Delete File

##### Request

###### End point
```
/v1/nfs/file/{filePath}/{isPathShared}
```

###### Method
```
DELETE
```

###### Request header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Response on Success

###### Response headers
```
status: 202 Accepted
```
