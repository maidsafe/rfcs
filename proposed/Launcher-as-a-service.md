- Feature Name: Launcher as a service
- Type: New Product
- Related components: [safe_client](https://github.com/maidsafe/safe_client), [safe_nfs](https://github.com/maidsafe/safe_nfs)
- Start Date: 11-September-2015

# Summary

Launcher will be a gateway for any app that wants to work on the SAFE Network on a user's behalf. It will run as a background process and will be responsible for decrypting data from the Network and re-encrypting using app specific keys while fetching data on app's behalf and vice-versa during app's request to put/post/delete data on the Network.

# Conventions
The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

# Motivation

## Why?

App's access of the SAFE Network on behalf of the user is an issue with high security concerns. Without Launcher, every app would ask for user credentials to log into the Network. This means that sensitive information like user's session packet etc., are compromised and can be potentially misused. Launcher will prevent this from happening by being the only one that gets the user credential. Apps only communicate with the Network indirectly via Launcher on user's behalf.

## What cases does it support?

Launcher

1. will allow user to create an account and/or log into the SAFE Network.

2. will authenticate a user installed app to access SAFE Network on the user's behalf.

3. will manage metadata related to apps to give uniformity in experience when shifting from one machine to another - e.g. if app `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE Account via Launcher, he/she will be presented with a union of all the apps that were installed on all the machines which access the SAFE Network on his/her behalf.

4. must not allow apps to mutate each other's configs.

5. shall easily revoke app's ability to read and mutate the Network on user's behalf.

# Detailed design

## User's Login Session Packet (for reference)
This is only to provide a context to the references to it below. This might change in future without affecting this RFC (i.e. only a small portion of this is actually relevant for this RFC).
```
Account {
    an_maid,
    maid,
    public_maid,
    an_mpid,
    mpid,
    public_mpid,
    Option<USER’S-PRIVATE-ROOT-DIRECTORY-ID>,
    Option<MAIDSAFE-SPECIFIC-CONFIG-ROOT>,
}
```
- The Root Directories are encrypted with MAID-keys.

## Start Flow

**step 0:** Start Launcher.

**step 1:** Enter credentials to either create an account or to log into a previously created one.

**step 2:** If it was a log in, Launcher fetches and decodes User-Session-Packet (USP).

**step 3:** Launcher fetches `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>` (See Session Packet description above) - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 4:** Launcher reads the special directory reserved for it - `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory`; (See Session Packet Description above) - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 5:** Launcher fetches `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>` - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 6:** Launcher checks for special directory named `SAFEDrive` inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/SAFEDrive` - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate). This is the directory that the user will usually see it mounted as root.

**step 7:** Launcher will listen on local host: `127.0.9.9:30000`. If unavailable, it will increment the port till a valid TCP Listener is in place. This will be thus `127.0.9.9:Launcher-Port`. This will remain on till either the OS is shutdown or Launcher Background Process is killed. We will call this combination `<Launcher-IP:Launcher-Port>`.

## Add App Flow

**step 0:** User drags `XYZ` app binary into Launcher to add it. Launcher will ask the user if the app should have access to `SAFEDrive`.

**step 1:** Launcher creates (if it’s the first time it saw this app):
1. Unique random 64 byte ID for this app - App-ID (because names of different binaries can be same if they are in different locations on the same machine - thus we need a unique identifier for the binary across all machines)
2. Unique Directory Id (a 64 byte ID associated with a directory) and an associated unique Root [Directory Listing](https://github.com/maidsafe/safe_nfs/blob/master/src/directory_listing/mod.rs) `<APP-ROOT-DIR>` for this app - `XYZ-Root-Dir`. For directory name conflicts append numbers so that they can exist on a file system - e.g. `XYZ-1-Root-Dir`. This shall be created inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/`.
- `<APP-ROOT-DIR>` shall be always **unversioned** and **private** (i.e. encrypted with app-specific crypto keys). Any requirement for a versioned or public directory from the app can be managed by the app itself by creating further subdirectories.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - format shall be CBOR (concise-binary-object-representation).
```
[
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        bool, // if access to SAFEDrive is allowed
        OtherMetadata, // For Future Use
    }
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        bool, // if access to SAFEDrive is allowed
        OtherMetadata, // For Future Use
    },
    … etc
]
```
- Create/Append to local config file (on the user's machine) the following:
```
[
    { App-ID, “/path/to/XYZ” }, // This is newly added by the above
    { App-ID, “/path/to/some/other/app/already/there” },
    ... etc.,
]
```
- The format of the config file will be CBOR (concise-binary-object-representation). The name of the local config file should be `<LOCAL-CONFIG-FILE> = launcher.config`. The config file location flowchart shall be [same as that of `crust` crate's](https://github.com/maidsafe/crust/blob/03d63b4526cf87667c1d1775377398875072c12a/docs/vault_config_file_flowchart.pdf).
- The local config file should be encrypted with crypto keys held in session packet and also signed with MAID keys.

**step 2:** User activates the app (e.g. double click) from within Launcher.

**step 3:** Launcher checks the App-ID, reads the path from the `<LOCAL-CONFIG-FILE>` that it made and starts the app as an independent process. Launcher supplies a random port on which it will listen to this app via command line options.

`/path/to/app/binary --launcher "tcp:<Launcher-IP:Launcher-Port>:<random-launcher-string>"`

All parameters are UTF-8 strings.

**step 4:** App generates a random asymmetric encryption keypair - `<App-Asymm-Keys>`. Then it connects to Launcher on the given endpoint asking for Launcher to give it an `<App-Specific-Symm-Key>`, its root directory-key and SAFEDrive directory-key, which Launcher had reserved as `XYZ-Root-Dir`
- The payload format for this request shall be a JSON encoded structure of the following:
```
{
    "rsa_key_exchange_request": {
        "launcher_string"      : String,        // This shall be the one supplied by Launcher
        "nonce"                : [ uint8 ... ], // sodiumoxide::crypto::box_::Nonce,
        "public_encryption_key": [ uint8 ... ]  // sodiumoxide::crypto::box_::PublicKey from
                                                // <App-Asymm-Keys>
    }
}
```

**step 5:** Launcher verifies the `launcher_string` field above and generates a strong random symmetric encryption key `<App-Specific-Symm-Key>`. This is encrypted using app's `public_encrytion_key` and `nonce` above.

**step 6:** Launcher gives the app what it requested concluding the RSA key exchange procedure.
- The payload format for this response shall be a JSON encoded structure of the following:
```
{
    "rsa_key_exchange_response": {
        "cipher_text": [ uint8 ... ] // encrypted symmetric keys
    }
}
```

- From this point onwards all data exchanges between Launcher and the app will happen in JSON format, subsequently encrypted by `<App-Specific-Symm-Key>`.

## Reads and Mutations by the App

- Every service provided by Launcher will be documented in Launcher service document (a separate RFC). The communication between Launcher and an app shall be in JSON subsequently encrypted by `<App-Specific-Symm-Key>`.
- The services provided by Launcher and their format are prone to change, hence every new document will have a version information. An app may do a version negotiation anytime after a successful RSA key exchange. Unless an explicit version negotiation happens at-least once, Launcher may default to the latest version. The version negotiation will happen via documented JSON format - e.g. of probable format:
```
{
    "version": "x.y.z" // where x.y.z could be 2.10.39 etc
}
```
- The reqest shall identify a module and an action as a minimum and then a payload, which could be a nested structure, for the corresponding action. The following is not a specification but just an e.g.
```
Modules {
    "NFS",
    "DNS"
}

NfsActions {
    "create-directory",
    "delete-directory",
    "create-file",
    "delete-file"
}

// Request from app to Launcher:
{
    "module": "NFS",
    "action": "create-directory",
    "payload": {
        "is_shared": true, // if path starts from SAFEDrive (true)
                           // or APP-ROOT-DIR (false)
        "path": "/path/to/example-directory", // path root will be interpreted according
                                              // the parameter above. The last token in
                                              // the path will be interpreted as the name
                                              // of directory to be created.
        "is_private": true,
        "is_versioned": false,
        "user_metadata": [       // array of uint8 - could represent binary data
            38,
            65,
            255
        ]
    }
}
```
- Errors will be given back. This is again an e.g. and not a specification:
```
// Error Response from Launcher to app:
{
    "error_code": 15,
    "details": "ClientError::AsymmetricDecipherFailure"
}
```

Since `<App-Specific-Symm-Key>` is recognised by Launcher only for current session, there is no security risk and the app will not be able to trick Launcher the next time it starts to use the previous keys to mutate network or read data on its behalf.

## Share Directory App Flow

Every time the app tries to access `SAFEDrive` Launcher will check the permission in `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile`.

### Grant and Revoke Access

- User can grant the app read/write access to the `SAFEDrive` directory or revoke that access by asking Launcher to do so.

## Remove App Flow

**procedure 0:** Launcher removes the app as follows:
- Delete from `<LOCAL-CONFIG-FILE>` (on the user's machine) the following:
```
[
    { App-ID, “/path/to/XYZ” }, // Remove this, other entries are untouched.
    ...
]
```
- Remove the SHA512(App-Binary) from the vector in `<LAUNCHER-CONFIG-FILE>`.
- Decrement `Reference Count` from `<LAUNCHER-CONFIG-FILE>`.
- If the `Reference Count` is **0** it means that this is the last machine where the app was present. Launcher shall not delete `<APP-ROOT-DIR>` without user intervention. It is user's responsibility to do that as it might contain information (like pictures, etc.) which the user might not want to lose. Instead Launcher will ask if the user wants to do that and act accordingly. Similarly only after user confirmation will Launcher remove the app entry from the `<LAUNCHER-CONFIG-FILE>`, as it contains necessary metadata to decrypt the app specific directories.

**procedure 1:** While the other procedure would work, there might be occassions when the user wants to immediately remove the app completely (which also translates as revoke app's permission to mutate network on user's behalf). The user might not have access to other machines where the app was installed and could be currently running and the previous procedure requires the user to remove it from all machines. Thus there shall be an option in Launcher to remove app completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

In both procedures, Launcher will terminate the TCP connection with the app and forget the `<App-Specific-Symm-Keys>` so effect of removal and hence revocation is immediate.

## Misc
- If the app is added to Launcher in one machine, the mention of this will go into `<LAUNCHER-CONFIG-FILE>` as stated previously. It will thus be listed on every machine when user logs into his/her account via Launcher on that machine. However when the app is attempted to be activated on a machine via Launcher where it was not previously added to Launcher then he/she will be prompted to associate a binary. Once done, the information as usual will go into the `<LOCAL-CONFIG-FILE>` on that machine and the user won't be prompted the next time.
- When the app is started via Launcher, it will first check if the `SHA512(App-Binary)` matches any one of the corresponding entries in the vector in `<LAUNCHER-CONFIG-FILE>`. If it does not Launcher will interpret this as a malacious app that has replaced the App-binary on user's machine, and thus will show a dialog to the user for confirmation  of whether to still continue, because there can be genuine reasons for binary not matching like the app was updated etc.

# Alternatives
There is an RFC proposal for a simplified version of Launcher which does not go to as deep an extent to cover all facets of security.

# Current Limitations and Future Scope
- All apps need to be started from within Launcher. This is a current limitation and possible future solution is to have Launcher write it's listening TCP endpoint to publicly readable file in a fixed location. The app-devs will be asked to construct shortcuts to their apps such that when activated the shortcut points to a binary that reads the current endpoints from the mentioned file, connects to the Launcher there passing it the path to the actual app binary and finally terminating itself. Launcher then checks for this path's validity in its local config file and starts the app as usual (as described in this RFC). An alternative to a file containing public readable endpoints could be a fixed UDP endpoint on which Launcher listens for path to binaries given by app-shortcuts.
- There is no provision for an app that is required to be started at system start up. For this we can have Launcher marked as a startup process and all apps (which make use of Launcher) that need to be activated at system start up be marked thus in Launcher. Launcher would then activate these once it has itself been successfully activated.

# Unresolved questions
**(Q0)** A local background service process to channel all requests through while may be ok for desktop platforms, might definitely need a feasibility check on mobile platforms. Would this approach work for mobiles ?

# Appendix

## Implementation hints

- Crate level design:

```
            safe_launcher_ui
                    |
            safe_launcher_core
                    |
   ------------------------------------
  |                      |             |
safe_dns             safe_nfs      safe_client
    |                    |
 -----------         safe_client
|           |
safe_nfs  safe_client
```

- Module level design:
```
            safe_launcher_core
                    |
    ---------------------------------
   |               |                 |
  ipc        app_handling           ffi
```

### ffi
This module is intended to interface with code written in other languages especially for Launcher-UI. FFI for self-authentication must be provided as this will provide the client-engine necessary to do anything useful in the SAFE Network:
```
/// Create an unregistered client. This or any one of the other companion functions to get a
/// client must be called before initiating any operation allowed by this crate.
#[no_mangle]
pub extern fn create_unregistered_client(client_handle: *mut *const libc::c_void) -> libc::int32_t {
    unsafe {
        *client_handle = cast_to_client_ffi_handle(ffi_try!(safe_client::client::Client::create_unregistered_client()));
    }

    0
}

/// Create a registered client. This or any one of the other companion functions to get a
/// client must be called before initiating any operation allowed by this crate. `client_handle` is
/// a pointer to a pointer and must point to a valid pointer not junk, else the consequences are
/// undefined.
#[no_mangle]
pub extern fn create_account(c_keyword    : *const libc::c_char,
                             c_pin        : *const libc::c_char,
                             c_password   : *const libc::c_char,
                             client_handle: *mut *const libc::c_void) -> libc::int32_t {
    let client = ffi_try!(safe_client::client::Client::create_account(ffi_try!(implementation::c_char_ptr_to_string(c_keyword)),
                                                                      ffi_try!(implementation::c_char_ptr_to_string(c_pin)),
                                                                      ffi_try!(implementation::c_char_ptr_to_string(c_password))));
    unsafe { *client_handle = cast_to_client_ffi_handle(client); }

    0
}

/// Log into a registered client. This or any one of the other companion functions to get a
/// client must be called before initiating any operation allowed by this crate. `client_handle` is
/// a pointer to a pointer and must point to a valid pointer not junk, else the consequences are
/// undefined.
#[no_mangle]
pub extern fn log_in(c_keyword    : *const libc::c_char,
                     c_pin        : *const libc::c_char,
                     c_password   : *const libc::c_char,
                     client_handle: *mut *const libc::c_void) -> libc::int32_t {
    let client = ffi_try!(safe_client::client::Client::log_in(ffi_try!(implementation::c_char_ptr_to_string(c_keyword)),
                                                              ffi_try!(implementation::c_char_ptr_to_string(c_pin)),
                                                              ffi_try!(implementation::c_char_ptr_to_string(c_password))));
    unsafe { *client_handle = cast_to_client_ffi_handle(client); }

    0
}

/// Discard and clean up the previously allocated client. Use this only if the client is obtained
/// from one of the client obtainment functions in this crate (`crate_account`, `log_in`,
/// `create_unregistered_client`). Using `client_handle` after a call to this functions is
/// undefined behaviour.
#[no_mangle]
pub extern fn drop_client(client_handle: *const libc::c_void) {
    let _ = unsafe { std::mem::transmute::<_, Box<std::sync::Arc<std::sync::Mutex<safe_client::client::Client>>>>(client_handle) };
}

fn cast_to_client_ffi_handle(client: safe_client::client::Client) -> *const libc::c_void {
    let boxed_client = Box::new(std::sync::Arc::new(std::sync::Mutex::new(client)));
    unsafe { std::mem::transmute(boxed_client) }
}

fn cast_from_client_ffi_handle(client_handle: *const libc::c_void) -> std::sync::Arc<std::sync::Mutex<safe_client::client::Client>> {
    let boxed_client: Box<std::sync::Arc<std::sync::Mutex<safe_client::client::Client>>> = unsafe {
        std::mem::transmute(client_handle)
    };

    let client = (*boxed_client).clone();
    std::mem::forget(boxed_client);

    client
}
```
These are already stable and coded in [safe_ffi crate](https://github.com/maidsafe/safe_ffi/blob/master/src/lib.rs) and code there can be resused. Obtained `client_handle` must be passed around and carefully destroyed when shutting down Launcher.

Another approach (instead of passing `client_handle` to and fro the FFI) could be to populate the obtained client_handle into a singleton pointer and access that from various modules. A quick reference to form a singleton cna be found in [safe_client crate here](https://github.com/maidsafe/safe_client/blob/master/src/client/non_networking_test_framework/mod.rs#L48).

Apart from this FFI will evolve more and more as Launcher-UI takes shape in future. It will provide convenient ways to interface with other core Launcher modules.

### app_handling
This module will contain rust code to be invoked when apps are dropped into Launcher, are removed from it or have related parameters (`SAFEDrive` authorisation) changed. This module will be responsible for handling of `LauncherConfigurationFile` and local config file. Some hint of this can already be found in the way `safe_dns` handles `DnsConfigurationFile` [here](https://github.com/maidsafe/safe_dns/blob/master/src/dns_operations/dns_configuration.rs#L29).

### ipc
A TCP listener would try and bind to `127.0.9.9:30000`. If unsuccessfull keep incrementing the port number. Once that reaches 65535 and still no available ports are found, increment the IP and repeat the procedure for `127.0.9.10:30000`. Once bound to an endpoint, keep a note of it to be returned when asked by `app_handling` module for invoking an app with command line parameters.
```
pub fn get_launcher_endpoint() -> std::net::SocketAddr;
```
Each incoming TCP connection request will spawn a new thread and pass the socket details to a new instance of `AppSession` below. An `AppSession` instance uniquely represents a single Launcher-App session. Cleanup codes should preferably be _lazy_.
```
pub enum Permission {
    None,
    ReadOnly,
    Full,
}

pub trait SessionState {
    fn execute(remaining_states: Vec<Box<SessionState>>);
    fn terminate();
}

pub struct AppSession {
    client          : std::sync::Arc<std::sync::Mutex<safe_client::client::Client>>,
    stream          : std::net::TcpStream,
    remote_peer     : std::net::SocketAddr,
    share_permission: Permission,
    states          : Vec<Box<SessionState>>, // will be popped at each stage
}

pub struct VerifyLauncherNonce;
impl SessionState for VerifyLauncherNonce { ... }

pub struct RSAKeyExchange;
impl SessionState for RSAKeyExchange { ... }

pub struct SecureCommunication;
impl SessionState for SecureCommunication { ... }
```
`SecureCommunication` could spawn a new thread for `GET`s and wait for the result. It can either timeout or wait indefinitely and can be dynamically configured by the app. The app should also get a chance to cancel all pending `GET`s. This is where a cloned `std::sync::mpsc::sender` from `safe_client::client::response_getter::ResponseGetter` ([reference](https://github.com/maidsafe/safe_client/blob/master/src/client/response_getter.rs#L71)) could be useful.

Also `SecureCommunication` could have composition of a type that marshalls data to and from Rust-structures and JSON. This should preferably be modularised (as parsers usually are).
