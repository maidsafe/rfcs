# Launcher service documentation
- Type: New Product
- Related components: safe_launcher_core (accompanying RFC)
- Start Date: 05-October-2015

# Summary

This is an accompanying RFC to the parent `Launcher-as-a-service` RFC and defines JSONs for Launcher-App RPC. This RFC must be read and is useful only in conjunction with the parent RFC.

# Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- `String` means UTF-8 encoded string unless specifically noted to be otherwise.
- Unless specifically mentioned, all JSON key-value pairs shall be considered mandatory.
- any mention of `[ uint8 ... ]` shall be interpreted as follows: binary data (array of unsigned 8 byte integers) encoded as `Base64` and packaged into a UTF-8 string. Correspondingly `[]` would be an empty such string.
- Configuration for Base64 encoding shall be as follows
  - Standard Character Set
  - Line Feed as new-line
  - Padding (in case the length is not divisible by 4) set to true (will use `=`)
  - No line length specified
- All mention of `Integer` correspond to 64 bit integers.

# Detailed design

- The final payload for the underlying stream (e.g. TCP) shall be as follows: an 8 byte, little-endian encoded, unsigned integer holding the size of the actual JSON/encrypted-JSON payload to follow. So if JSON/encrypted-JSON data is denoted as `{P}`, where `{P}` is a sequence of bytes, then the final payload on the wire should be `{S}{P}` where `{S}` is the size of `{P}` in bytes in the format mentioned. All associated responses (errors or otherwise) shall contain `SHA512` of `{P}` in the `id` field, described in detail later.
- `endpoint` shall follow `<id>/<version>/<module>/<request>` pattern. E.g. `safe-api/v1.29/nfs/create-dir`.
- Allowed `module`s are:
```
"handshake",
"nfs",
"dns"
```
- In all cases after this point, `<version>` for the `endpoint` field is a variable and is written as `v1.0` just as an example.
- Associated errors will always have the format
```javascript
{
    "id": [ uint8 ... ], // SHA512({P})
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
- Requests
```
"authenticate-app"
```

- ECDH-Key-Exchange, app to Launcher: This uses [Curve25519](https://en.wikipedia.org/wiki/Curve25519) (from libsodium) for symmetric key exchange
```javascript
{
    "endpoint": "safe-api/v1.0/handshake/authenticate-app",
    "data": {
        "launcher_string": String, // This shall be the one supplied by Launcher
        "asymm_nonce": [ uint8 ... ], // sodiumoxide::crypto::box_::Nonce,
        "asymm_pub_key": [ uint8 ... ]  // sodiumoxide::crypto::box_::PublicKey from
                                                // <App-Asymm-Keys>
    }
}
```
Associated response
```javascript
{
    "id": [],
    "data": {
        "encrypted_symm_key": [ uint8 ... ], // Symmetric-Key encrypted with peer's nonce and public-asymmetric-key and authenticated with Launcher's private-symmetric-key.
	"launcher_public_key": [ uint8 ... ], // Public-key of Launcher, the private counterpart of which was used to authenticate the above.
    }
}
```
- No communication hence forth shall be in plain-text. All JSON's must be encrypted using the symmetric-key and iv provided by Launcher.

## nfs
- Requests
```
"create-dir",
"delete-dir",
"get-dir",
"modify-dir",
"create-file",
"delete-file",
"get-file",
"modify-file"
```

- Create directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/create-dir",
    "data": {
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "dir_path": String, // Path root will be interpreted according
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

- Delete directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/delete-dir",
    "data": {
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "dir_path": String // Path root will be interpreted according
                           // the parameter above. The last token in
                           // the path will be interpreted as the name
                           // of directory to be deleted.
                           // e.g. "/path/to/an/existing_directory"
    }
}
```

- Get directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/get-dir",
    "data": {
        "timeout_ms": Integer, // Time out a GET request after specified number of miliseconds.
                               // Negative values mean that request will never time out.
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "dir_path": String // Path root will be interpreted according
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
    "id": [ uint8 ... ], // SHA512({P})
    "data": {
	"info" {
            "name": String,
            "creation_time_sec": Integer, // Number of sec after beginning of epoch.
            "creation_time_nsec": Integer, // Number of nano-sec offset from creation_time_sec.
            "modification_time_sec": Integer, // Number of sec after beginning of epoch.
            "modification_time_nsec": Integer, // Number of nano-sec offset from modification_time_sec.
            "is_private": Boolean,
            "is_versioned": Boolean,
            "user_metadata": [ uint8 ... ],
	}
        "sub_directories": [
            {
                "name": String,
                "creation_time_sec": Integer, // Number of sec after beginning of epoch.
                "creation_time_nsec": Integer, // Number of nano-sec offset from creation_time_sec.
                "modification_time_sec": Integer, // Number of sec after beginning of epoch.
                "modification_time_nsec": Integer, // Number of nano-sec offset from
                                                   // modification_time_sec.
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
                "creation_time_nsec": Integer, // Number of nano-sec offset from creation_time_sec.
                "modification_time_sec": Integer, // Number of sec after beginning of epoch.
                "modification_time_nsec": Integer, // Number of nano-sec offset from
                                                   // modification_time_sec.
                "user_metadata": [ uint8 ... ]
            },
            ...
        ]
    }
}
```

- Modify directory
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/modify-dir",
    "data": {
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
        "dir_path": String // Path root will be interpreted according
                           // the parameter above. The last token in
                           // the path will be interpreted as the name
                           // of directory to be read.
                           // e.g. "/path/to/an/existing_directory"
        "new_values": {
            // All fields are optional. The ones which are present will be updated with the new
            // value against them.
            "name": String,
            "user_metadata": [ uint8 ... ]
        }
    }
}
```

- Create File
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/create-file",
    "data": {
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "file_path": String, // Path root will be interpreted according
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
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "file_path": String // Path root will be interpreted according
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
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "file_path": String, // Path root will be interpreted according
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
    "id": [ uint8 ... ], // SHA512({P})
    "data": {
        "content": [ uint8 ... ],
        "metadata": { // This field will be absent if `include_metadata` was false in the request.
            "name": String,
            "size": Integer,
            "creation_time_sec": Integer, // Number of sec after beginning of epoch.
            "creation_time_nsec": Integer, // Number of nano-sec offset from creation_time_sec.
            "modification_time_sec": Integer, // Number of sec after beginning of epoch.
            "modification_time_nsec": Integer, // Number of nano-sec offset from
                                               // modification_time_sec.
            "user_metadata": [ uint8 ... ]
        }
    }
}
```

- Modify file
```javascript
{
    "endpoint": "safe-api/v1.0/nfs/modify-file",
    "data": {
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
        "file_path": String // Path root will be interpreted according
                            // the parameter above. The last token in
                            // the path will be interpreted as the name
                            // of file to be read.
                            // e.g. "/path/to/an/existing_file.ext"
        "new_values": {
            // All fields are optional. The ones which are present will be updated with the new
            // value against them.
            "name": String,
            "content": {                
                "offset": Integer, // Optional field. Offset in bytes to start writing from.
                                   // If the offset key is not present, then the entire file is overwritten.                                   
                "bytes": [ uint8 ... ] // Mandatory field. Contents of the file
            },
            "user_metadata": [ uint8 ... ]
        }
    }
}
```

## dns
- Requests
```
"register-dns",
"add-service"
```

- Register DNS
```javascript
{
    "endpoint": "safe-api/v1.0/dns/register-dns",
    "data": {
        "long_name": String, // e.g. "SomeNewDnsName"
        "service_name": String, // e.g. "www"
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
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
        "long_name": String, // e.g. "SomeExistingDnsName"
        "service_name": String, // e.g. "blog"
        "is_path_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise.
                                   // e.g. false
        "service_home_dir_path": String // Path root will be interpreted according
                                        // the parameter above. The last token in
                                        // the path will be interpreted as the name
                                        // of the home directory for the service.
                                        // e.g. "/path/to/an/existing_directory_blog"
    }
}
```
