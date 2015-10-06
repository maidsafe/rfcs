- Feature Name: Launcher service documentation
- Type: New Product
- Related components: safe_launcher_core (accompanying RFC)
- Start Date: 05-October-2015

# Summary

This is an accompanying RFC to the parent `Launcher-as-a-service` RFC and defines JSONs for Launcher-App RPC. This RFC must be read and is useful only in conjunction with the parent RFC.

# Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- `String` means UTF-8 encoded string unless specifically noted to be otherwise.

# Detailed design

- `endpoint` shall follow `<id>/<version>/<disambiguator>/<request>` pattern. E.g. `safe-api/v1.29/nfs/create-dir`.
- Allowed `disambiguator`s are:
```
"handshake"
"nfs"
"dns"
```
- In all cases after this point, `<version>` for the `endpoint` field is a variable and is written as `v1.0` just as an example.
- Associated errors will always have the format
```javascript
{
    "id": [ uint8 ... ], // SHA512(JSON-request-string)
    "error": {
        "code": Integer, // This shall be derived from Into traits for all errors in Client modules
        "description": String
    }
}
```
- Unassociated errors will always have the format
```javascript
{
    "id": [],
    "error": {
        "code": Integer, // This shall be derived from Into traits for all errors in Client modules
        "description": String
    }
}
```

## handshake
- RSA-Key-Exchange, app to Launcher
```javascript
{
    "endpoint": "safe-api/v1.0/handshake/rsa-key-exchange",
    "data": {
        "launcher_string": String, // This shall be the one supplied by Launcher
        "nonce": [ uint8 ... ], // sodiumoxide::crypto::box_::Nonce,
        "public_encryption_key": [ uint8 ... ]  // sodiumoxide::crypto::box_::PublicKey from
                                                // <App-Asymm-Keys>
    }
}
```
Associated response
```javascript
{
    "id": [ uint8 ... ], // SHA512(JSON-request-string)
    "data": [ uint8 ... ] // encrypted symmetric keys
}
```

## nfs
- Requests
```
"create-dir"
"delete-dir"
"get-dir"
"create-file"
"delete-file"
"get-file"
```

- Create directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/create-dir",
    "data": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of directory to be created.
                        // e.g. "/path/to/a/new_directory"
        "is_private": Boolean, // true if the created directory must be encrypted, false if
                               // publicly viewable.
                               // e.g. true
        "is_versioned": Boolean, // e.g. false
        "user_metadata": [ uint8 ... ] // Any additional metadata.
                                       // e.g. [ 20, 30, 255, 254, 0, 119 ]
    }
}
```

- Delete Directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/delete-dir",
    "data": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String // Path root will be interpreted according
                       // the parameter above. The last token in
                       // the path will be interpreted as the name
                       // of directory to be deleted.
                       // e.g. "/path/to/an/existing_directory"
    }
}
```

- Get Directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/get-dir",
    "data": {
        "timeout_ms": Integer, // Time out a GET request after specified number of miliseconds.
                               // Negative values mean that request will never time out.
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String // Path root will be interpreted according
                       // the parameter above. The last token in
                       // the path will be interpreted as the name
                       // of directory to be read.
                       // e.g. "/path/to/an/existing_directory"
    }
}
```
Associated response
```javascript
{
    "id": [ uint8 ... ], // SHA512(JSON-request-string)
    "data": {
        "name": String,
        "creation_time_sec": Integer, // Number of sec after beginning of epoch.
        "creation_time_nsec": Integer, // Number of nano-sec, offset from creation_time_sec.
        "is_private": Boolean,
        "is_versioned": Boolean,
        "user_metadata": [ uint8 ... ],
        "sub-directories": [
            {
                "name": String,
                "creation_time_sec": Integer, // Number of sec after beginning of epoch.
                "creation_time_nsec": Integer, // Number of nano-sec, offset from creation_time_sec.
                "is_private": Boolean,
                "is_versioned": Boolean,
                "user_metadata": [ uint8 ... ]
            },
            ...
        ],
        "files": [
            {
                "name": String,
                "size": Integer,
                "creation_time_sec": Integer, // Number of sec after beginning of epoch.
                "creation_time_nsec": Integer, // Number of nano-sec, offset from creation_time_sec.
                "user_metadata": [ uint8 ... ]
            },
            ...
        ]
    }
}
```

- Create File
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/create-file",
    "data": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of file to be created.
                        // e.g. "/path/to/a/new_file.ext"
        "user_metadata": [ uint8 ... ] // Any additional metadata.
                                       // e.g. [ 20, 30, 255, 254, 0, 119 ]
    }
}
```

- Delete File
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/delete-file",
    "data": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String // Path root will be interpreted according
                       // the parameter above. The last token in
                       // the path will be interpreted as the name
                       // of file to be deleted.
                       // e.g. "/path/to/an/existing_file.ext"
    }
}
```

- Get File
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/get-file",
    "data": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of file to be read.
                        // e.g. "/path/to/an/existing_file.ext"
        "offset": Integer, // Offset in bytes to start reading from. Will be an error if out
                           // of bounds.
        "length": Integer, // Number of bytes to read starting from the given offset above. If
                           // offset + length >= file-size then complete file will be read starting
                           // from the offset. If negative, then complete file will be read
                           // starting from the offset.
        "include_metadata": Boolean // false if only the raw content is to be given,
                                    // true otherwise. E.g. false
    }
}
```
Associated response
```javascript
{
    "id": [ uint8 ... ], // SHA512(JSON-request-string)
    "data": {
        "content": [ uint8 ... ],
        "metadata": { // This field will be absent if `include_metadata` was false in the request.
            "name": String,
            "size": Integer,
            "creation_time_sec": Integer, // Number of sec after beginning of epoch.
            "creation_time_nsec": Integer, // Number of nano-sec, offset from creation_time_sec.
            "user_metadata": [ uint8 ... ]
        }
    }
}
```
## dns
- Requests
```
"register-dns"
"add-service"
```

- Register DNS
```javascript
{
    "endpoint": "safe-api/v1.0/dns/register-dns",
    "data": {
        "long_name": String, // e.g. "new-name.com"
        "service_name": String, // e.g. "www"
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "service_home_dir_path": String // Path root will be interpreted according
                                        // the parameter above. The last token in
                                        // the path will be interpreted as the name
                                        // of the home directory for the service.
                                        // e.g. "/path/to/an/existing_directory_www"
    }
}
```

- Add service
```javascript
{
    "endpoint": "safe-api/v1.0/dns/add-service",
    "parameters": {
        "long_name": String, // e.g. "existing-name.com"
        "service_name": String, // e.g. "blog"
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                              // e.g. false
        "service_home_dir_path": String // Path root will be interpreted according
                                        // the parameter above. The last token in
                                        // the path will be interpreted as the name
                                        // of the home directory for the service.
                                        // e.g. "/path/to/an/existing_directory_blog"
    }
}
```
