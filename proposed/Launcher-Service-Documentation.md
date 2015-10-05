- Feature Name: Launcher service documentation
- Type: New Product
- Related components: safe_launcher_core (accompanying RFC)
- Start Date: 05-October-2015

# Summary

This is an accompanying RFC to the parent `Launcher-as-a-service` RFC and defines JSONs for Launcher-App RPC. This RFC must be read and is useful only in conjunction with the parent RFC.

# Conventions
The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

# Detailed design

## Globals
- RSA-Key-Exchange, app to Launcher
```
{
    "rsa_key_exchange_request": {
        "launcher_string": UTF-8 String,        // This shall be the one supplied by Launcher
        "nonce": [ uint8 ... ],                 // sodiumoxide::crypto::box_::Nonce,
        "public_encryption_key": [ uint8 ... ]  // sodiumoxide::crypto::box_::PublicKey from
                                                // <App-Asymm-Keys>
    }
}
```
- RSA-Key-Exchange, Launcher to app
```
{
    "rsa_key_exchange_response": {
        "cipher_text": [ uint8 ... ] // encrypted symmetric keys
    }
}
```
- Version Negotiation
```
{
    "version": x.y // where x.y could be 2.10 etc
}
```

## Module-Specific
- Modules
```
"NFS"
"DNS"
```

### NFS
- Actions
```
"create-dir"
"delete-dir"
"create-file"
"delete-file"
```

- Create Directory
```
{
    "module": "NFS"
    "action": "create-dir"
    "parameters": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of directory to be created.
                        // e.g. "/path/to/a/new_directory"
        "is_private": Boolean // true if the created directory must be encrypted, false if
                              // publicly viewable.
                              // e.g. true
        "is_versioned": Boolean, // e.g. false
        "user_metadata": [ uint8 ... ] // Any additional metadata.
                                       // e.g. [ 20, 30, 255, 254, 0, 119 ]
    }
}
```

- Delete Directory
```
{
    "module": "NFS"
    "action": "delete-dir"
    "parameters": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of directory to be deleted.
                        // e.g. "/path/to/an/existing_directory"
    }
}
```

- Create File
```
{
    "module": "NFS"
    "action": "create-file"
    "parameters": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
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
```
{
    "module": "NFS"
    "action": "delete-file"
    "parameters": {
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
                              // e.g. false
        "path": String, // Path root will be interpreted according
                        // the parameter above. The last token in
                        // the path will be interpreted as the name
                        // of file to be deleted.
                        // e.g. "/path/to/an/existing_file.ext"
    }
}
```

### DNS
- Actions
```
"register-dns"
"add-service"
```

- Register DNS
```
{
    "module": "DNS"
    "action": "register-dns"
    "parameters": {
        "long_name": String, // e.g. "new-name.com"
        "service_name": String, // e.g. "www"
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
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
```
{
    "module": "DNS"
    "action": "add-service"
    "parameters": {
        "long_name": String, // e.g. "existing-name.com"
        "service_name": String, // e.g. "blog"
        "is_shared": Boolean, // true if root is to be considered `SAFEDrive`, false otherwise
                              // e.g. false
        "service_home_dir_path": String // Path root will be interpreted according
                                        // the parameter above. The last token in
                                        // the path will be interpreted as the name
                                        // of the home directory for the service.
                                        // e.g. "/path/to/an/existing_directory_blog"
    }
}
```
