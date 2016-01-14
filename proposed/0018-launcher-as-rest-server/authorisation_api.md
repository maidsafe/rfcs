- Feature Name: Launcher as a local server
- Type New Product
- Related components safe_launcher
- Start Date: 11-01-2016

# Summary
This is an accompanying RFC to the parent launcher-as-local-rest-server RFC and defines APIs for RESTFull interface.
This RFC must be read and is useful only in conjunction with the parent RFC.

# Conventions

- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and
"OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- String means UTF-8 encoded string unless specifically noted to be otherwise.
- Unless specifically mentioned, all JSON key-value pairs shall be considered mandatory.
any mention of [ uint8 ... ] shall be interpreted as follows: binary data (array of unsigned 8 byte integers) encoded as
Base64 and packaged into a UTF-8 string. Correspondingly [] would be an empty such string.
- Configuration for Base64 encoding shall be as follows
    - Standard Character Set
    - Line Feed as new-line
    - Padding (in case the length is not divisible by 4) set to true (will use =)
    - No line length specified
- SERVER_PORT refers to the port to which the server binds

# Detailed API Design

The APIs can be directly invoked from localhost:SERVER_PORT. Web application are expected to direct the API requests to
`api.safenet`

**For authorised requests, the query string and also the HTTP body payload must be encrypted using the Symmetric key
obtained after authorisation**

## Authorisation API

### Application Authorisation

#### Request

##### End point
```
/v1/auth
```

##### Method
```
POST
```

##### Request header
```
content-type: application/json
```

##### Request Body

```javascript
{
    "app": {
        "name": "Application Name", //string - application name
        "vendor": "Vendor name", //string - application vendor name
        "version": "0.0.1", // string - application version number
        "key": "unique_package_key" // String - unique key for application        
    },
    "permissions": [ // optional field
        'SAFE_DRIVE_ACCESS'
    ],
    "publicKey": "base64 string", // [uint8] as base64 String
    "nonce": "base64 string", // [uint8] as base64 String
}
```

#### Response on Success

##### Response headers
```
content-type: application/json
status: 200 OK
```

##### Response Body
```javascript
{
    "token": "header.payload.signed", //JWT token
    "encryptionKey": "base64", // Encrypted Symmetric key
    "publicKey": "base64", // public key used for encryption of the Symmetric Key,
    "permissions": [ // list of permissions approved
        "SAFE_DRIVE_ACCESS"
    ]
}
```

### Revoke Token

Revoke token is invoked to revoke the authorised token.  

#### Request

##### End point
```
/v1/auth/revoke?token={JWT_TOKEN}
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
status: 200 OK
```
