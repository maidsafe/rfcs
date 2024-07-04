## DNS

### General Parameter Specification

Common parameters used,
- isPathShared denotes whether the path of the directory is shared from SAFE Drive or from the application
  root directory. Optional parameter. Default value will be `false`.

### Unregistered Client Access
Get Service Directory and get File APIs are exposed for Unregistered client access.

### Authorised Request
- The Authorisation token must be passed in the request header.
- Authorised requests should encrypt the entire url path, using the symmetric encryption key.
- The body of the http request should also be encrypted using the symmetric key.

For example,
```
GET http:\\api.safenet\{encrypted_path_along_with_the_query_params}
```


### Register DNS

#### Request

##### Endpoint
```
/v1/dns/register
```

##### Method
```
POST
```

##### Request Header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Request Body
```
{
    "longName": String, // e.g. "SomeNewDnsName"
    "serviceName": String, // e.g. "www"
    "isPathShared": Boolean, // Optional
    "serviceHomeDirPath": String // Path root will be interpreted according
                                    // the parameter above. The last token in
                                    // the path will be interpreted as the name
                                    // of the home directory for the service.
                                    // e.g. "/path/to/an/existing_directory_www"
}
```

#### Response

##### Response Header
```
status: 202 Accepted
```

### Add Service

#### Request

##### Endpoint
```
/v1/dns/service
```

##### Method
```
POST
```

##### Request Header
```
content-type: application/json
Authorization: Bearer {TOKEN}
```

##### Request Body
```
{
    "longName": String, // e.g. "SomeNewDnsName"
    "serviceName": String, // e.g. "www"
    "isPathShared": Boolean, // Optional
    "serviceHomeDirPath": String // Path root will be interpreted according
                                 // the parameter above. The last token in
                                 // the path will be interpreted as the name
                                 // of the home directory for the service.
                                 // e.g. "/path/to/an/existing_directory_www"
}
```

#### Response

##### Response Header
```
status: 202 Accepted
```

### Get Service Directory (Unregistered Client Access)

This API can be called without authorisation token for getting public unencrypted data

#### Request

##### Endpoint
```
/v1/dns/{serviceName}/{longName}
```

##### Method
```
GET
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
status: 200 Ok
```

##### Response body
```
{
    "info" {
            "name": String,
            "creationTime": Integer, // milli seconds            
            "modificationTime": Integer, // milli seconds            
            "isPrivate": Boolean,
            "isVersioned": Boolean,
            "metadata": base64 String,
    },
    "subDirectories": [
        {
            "name": String,
            "creationTime": Integer, // milli seconds            
            "modificationTime": Integer, // milli seconds
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
            "creationTime": Integer, // milli seconds            
            "modificationTime": Integer, // milli seconds
            "metadata": base64 String
        },
        ...
    ]
}
```

### Get File (Unregistered Client Access)

#### Request
filePath must be url encoded

##### End point
```
/v1/dns/{serviceName}/{longName}/{filePath}
```

##### Optional parameters
```
offset - Default value is 0. This indicates the starting position from where to read the file
length - Reads the entire content from the offset by default. If length is mentioned it will read only the
         content from offset to the size specified
```

##### Method
```
GET
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
Content-Length: Size
Content-Type: {mime type of the file}
status: 200 Ok
```

##### Response body
```
Will serve the raw data of the file (Encrypted only for authorised request)
```

### List Long Names

List the public names for the user

#### Request

##### End point
```
/v1/dns/longName
```

##### Method
```
GET
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
Content-Type: application/json
status: 200 Ok
```

##### Response body
```
[ // List of String (Public Names)
    String
]
```


### List Services

List the services available for a long name

#### Request

##### End point
```
/v1/dns/service/{longName}
```

##### Method
```
GET
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
Content-Type: application/json
status: 200 Ok
```

##### Response body
```
[ // List of String (Services)
    String
]
```


### Delete Long Name

#### Request

##### End point
```
/v1/dns/longName/{longName}
```

##### Method
```
DELETE
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
status: 202 Accepted
```


### Delete Service for a Long Name

#### Request

##### End point
```
/v1/dns/service/{serviceName}/{longName}
```

##### Method
```
DELETE
```

##### Request Header
```
Authorization: Bearer {TOKEN}
```

#### Response

##### Response Header
```
status: 202 Accepted
```
