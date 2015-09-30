- Feature Name: Launcher as a service
- Type: New Product
- Related components: [safe_client](https://github.com/maidsafe/safe_client), [safe_nfs](https://github.com/maidsafe/safe_nfs)
- Start Date: 11-September-2015

# Summary

Launcher will be a gateway for any app that wants to work on the SAFE Network on a user's behalf. It will run as a background process and will be responsible for decrypting data from the Network and re-encrypting using app specific keys while fetching data on app's behalf and vice-versa during app's request to put/post/delete data on the Network.

# Motivation

## Why?

App's access of the SAFE Network on behalf of the user is an issue with high security concerns. Without Launcher, every app would ask for user credentials to log into the Network. This means that sensitive information like user's session packet etc., are compromised and can be potentially misused. Launcher will prevent this from happening by being the only one that gets the user credential. Apps only communicate with the Network indirectly via Launcher on user's behalf.

## What cases does it support?

Launcher

1. will allow user to create an account and/or log into the SAFE Network.

2. will authenticate a user installed app to access SAFE Network on the user's behalf.

3. will manage metadata related to apps to give uniformity in experience when shifting from one machine to another - e.g. if app `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE Account via Launcher, he/she will be presented with a union of all the apps that were installed on all the machines which access the SAFE Network on his/her behalf.

4. not allow apps to mutate each other's configs.

5. easily revoke app's ability to read and mutate the Network on user's behalf.

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
- The user is asked if this app should be given read/write access to the `SAFEDrive` directory.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - format shall be CBOR (concise-binary-object-representation).
```
[
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        Option<SAFEDrive-Directory-Key>,
        OtherMetadata, // For Future Use
    }
    {
        App Name,
        Reference Count, // How many machines this app in installed on
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR-Key>,
        Option<SAFEDrive-Directory-Key>,
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
- The payload format for this request shall be a CBOR encoded structure of the following:
```
struct Request {
    launcher_string      : String, // This must be the one supplied by Launcher
    nonce                : sodiumoxide::crypto::box_::Nonce,
    public_encryption_key: sodiumoxide::crypto::box_::PublicKey, // from <App-Asymm-Keys>
}
```

**step 5:** Launcher verifies the `launcher_string` field above and generates a strong random symmetric encryption key `<App-Specific-Symm-Key>`. This is encrypted using app's `public_encrytion_key` and `nonce` above.

**step 6:** Launcher gives the app what it requested.
- The payload format for this response shall be a CBOR encoded structure of the following:
```
struct Response {
    cipher_text       : Vec<u8>, // encrypted symmetric keys
    app_root_dir_key  : DirectoryKey,
    safe_drive_dir_key: Option<DirectoryKey>, // if user gave permission
    tcp_listening_port: u16, // <Launcher-Port>
}
```
where `DirectoryKey` is defined [here](https://github.com/ustulation/safe_nfs/blob/master/src/metadata/directory_key.rs).

- From this point onwards any data in the `data` field of a `StructuredData` ([reference](https://github.com/maidsafe/rfcs/blob/master/active/0000-Unified-structured-data.md)) with private accessibility should be exchanged using the `<App-Specific-Symm-Key>` for this socket. The following shall be the format of Launcher-App messages:
```
enum LauncherMessage {
    // Optional fields mean that they will be sent to the default authority which should
    // suffice for most cases.
    Get(routing::data::DataRequest, Option<routing::authority::Authority>),
    Put(routing::data::Data, Option<routing::authority::Authority>),
    Post(routing::data::Data, Option<routing::authority::Authority>),
    Delete(routing::data::Data, Option<routing::authority::Authority>),
}
```

## Reads and Mutations by the App

- All `GET/PUT/POST/DELETE`s will go via Launcher. The app will essentially encrypt the `data` field of `StructuredData` using `<App-Specific-Symm-Key>` if the access level for the `StructuredData` happens to be `Private` and pass it to Launcher which will decrypt and re-encrypt and sign it using normal process (currently MAID-keys). For `GET`s it will be the reverse - Launcher will eventually encrypt using `<App-Specific-Symm-Key>` before handing the data over to the app. For `Public` accessed `StructuredData` no such encryption translation will be done by Launcher. `ImmutableData` shall not be inspected by Launcher - it will merely be put or got `PUT/GET` as-is. The motivation for this is `ImmutableData` are expected to be protected via self-encryption. Thus the app has a choice to fetch `ImmutableData` or `StructuredData` directly from the Network if it deems fit. E.g. app can fetch publicly-accessed DNS records without going through Launcher and browsers working with SAFE protocol will definitely do this.

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
- If the `Reference Count` is **0** it means that this is the last machine where the app was present. Launcher shall not delete `<APP-ROOT-DIR>`. It is user's responsibility to do that as it might contain information (like pictures, etc.) which the user may not want to lose. Instead Launcher will ask if the user wants to do that and act accordingly. Similarly only after user confirmation will Launcher remove the app entry from the `<LAUNCHER-CONFIG-FILE>`, as it contains necessary metadata to decrypt the app specific directories.

**procedure 1:** While the other procedure would work, there might be occassions when the user wants to immediately remove the app completely (which also translates as revoke app's permission to mutate network on user's behalf). He may not have access to other machines where the app was installed and may be currently running and the previous procedure requires the user to remove it from all machines. Thus there shall be an option in Launcher to remove app completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

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
**(Q0)** Once the user has revoked an app's permission to use `SAFEDrive`, how will Launcher ascertain that `StructuredData` that the app is asking Launcher to `PUT/POST` to the Network is not related to some directory listing inside the `SAFEDrive` directory ?