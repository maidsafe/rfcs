# Launcher Not as a Service

- Status: rejected
- Type: New Product
- Related components: [safe_client](https://github.com/maidsafe/safe_client), [safe_nfs](https://github.com/maidsafe/safe_nfs), [safe_vault](https://github.com/maidsafe/safe_vault)
- Start Date: 11-September-2015
- RFC PR: #41

## Summary

Launcher will be a gateway for any app that wants to work on the SAFE Network on a user's behalf. It will generate a single set of ownership and crypto keys which it will give to all the apps that want to access the Network on user's behalf. It will also give access to `SAFEDrive` with an intention to allow apps to share data among themselves.

## Motivation

### Why?

App's access of the SAFE Network on behalf of the user is an issue with high security concerns. Without Launcher, every app would ask for user credentials to log into the Network. This means that sensitive information like user's session packet etc., are compromised and can be potentially misused. Launcher will prevent this from happening by being the only one that gets the user credential. Apps only communicate with the Network with keys given by Launcher on user's behalf. These will be different to the user's MAID keys.

### What cases does it support?

Launcher

1. will allow user to create an account and/or log into the SAFE Network.

1. will authenticate a user installed app to access SAFE Network on the user's behalf.

1. will manage metadata related to apps to give uniformity in experience when shifting from one machine to another - e.g. if app `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE Account via Launcher, he/she will be presented with a union of all the apps that were installed on all the machines which access the SAFE Network on his/her behalf.

1. along with [safe_vault](https://github.com/maidsafe/safe_vault) will manage the mapping and de-mapping of ownership keys for an app.
## Detailed design

### User's Login Session Packet (for reference)
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

### Start Flow

**step 0:** Start Launcher.

**step 1:** Enter credentials to either create an account or to log into a previously created one.

**step 2:** If it was a log in, Launcher fetches and decodes User-Session-Packet (USP).

**step 3:** Launcher fetches `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>` (See Session Packet description above) - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 4:** Launcher reads the special directory reserved for it - `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory`; (See Session Packet Description above) - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 5:** Launcher fetches `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>` - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 6:** Launcher will generate `<Launcher Keys>` which is a pair of encryption and ownership keypairs. These keys will be given to all the apps that are started from Launcher.

**step 7:** Launcher checks for special directory named `SAFEDrive` inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/SAFEDrive` - if not present Launcher will create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate). This is the directory that the user will usually see it mounted as root during VFS mounts. Also this is the directory the apps are expected to use to share data and will be encrypted and signed using `<Laucher Keys>`.

**step 8:** Launcher will listen on local host: `127.0.9.9:30000`. If unavailable, it will increment the port till a valid TCP Listener is in place. This will be thus `127.0.9.9:Launcher-Port`. This will remain on till Launcher is closed. We will call this combination `<Laucher-IP:Launcher-Port>`.

### Add App Flow

**step 0:** User drags `XYZ` App binary into the Launcher to add it. Launcher will ask the user if the app should have access to `SAFEDrive`.

**step 1:** Launcher creates (if it’s the first time it saw this App):
1. Unique random 64 byte ID for this app - App-ID (because names of different binaries can be same if they are in different locations on the same machine - thus we need a unique identifier for the binary across all machines)
2. Unique Directory Id (a 64 byte ID associated with a directory) and an associated unique Root [Directory Listing](https://github.com/maidsafe/safe_nfs/blob/master/src/directory_listing/mod.rs) `<APP-ROOT-DIR>` for this app - `XYZ-Root-Dir`. For directory name conflicts append numbers so that they can exist on a file system - e.g. `XYZ-1-Root-Dir`. This shall be created inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/`.
- `<APP-ROOT-DIR>` shall be always **unversioned** and **private** (i.e. encrypted with `<Launcher Keys>`). Any requirement for a versioned or public directory from the app can be managed by the app itself by creating further subdirectories.
- The user is asked if this app should be given read/write access to the `SAFEDrive` directory.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - format shall be CBOR (concise-binary-object-representation).
```
launcher_keys: (sodiumoxide::crypto::box_::gen_keypair(),
                sodiumoxide::crypto::sign::gen_keypair()),
[
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        Option<SAFEDrive-Directory-Key>,
    }
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        Option<SAFEDrive-Directory-Key>,
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

**step 2:** User activates the app (e.g. double click) from within the Launcher.

**step 3:** Launcher checks the App-ID, reads the path from the `<LOCAL-CONFIG-FILE>` that it made and starts the app as an independent process. The Launcher supplies a random port on which it will listen to this app via command line options.

`/path/to/app/binary --launcher "tcp:<Laucher-IP:Launcher-Port>:<random-launcher-string>"`

All parameters are UTF-8 strings.

**step 4:** App generates a random asymmetric encryption keypair - `<App-Asymm-Keys>`. Then it connects to Launcher on the given endpoint asking for Launcher to give it an `<App-Specific-Symm-Key>`, its root directory-key and SAFEDrive directory-key.
- The payload format for this request shall be a CBOR encoded structure of the following:
```
struct Request {
    launcher_string      : String, // This must be the one supplied by Launcher
    nonce                : sodiumoxide::crypto::box_::Nonce,
    public_encryption_key: sodiumoxide::crypto::box_::PublicKey, // from <App-Asymm-Keys>
}
```

**step 5:** The Launcher verifies the `launcher_string` field and encrypts `<Launcher Keys>` using app's `public_encrytion_key` and `nonce` above. If Launcher had not previously created a mapping for ownership keys in `<Launcher Keys>` in Vaults, it will do so now as follows:
- The mapping is done only for the ownership keys (not encryption keys) and is done only when the first app is added.
- The request for mapping/un-mapping shall be a command to the `MaidManagers`. For commands to `MaidManagers`, `StructuredData` with special type-tag will be reserved. On reception of this `StructuredData` via a `POST` message explicity directed towards `MaidManagers` the vaults will check the payload and act upon the request instead of executing a normal reaction to the usual `POST`s.
- `StructuredData` with Tag-Type **9** shall be reserved for `Client <-> Vault` messages.
- The messages will be defined in [safe_vault](https://github.com/maidsafe/safe_vault).
- This shall be the format of `Client <-> Vault` messages:
```
pub enum ClientVaultMessage {
    MapKeys {
        ownership_key_to_map: sodiumoxide::crypto::sign::PublicKey, // Launcher-Assigned-Public-Key-For-App
    },
    UnMapKeys {
        ownership_key_to_unmap: sodiumoxide::crypto::sign::PublicKey, // Launcher-Assigned-Public-Key-For-App
    },
}
```
- These messages are acted upon by the vaults only if the message came from original owner, ie., the mapped owners shall not be allowed to perform these operations.

**step 6:** Launcher gives the app what it requested.
- The payload format for this response shall be a CBOR encoded structure of the following:
```
struct Response {
    cipher_text       : Vec<u8>, // encrypted symmetric keys
    app_root_dir_key  : DirectoryKey,
    safe_drive_dir_key: Option<DirectoryKey>, // if user gave permission
    tcp_listening_port: u16, // <Laucher-Port>
}
```
where `DirectoryKey` is defined [here](https://github.com/ustulation/safe_nfs/blob/master/src/metadata/directory_key.rs).
- Launcher closes the connection and is no longer needed for this session for this app.

### Reads and Mutations by the App

- All `GET/PUT/POST/DELETE`s will go directly to the Network. The app will use `<Launcher Keys>` to encrypt and sign data.

### Remove App Flow

**procedure 0:** Launcher removes the App as follows:
- Delete from `<LOCAL-CONFIG-FILE>` (on the user's machine) the following:
```
[
    { App-ID, “/path/to/XYZ” }, // Remove this, other entries are untouched.
    ...
]
```
- Remove the SHA512(App-Binary) from the vector in `<LAUNCHER-CONFIG-FILE>`.
- Decrement `Reference Count` from `<LAUNCHER-CONFIG-FILE>`.
- If the `Reference Count` is **0** it means that this is the last machine where the App was present. The Launcher shall not delete `<APP-ROOT-DIR>`. It is user's responsibility to do that as it might contain information (like pictures, etc.) which the user may not want to lose. Instead the Launcher will ask if the user wants to do that and act accordingly. Launcher will however remove the app entry from the `<LAUNCHER-CONFIG-FILE>`.

**procedure 1:** While the other procedure would work, there might be occassions when the user wants to immediately remove the app completely (which also translates as revoke App's permission to mutate network on user's behalf). He may not have access to other machines where the App was installed and may be currently running and the previous procedure requires the user to remove it from all machines. Thus there shall be an option in Launcher to remove App completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

### Misc
- If the App is added to Launcher in one machine, the mention of this will go into `<LAUNCHER-CONFIG-FILE>` as stated previously. It will thus be listed on every machine when user logs into his/her account via Launcher on that machine. However when the App is attempted to be activated on a machine via Launcher where it was not previously added to Launcher then he/she will be prompted to associate a binary. Once done, the information as usual will go into the `<LOCAL-CONFIG-FILE>` on that machine and the user won't be prompted the next time.
- When the App is started via Launcher, it will first check if the `SHA512(App-Binary)` matches any one of the corresponding entries in the vector in `<LAUNCHER-CONFIG-FILE>`. If it does not Launcher will interpret this as a malacious App that has replaced the App-binary on user's machine, and thus will show a dialog to the user for confirmation  of whether to still continue, because there can be genuine reasons for binary not matching like the App was updated etc.

## Alternatives
There is an RFC proposal for a more robust and complicated design of Launcher which aims at solving many more security concerns.

## Drawbacks
1. Since same keys are used by all apps, an app can easily gain access to another app's root directory and config files.
1. Since same keys are used by all apps, an app can continue to access the Network even after it's removal from Launcher. Revocation is not easy and is a tedious process of fetching all user data from the Network, and encrypting and signing them with different keys and making these available to all other authenticated apps (in all machines). The vaults too must be informed and asked to un-map the previous keys and map new ones.
