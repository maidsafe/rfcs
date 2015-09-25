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
    Option<USER’S-PRIVATE-ROOT-DIRECTORY-ID>, // This is easily accessible to the user
                                              // (i.e. mounted as a drive etc.)
    Option<MAIDSAFE-SPECIFIC-CONFIG-ROOT>     // This is accessible only if specifically
	                                      // asked and comes with a warning to not directly modify it.
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

**step 7:** Launcher will listen on local host: `127.0.9.9:30000`. If unavailable, it will increment the port till a valid TCP Listener is in place. This will be thus `127.0.9.9:Launcher-Port`. This will remain on till either the OS is shutdown or Launcher Background Process is killed. We will call this combination `<Laucher-IP:Launcher-Port>`.

## Add App Flow

**step 0:** User drags `XYZ` App binary into the Launcher to add it. Launcher will ask the user if the app should have access to `SAFEDrive`.

**step 1:** Launcher creates (if it’s the first time it saw this App):
1. Unique random 64 byte ID for this app - App-ID (because names of different binaries can be same if they are in different locations on the same machine - thus we need a unique identifier for the binary across all machines)
2. Unique Directory Id (a 64 byte ID associated with a directory) and an associated unique Root [Directory Listing](https://github.com/maidsafe/safe_nfs/blob/master/src/directory_listing/mod.rs) `<APP-ROOT-DIR>` for this app - `XYZ-Root-Dir`. For directory name conflicts append numbers so that they can exist on a file system - e.g. `XYZ-1-Root-Dir`. This shall be created inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/`.
- `<APP-ROOT-DIR>` shall be always **unversioned** and **private** (i.e. encrypted with app-specific crypto keys). Any requirement for a versioned or public directory from the app can be managed by the app itself by creating further subdirectories.
- The user is asked if this app should be given read/write access to the `SAFEDrive` directory.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - format shall be CBOR (concise-binary-object-representation).
```
[
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>, // Also serves to tell how many machines where the App is added
                                 // to Launcher
        <APP-ROOT-DIR-Key>,
        Option<SAFEDrive-Directory-Key>,
        OtherMetadata, // For Future Use
    }
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>, // Also serves to tell how many machines where the App is added
                                 // to Launcher
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

**step 2:** User activates the app (e.g. double click) from within the Launcher.

**step 3:** Launcher checks the App-ID, reads the path from the `<LOCAL-CONFIG-FILE>` that it made and starts the app as an independent process. The Launcher supplies a random port on which it will listen to this app via command line options.

`/path/to/app/binary --launcher "tcp:<Laucher-IP:Launcher-Port>"`

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

**step 5:** The Launcher verifies the `launcher_string` field above and generates a strong random symmetric encryption key `<App-Specific-Symm-Key>`. This is encrypted using app's `public_encrytion_key` and `nonce` above.

**step 6:** Launcher gives the App what it requested.
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

- From this point onwards any data in the `data` field of a `StructuredData` ([reference](https://github.com/maidsafe/rfcs/blob/master/active/0000-Unified-structured-data.md)) with private accessibility should be exchanged using the `<App-Specific-Symm-Key>` for this socket.

## Reads and Mutations by the App

- All `GET/PUT/POST/DELETE`s will go via the Launcher. The app will essentially encrypt the `data` field of `StructuredData` using `<App-Specific-Symm-Key>` if the access level for the `StructuredData` happens to be `Private` and pass it to Launcher which will decrypt and re-encrypt and sign it using normal process (currently MAID-keys). For `GET`s it will be the reverse - Launcher will eventually encrypt using `<App-Specific-Symm-Key>` before handing the data over to the App. For `Public` accessed `StructuredData` no such encryption translation will be done by Launcher. `ImmutableData` shall not be inspected by Launcher - it will merely be put or got `PUT/GET` as-is. The motivation for this is `ImmutableData` are expected to be protected via self-encryption. Thus the app has a choice to fetch `ImmutableData` or `StructuredData` directly from the Network if it deems fit. E.g. app can fetch publicly-accessed DNS records without going through Launcher and browsers working with SAFE protocol will definitely do this.

Since `<App-Specific-Symm-Key>` is recognised by Launcher only for current session, there is no security risk and the App will not be able to trick Launcher the next time it starts to use the previous keys to mutate network or read data on its behalf.

## Share Directory App Flow

Every time the App tries to access `SAFEDrive` Launcher will check the permission in `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile`.

### Grant and Revoke Access

- User can grant the app read/write access to the `SAFEDrive` directory or revoke that access by asking the Launcher to do so.

## Remove App Flow

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
- If the `Reference Count` is **0** it means that this is the last machine where the App was present. The Launcher shall not delete `<APP-ROOT-DIR>` from within `SAFEDrive` folder. It is user's responsibility to do that as it might contain information (like pictures, etc.) which the user may not want to lose. Instead the Launcher will ask if the user wants to do that and act accordingly. Similarly only after user confirmation will Launcher remove the App entry from the `<LAUNCHER-CONFIG-FILE>`, as it contains necessary metadata to decrypt the App specific directories.

**procedure 1:** While the other procedure would work, there might be occassions when the user wants to immediately remove the app completely (which also translates as revoke App's permission to mutate network on user's behalf). He may not have access to other machines where the App was installed and may be currently running and the previous procedure requires the user to remove it from all machines. Thus there shall be an option in Launcher to remove App completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

In both procedures, Launcher will terminate the TCP connection with the App and forget the `<App-Specific-Symm-Keys>` so effect of removal and hence revocation is immediate.

## Misc
- If the App is added to Launcher in one machine, the mention of this will go into `<LAUNCHER-CONFIG-FILE>` as stated previously. It will thus be listed on every machine when user logs into his/her account via Launcher on that machine. However when the App is attempted to be activated on a machine via Launcher where it was not previously added to Launcher then he/she will be prompted to associate a binary. Once done, the information as usual will go into the `<LOCAL-CONFIG-FILE>` on that machine and the user won't be prompted the next time.
- When the App is started via Launcher, it will first check if the `SHA512(App-Binary)` matches any one of the corresponding entries in the vector in `<LAUNCHER-CONFIG-FILE>`. If it does not Launcher will interpret this as a malacious App that has replaced the App-binary on user's machine, and thus will show a dialog to the user for confirmation  of whether to still continue, because there can be genuine reasons for binary not matching like the App was updated etc.

# Alternatives
There is an RFC proposal for a simplified version of the Launcher which does not go to as deep an extent to cover all facets of security.

# Unresolved questions
**(Q0)** Once the user has revoked an app's permission to use `SAFEDrive`, how will the Launcher ascertain that `StructuredData` that the app is asking the Launcher to `PUT/POST` to the Network is not related to some directory listing inside the `SAFEDrive` directory ?
