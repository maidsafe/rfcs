# Authentication Protocol


## Summary

This appendix of the New App Authentication flow describes the protocol between the Authenticator and third party apps to gain and extend access.

## Basics

In order to provide the same user flow and behaviour on mobile and desktop platforms, this protocol is using url-open commands to communicate between the apps. The Authenticator, when starting up, must register itself as the default handler for the `safeauth`-scheme on the system it was started on. Similarly, every app that wants to communicate with Authenticator must register itself under `safeauth-${base64(appId)}` - we will refer to this as the appURI from here on.

All keys handed over with the protocol, let that be the app authentication keys, dataIds to containers or encryption keys of any kind, are sensitive information that will persist unless explicitly changed by the User. It is the receiving apps responsibility to savely store and retrieve them. If the app isn't sure it can do this, it should ask for permissions on every start and only keep those in memory. When the authenticator is asked to grant the exact same permissions is has granted that app before, it should respond with the necessary information without any further user interaction.


**TODO**: base64 contains '=' as fillers character, and we are using `-` for prefixing: Are those allowed in url-schemes on all platforms? If not we might have to remove them before creating our return URLS.


## Format

The protocol is based on strings of valid URIs with a colon (`:`) delimiters, encoding serialised Rust Structs data within base64 packets as follows:

### Authenticator

```
safeauth:action:appId[:payload][?key=value&key2=value2]
```

where `action` is a particular string identifying the action (like `request-access`), appID is the `base64` encoded version of the given appId and `payload` is a `base64`-encoded serialised Rust Struct. Lastly the protocol allows for optional query information passed _after the last parameter_ in the `key=value`-convention for control flow and must ONLY be used for that.


### Client

```
safeURI:action[:payload][?q=1]
```

where `action` is a particular string identifying the action (like `access-granted`) and `payload` is a `base64`-encoded serialised Rust Struct. As the appID is already used in the schema, it is not required as a separate parameter. Lastly the protocol allows for optional query information passed _after the last parameter_ in the `key=value`-convention for control flow and must ONLY be used for that.


_Note_: The protocol leaves open whether further colon delimited parameters might be added later (in particular we are thinking of requiring a signature of the given string at the end). An implementation must NOT break because there are more entries, it MAY rely on the order however.


### Examples:

```
safeauth:ping:bmV0Lm1haWRzYWZlLmV4YW1wbGUuaGVsbG8K:eyAiaWQiOiAibmV0Lm1haWRzYWZlLmV4YW1wbGUuaGVsbG8iIH0K
```

This example represents a `ping`-request from the appId `net.maidsafe.example.hello` with a JSON payload of `{ "id": "net.maidsafe.example.hello" }` to the authenticator.


```
safeauth-bmV0Lm1haWRzYWZlLmV4YW1wbGUuaGVsbG8K:pong:eyAiaGVsbG8iOiAid29ybGQiIH0K
```

This example represents a `pong`-response from the authenticator to the app register with ID `net.maidsafe.example.hello` with a JSON payload of `{ "hello": "world" }`.


## Process flow parameters

As most of the protocol relies has a request-response-style pattern, but there is no direct connection between both parties to reliably assign those, the protocol offers the special `riq`-process-control parameter: if it is present in the request, it must be passed back on the response unchanged. By using unique identifier when generating a request and putting it in there, the app can identify which request a particular response belongs to.

_Example_:

When the app `net.maidsafe.example.hello` send the following request:

```
safeauth:ping:bmV0Lm1haWRzYWZlLmV4YW1wbGUuaGVsbG8K?riq=18hae
```

The authenticator must respond with

```
safeauth-bmV0Lm1haWRzYWZlLmV4YW1wbGUuaGVsbG8K:pong?riq=18hae
```

informing the app that it responds to the request with the unique id `18hae`

If the request can't be understood, but a `riq` is found, the Authenticator must respond with an `error`-action to finish allow that app to finish its `riq` properly and report the problem to the user.

## Basic Structs


```rust
pub enum PermissionAccess {
  READ,
  INSERT,
  UPDATE,
  DELETE
}

pub struct ContainerPermission {
  // a mapping of container key and granted permissions
  // TODO: shouldn't we do a `dataId` when storing this?
  pub container_key: String,
  pub access: Vec<PermissionAccess>
}

pub struct AppExchangeInfo {
  // ID of the app. It must be a network wide unique name. If the ID changes, the app will not be able to access its data anymore.
  pub id: String,
  // some apps run on multiple devices or different contexts, this allows you to specify which on this is.
  pub scope: Option<String>,
  // (human readable) Name of the app requesting authorization with the SAFE Authenticator.
  pub name: String,
  // (human readable) Name of the vendor of the app.
  pub vendor: String,
  // Version of the 
  pub version: String,
}


pub struct AppAccessToken {
  // TODO: can't we make this a little more specific?
  pub enc_key: secrectbox::KBYTES,
  pub sign_key_public: sign::PUBLICKBYTES,
  pub sign_key_private: sign::PRIVATEKBYTES
}


```

## Actions

### Authentication

This is more less a direct translation of the former [REST Authorisation API](https://api.safedev.org/auth/authorize-app.html) into this protocol. The usage of the `riq`-parameter is recommended.

**Request**:

* Action: `auth`
* payload:

```rust
pub struct AuthRequest {
    app: AppExchangeInfo,
    app_container: Bool,
    containers: Vec<ContainerPermission>
}
```

**Success Response**

If the user granted access to the app, it will receive a URI with:

* Action: `auth-granted`
* payload:

```rust
pub struct AuthGranted {
    // the access token to the network
    access_token: AppAccessToken,
    bootstrap_config: Vec<u8>,
    access_container: Option<DataId>,
    containers: Vec<ContainerPermission>
}
```

**Error Response**

If the user denied access, but is redirected to the app, it will be with the `auth-denied` action:

- Action: `auth-denied`
- Payload: `None`

#### Revokation

If the user revokes access to an app, the authenticator MAY inform the app about this with the `auth-revoked` action:

- Action: `auth-revoked`
- Payload: `None`


## Shared Container Access

In order to request access to a specific shared User container, the app can send a `containers` request to the authenticator:

**Request**

* Action: `containers`
* payload: `Vec<ContainerPermission>`

and convert that into base64 for the request.

**Success Response**

If the user granted access to _at least one_ container the authenticator must send a success response. Because of that the app must check the resulting payload for the keys _and the access level it was granted_ explicitly before continuing. The app should assume that the user knows what it wanted to grant and continue with whatever limited access it was given.

* Action `containers-granted`
* payload: `Vec<ContainerPermission>`

From this point on the app can look up the local encryption key and the usage convention in its `AccessContainer`, see the container Appendix for more.

**Error Response**

If the user didn't respond to the request and denied access to all requested container, the App will receive the `containers-denied` action:

* Action: `containers-denied`
* Payload: `None`

### Revokation

If the user revokes access to one or many containers, the authenticator MAY inform the app about this with the `containers-revoked` action:

* Action: `containers-revoked`
* payload: `Vec<ContainerPermission>`

When an app receives this message, it should prompt the user asking about whether it should try to retry before starting the process again and not do so, if the user denies. 

## Generic Errors

If an error occurs that isn't covered by specific request-response flow actions as defined here, the catchall `error`-action must be used in order to inform about the problem. It has the following format:

* Action: `error`
* payload:

```rust
pub struct GenericAuthError {
  // numeric code of this error (for easy checking)
  code: uint,
  // string name of that code (internal)
  error: String,
  // A message as it can (and should) be displayed to the user
  message: String,
  // More details about the error, if it can be provided by the app, allowing for easier debugging.
  details: Option<String>,
  // A url with further information for the developer to understand why this error happened and how to fix it.
  ref: Option<String>
}
```


In general client-side errors are in the 4000-4999 range, while authenticator side errors are in the 5000-5999 range (assigned as they will be defined).

**Common app caused errors**:

* 4001 `UNKNOWN_ACTION`: the action supplied isn't known to the other party
* 4002 `MISSING_PARAMETER`: a mandatory parameter has not been supplied. The `message` states which one (or many).
* 4003 `MALFORMED_PARAMETER`: One or more parameters supplied are not formatted properly. The `message` states which one (or many).
* 4004 `BAD_PARAMETER`: One or more parameters supplied turned out to be faulty. For example because they hold missing reference, are of invalid value, are not allowed in certain combination or would put the system into a faulty state. The `message` states which one (or many) and their problem.
* 4005 `MISSING_PERMISSION`: the requesting party doesn't have the permission to perform this action
* 4006 `DENIED`: the app has been denied and tried too often, it has been black listed and no response will be answered going forward (until the user reacts)

**Common authenticator caused error**

* 5001 `INTERNAL_ERROR`: catch-all for any error caused by the action performed that isn't defined here yet
* 5002 `USER_INTERVENTION_NEEDED`: the authenticator is in an unrecoverable state the requires user intervention. The app inform the user about that and tell them to check their authenticator.
* 5003 `NOT_IMPLEMENTED`: this authenticator knows about the action but it isn't (fully) implemented yet
* 5004 `LOST_CONNECTION`: the authenticator lost connection to the network and can't perform the request at this time, try again later
