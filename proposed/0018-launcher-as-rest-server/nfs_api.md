## NFS

### General Parameter Specification

Common parameters used,

- isVersioned is used to denote whether the Directory is to to be versioned or not.
- isPrivate is used to indicate whether the Directory is private and encrypted.
  If not, then the Directory is exposed as public and unencrypted.
- dirPath corresponds to the root path. The last token in the path will be interpreted as the name of
          directory to be used for the operation. e.g. "/path/to/a/new_directory"
- isPathShared denotes whether the path of the directory is shared from SAFE Drive or from the application
  root directory.
- metadata - refers to the user metadata that can be saved along with the Directory or File

Get Directory and File APIs are exposed for Unregistered client access.
For all other requests the query string and the response body must be encrypted with the symmetric key and nonce
obtained after authorisation.

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
    "isPathShared": Boolean,
    "dirPath": String,
    "isPrivate": Boolean,
    "isVersioned": Boolean,
    "metadata": base64 String // Optional field. Any additional metadata. to be passed as base64 string
}
```

#### Response on Success

##### Response headers
```
status: 202 Accepted
```

#### Get Directory

##### End point
```
/v1/nfs/directory?isPathShared=boolean&dirPath=path_of_directory
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
```
/v1/nfs/directory?isPathShared=true&dirPath=dir_path_to_delete
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
/v1/nfs/directory?isPathShared=boolean&dirPath=path_of_directory
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
    "isPathShared": Boolean,
    "filePath": String, // e.g. "/path/to/a/new_file.ext"
    "metadata": base64 string
}
```

##### Response on Success

###### Response headers
```
status: 202 Accepted
```

#### Read File

##### Request

###### End point
```
/v1/nfs/file?isPathShared=boolean&filePath=path_to_file
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
content-type: application/json
content-size: 3000
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
/v1/nfs/file/metadata?isPathShared=boolean&filePath=path_of_file
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
/v1/nfs/file?isPathShared=boolean&filePath=path_of_file
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
/v1/nfs/file?isPathShared=boolean&filePath=path_of_file
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
