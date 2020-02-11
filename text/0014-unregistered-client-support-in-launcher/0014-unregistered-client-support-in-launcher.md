# Unregistered Client Support in Launcher

- Status: rejected
- Type: New Feature
- Related components: [safe_launcher](https://github.com/maidsafe/safe_launcher)
- Start Date: 03-November-2015
- RFC PR: #66
- Issue number: Proposed - #67

## Summary

Launcher will need to cater to the requests made by unregistered clients to access the SAFE Network. This RFC details why this is required and how this will be done.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- `{P}` refers to the payload. More details in the (RFC here](https://github.com/maidsafe/rfcs/blob/master/text/0010-launcher-as-a-service/0010-launcher-as-a-service.md).

## Motivation

### Why?

There are plenty of use cases for unregistered clients (those that don't have a valid SAFE Account with `MaidManagers`) to access the SAFE Network. One such example is the browser category. The browsers do not need to create an account to access the SAFE Network nor do they require a registered client engine (one that performs operations on a valid account) because all they care about is the fetching and display of data. This is in line with our philosophy that anyone can fetch data from the SAFE Network - it will be of no use if it is encrypted and client fetching it does not have the decryption keys, but that is another matter. Without Launcher, each such application will have to interface with low level libraries like [safe_core](https://github.com/maidsafe/safe_core) and/or [safe_nfs](https://github.com/maidsafe/safe_nfs). Further every instance of an engine from [safe_core](https://github.com/maidsafe/safe_core) will create a new routing object. All this is unnecessary overhead. Launcher will funnel requests from all unregistered applications through a single instance of an unregistered client engine obtained from [safe_core](https://github.com/maidsafe/safe_core).

## Detailed design

There shall be a discovery mechanism in place to detect a Launcher binary running on a machine. For this Launcher shall broadcast a special packet containing a UTF-8 encoded string which will be `--launcher:tcp:<ip>:<port>`. The broadcast shall be done every **5 seconds** on port **59999**. Any application can then connect to Launcher on the announced endpoint. Once the connection is made, it will give Launcher the a special endpoint string that is metioned below. Launcher on receiving this string shall not do further Handshake. It will listen to JSON requests and carry out the tasks as usual, returning either data or error via JSON. The communication will be in plain text instead of cipher text, i.e. these JSONs will be unencrypted. On first such encounter of such a request, Launcher shall request an unregistered client engine from [safe_core](https://github.com/maidsafe/safe_core) and use that for the present as well as for all such future connections. The JSONs are described below.

Handshake for anonymous access:
```javascript
{
    "endpoint": "safe-api/v1.0/handshake/anonymous-access",
}
```

### dns
- Addtional requests to those mentioned [here for dns](https://github.com/maidsafe/rfcs/blob/master/text/0010-launcher-as-a-service/0010-launcher-as-a-service.md)
```
"get-services"
"get-service-file-size"
"get-service-file"
```

- Get all services for a DNS record
```javascript
{
    "endpoint": "safe-api/v1.0/dns/get-services",
    "data": {
        "long_name": String, // DNS record name. E.g. "maidsafe"
        "service_name": String, // E.g. "www" , "blog" etc.
    }
}
```
Associated response
```javascript
{
    "id": [ uint8 ... ], // SHA512({P})
    "data": {
        "services": [ Strings ... ]
    }
}
```

- Get file size for a file in DNS service's home directory tree
```javascript
{
    "endpoint": "safe-api/v1.0/dns/get-service-file-size",
    "data": {
        "long_name": String, // DNS record name. E.g. "maidsafe"
        "service_name": String, // E.g. "www" , "blog" etc.
        "file_path": String, // Path root will be interpreted as the mentioned
                             // service's Home directory. The last token in
                             // the path will be interpreted as the name
                             // of file to be read.
                             // e.g. "/path/to/an/existing_file.ext"
        "include_metadata": Boolean // false if only the size is to be given,
                                    // true otherwise. E.g. false
    }
}
```
Associated response
```javascript
{
    "id": [ uint8 ... ], // SHA512({P})
    "data": {
        "size": Integer,
        "metadata": { // This field will be absent if `include_metadata` was false in the request.
            "name": String,
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

- Get file contents for a file in DNS service's home directory tree
```javascript
{
    "endpoint": "safe-api/v1.0/dns/get-service-file",
    "data": {
        "long_name": String, // DNS record name. E.g. "maidsafe"
        "service_name": String, // E.g. "www" , "blog" etc.
        "file_path": String, // Path root will be interpreted as the mentioned
                             // service's Home directory. The last token in
                             // the path will be interpreted as the name
                             // of file to be read.
                             // e.g. "/path/to/an/existing_file.ext"
        "offset": Integer, // Offset in bytes to start reading from.
        "length": Integer, // Number of bytes to read starting from the given offset above.
                           // If negative, then complete file will be read starting from the
                           // offset.
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

## Alternative

1. Another way would be to add applications like browsers like any other app to Launcher and start them via Launcher. During adding of an app, Launcher would additionally prompt the user to specify if this app should be given the previlege to access the Network on his/her behalf or just access the Network anonymously (which will ofcourse limit the permitted operations to only reads). This would have an advantage of not complicating the design by adding UDP discovery mechanism and while also providing a uniform and a consistent interface to the user.

2. An alternative to the discovery mechanism where Launcher is multicasting/broadcasting to the apps, we can have the reverse. Launcher would bind to port **59999** and apps would send UDP packets to it asking for the TCP address. This can have an advantage because we are unlikely to thousands of clients connecting to Launcher. So once an app gets a UDP reply from Launcher it will stop sending requests to port **59999**. Statistically we would soon have no UDP messages as all apps would have connected to Launcher. So there would not be unnecessary UDP packets being continually sent by Launcher. Another advantage is that only Launcher is going to be bound to the listening UDP port so there much lesser chance of port contention.

3. Third alternative is to just assign a fix TCP endpoint to Launcher process. If Launcher is unable to attach an acceptor to this endpoint it will not start. So we have no discovery process. All apps know where to find Launcher.

## Implementation hints

- [authenticate_app.rs](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/ipc_session/authenticate_app.rs) will need to be changed to handle mulitple forms of handshake. If the endpoint is anonymous-access, it will not go through the process of handshake any further and just inform `IpcSession` that the handshake is over.
- [AppAuthenticationEvent](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/ipc_session/events.rs) will have to be changed to:
```rust
pub type AppAuthenticationEvent = Result<Option<event_data::AuthData>, ::errors::LauncherError>;
```
so that when `Result` evaluates to `Ok(None)`, it is to be understood that an unregistered access to the Network is desired.
- `IpcSession` will have to work in conjunction with `IpcServer` authenticate this session. This will require modification to [IpcServer](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/events.rs) and [IpcSession](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/ipc_session/events.rs) events. Also IpcServer should allow observation of such sessions (construction and tearing down).
- [SecureCommunication](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/ipc_session/secure_communication.rs) will have to know if it has to perform encryption and decryption or not. For this, a simple modification of definition to:
```rust
pub struct SecureCommunication {
    observer         : ::launcher::ipc_server::ipc_session::EventSenderToSession<::launcher
                                                                                 ::ipc_server
                                                                                 ::ipc_session
                                                                                 ::events::SecureCommunicationEvent>,
    symm_key         : Option<::sodiumoxide::crypto::secretbox::Key>,
    symm_nonce       : Option<::sodiumoxide::crypto::secretbox::Nonce>,
    ipc_stream       : ::launcher::ipc_server::ipc_session::stream::IpcStream,
    parser_parameters: ::launcher::parser::ParameterPacket,
}
```
and branching on if `Option` is `None` or otherwise should do it.
- UDP broadcast will be done by `IpcServer` once it has obtained the successfully spawned [acceptor](https://github.com/maidsafe/safe_launcher/blob/master/src/launcher/ipc_server/mod.rs#L321).
