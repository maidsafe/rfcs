- Feature Name: safe_dns API from safe_launcher
- Status: rejected
- Type: Enhancement
- Related components safe_launcher
- Start Date: 24-11-2015
- RFC PR: #76
- Issue number: Proposed - #78

# Summary

Expose needed APIs from safe_launcher to perform public name and service lookup

# Motivation

At present the launcher exposes API for Registering public names and adding service for a public name.
To intercept the scheme and serve the content from the browser add-on, the dns_api for
fetching the home directory of the service should be exposed. Along with this if the
other essential APIs like `get_all_registered_names`, `get_all_services`, `delete_service`, etc. are exposed, it
would allow variety of applications to be built on the network.
For example, SAFENetwork Service management application could be something which can help users to manage the
services exposed under their public name.

# Detailed design

The APIs are just an extension from the already existing [dns API](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/parser/dns/mod.rs) exposed from the safe_launcher.

### Get Service Home Directory

#### Implementation
The request would make use of the [get_home_service_directory](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/mod.rs#L150)
function from safe_dns crate and get the DirectoryKey. Using the directory key the actual directory content is retrieved using the safe_nfs crate.
The fetched directory is returned as part of the response.

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/get-service-directory',
  data: {
    service_name: String,
    public_name: String
  }
}
```

#### Response

##### On Success
```javascript
{
  'id': String, // Base64 String - [ uint8 ... ] SHA512(Request)
  'data': {
    'info': {
      'name': String,
      'creation_time_sec': Integer, // Number of sec after beginning of epoch.
      'creation_time_nsec': Integer, // Number of nano-sec offset from creation_time_sec.
      'modification_time_sec': Integer, // Number of sec after beginning of epoch.
      'modification_time_nsec': Integer, // Number of nano-sec offset from modification_time_sec.
      'is_private': Boolean,
      'is_versioned': Boolean,
      'user_metadata': String, // base64 string - [ uint8 ... ]
    }
    'sub_directories': [
      {
        'name': String,
        'creation_time_sec': Integer, // Number of sec after beginning of epoch.
        'creation_time_nsec': Integer, // Number of nano-sec offset from creation_time_sec.
        'modification_time_sec': Integer, // Number of sec after beginning of epoch.
        'modification_time_nsec': Integer, // Number of nano-sec offset from
                                           // modification_time_sec.
        'is_private': Boolean,
        'is_versioned': Boolean,
        'user_metadata': String, // base64 string - [ uint8 ... ]
      },
      ...
    ],
    'files': [
      {
        'name': String,
        'size': Integer,
        'creation_time_sec': Integer, // Number of sec after beginning of epoch.
        'creation_time_nsec': Integer, // Number of nano-sec offset from creation_time_sec.
        'modification_time_sec': Integer, // Number of sec after beginning of epoch.
        'modification_time_nsec': Integer, // Number of nano-sec offset from
                                           // modification_time_sec.
        'user_metadata': String, // base64 string - [ uint8 ... ]
      },
      ...
    ]
  }
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

### Get Content from URL Path

This API would be able to fetch the contents of the file corresponding to the path specified.

#### Implementation

Using the `safe_dns` crate the the Service home DirectoryKey can be fetched, and using the
DirectoryKey the actual Directory can be fetched from the network. Once the directory is
fetched the file corresponding to the path can be traversed. Error is returned, if the file is not found at the path specified.
Else the file is retrieved and sent back as response.

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/get-file-from-service-directory',
  data: {
    service_name: String, // eg, blog
    public_name: String, // eg, maidsafe
    path: String // /index.html or /imgs/logo.png - the complete path
    range: { // This is an optional field. This would help in retrieval of data in smaller chunks.
      offset: Integer,
      size: Integer
    }
  }
}
```

#### Response

##### On Success
```javascript
{
  id: String
  data: {
    metadata: {
      'name': String,
      'size': Integer,
      'creation_time_sec': Integer, // Number of sec after beginning of epoch.
      'creation_time_nsec': Integer, // Number of nano-sec offset from creation_time_sec.
      'modification_time_sec': Integer, // Number of sec after beginning of epoch.
      'modification_time_nsec': Integer, // Number of nano-sec offset from
                                         // modification_time_sec.
      'user_metadata': String, // base64 string - [ uint8 ... ]
    },
    content: String, //base64 String
  }
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

### List Public Names

API would list all the public name associated to the User Account.

#### Implementation
This is simple a direct API call to the [get_all_registered_names](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/mod.rs#L119) function

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/list-public-names',
  data: {}
}
```

#### Response

##### on Success
```javascript
{
  id: String,
  data: [ // List of String
    String
  ]
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

### List all registered Services for a Public Name

API would list all the registered services for a public name associated to the User Account

#### Implementation
This is a direct API call to the [get_all_services](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/mod.rs#L132) function

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/list-services',
  data: {
    public_name: String, // Simple String value. eg: maidsafe
  }
}
```

#### Response

##### on Success
```javascript
{
  id: String,
  data: [ // List of String
    String
  ]
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

### Delete a Public Name

API would delete a public name associated to the User Account

#### Implementation
This is a direct API call to the [delete_dns](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/mod.rs#L95) function

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/delete-public-name',
  data: {
    public_name: String, // Simple String value. eg: maidsafe
  }
}
```

#### Response

##### on Success
```javascript
{
  id: String,
  error: null
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

### Delete a Service for a Public Name
API would delete a service for a public name associated to the User Account.

#### Implementation
This is a direct API call to the [remove_service](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/mod.rs#L180) function

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/dns/delete-service',
  data: {
    public_name: String, // Simple String value. eg: maidsafe
    service: String, // Simple String value. eg: blog
  }
}
```

#### Response

##### on Success
```javascript
{
  id: String,
  error: null
}
```

##### On Error

```javascript
{
  id: String
  error: {
    code: -200,
    description: 'Some error'
  }
}
```

# Drawbacks

None

# Alternatives

None

# Unresolved questions

None
