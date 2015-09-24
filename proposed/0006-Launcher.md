- Feature Name: SAFE-Launcher
- Type: New Product
- Related components: [safe_client](https://github.com/maidsafe/safe_client), [safe_nfs](https://github.com/maidsafe/safe_nfs)
- Start Date: 11-September-2015

# Summary

Launcher will be a gateway for any App that wants to work on the SAFE-Network on a user's behalf.

# Motivation

## Why?

App's access of the SAFE-Network on behalf of the user is an issue with high security concerns. Further, decentralisation allows for uniformity in experience on various machines concerning user installed Apps.

## What cases does it support?

SAFE-Launcher

<1> will allow user to create an account and/or log into the SAFE-Network.

<2> will authenticate a user installed App to access SAFE-Network on the user's behalf.

<3> will manage metadata related to Apps to give uniformity in experience when shifting from one machine to another - eg., if App `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE-Account via Launcher, he/she will be presented with a union of all the Apps that were installed on all the machines which access the SAFE-Network on his/her behalf.

<4> Not allow Apps to mutate each other's configs.

<5> Easily Revoke App's ability to Read and Mutate the Network on User's behalf.

## Expected outcome

SAFE-Launcher

<1> will allow user to create an account and/or log into the SAFE-Network.

<2> will authenticate a user installed App to access SAFE-Network on the user's behalf.

<3> will manage metadata related to Apps to give uniformity in experience when shifting from one machine to another - eg., if App `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE-Account via Launcher, he/she will be presented with a union of all the Apps that were installed on all the machines which access the SAFE-Network on his/her behalf.

<4> Not allow Apps to mutate each other's configs.

<5> Easily Revoke App's ability to Read and Mutate the Network on User's behalf.

# Detailed design

## User's Login Session Packet (for reference)
This is only to provide a context to the references to it below. This might change in future without affecting this RFC (ie., only a small portion of this is actually relevant for this RFC).
```
Account {
    an_maid,
    maid,
    public_maid,
    an_mpid,
    mpid,
    public_mpid,
    Option<USER’S-PRIVATE-ROOT-DIRECTORY-ID>, // This is easily accessible to the user (ie., mounted as a drive etc.).
    Option<MAIDSAFE-SPECIFIC-CONFIG-ROOT>     // This is accessible only if specifically asked and comes with a warning to not directly modify it.
}
```
- The Root Directories are encrypted with MAID-keys.

## Start Flow

**step 0:** Start Launcher.

**step 1:** Enter Credentials to either create an account or to log into a previously created one.

**step 2:** If it was a log in, Launcher Fetches and decodes User-Session-Packet (USP).

**step 3:** Launcher Fetches `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>` (See Session Packet description) - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 4:** Launcher Reads the special Directory reserved for it - `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory`; (See Session Packet Description) - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 5:** Launcher Fetches `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>` - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 6:** Launcher Checks for special Directory named `SAFEDrive` inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/SAFEDrive` - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate). This is the directory that the user will usually see it mounted as root.

**step 7:** Launcher Background Process will listen on local host: `127.0.9.9:30000`. If unavailable, it will increment the port till a valid TCP Listener is in place. This will be thus `127.0.9.9:Launcher-Port`. This will remain on till either the OS is shutdown or Launcher Background Process is killed. We will call this combination `<Laucher-IP:Launcher-Port>`.

## Add App Flow

**step 0:** User Drags `XYZ` App binary into the Launcher to add it.

**step 1:** Launcher Creates (if it’s the first time it saw this App):
- Unique Random 64 Byte Id for this app - App-ID (because names of different binaries can be same if they are in different locations on the same machine - thus we need a unique identifier for the binary across all machines)
- Unique Directory Id and an associated Unique Root Directory Listing `<APP-ROOT-DIR>` for this app - `XYZ-Root-Dir`. For directory name conflicts append numbers so that they can exist on a file system - eg., `XYZ-1-Root-Dir`. This shall be created inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/`.
- `<APP-ROOT-DIR>` shall be always **unversioned** and **private** (ie., encrypted with App-specific crypto keys). Any requirement for a versioned or public directory from the App can be managed by the App itself by creating further subdirectories.
- The user is asked if it's wanted to share the access of SAFEDrive directory with this App.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - Format shall be CBOR (compact-binary-object-representation).
```
[
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>, // Also serves to tell how many machines where the App is added to Launcher
        <APP-ROOT-DIR-Key>,
	Option<SAFEDrive-Directory-Key>,
        OtherMetadata, // For Future Use
    }
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>, // Also serves to tell how many machines where the App is added to Launcher
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
- The format of the config file will be CBOR (compact-binary-object-representation). The name of the local config file should be `<LOCAL-CONFIG-FILE> = launcher.config`. The config file location flowchart shall be same as that of `crust` crate's.

**step 2:** User activates the app (eg., double click) from within the Launcher.

**step 3:** Launcher checks the App-ID, reads the path from the `<LOCAL-CONFIG-FILE>` that it made and starts the app as an independent process. The Launcher supplies a random port on which it will listen to this app via command line options.

**/path/to/app/binary --launcher "udp:<random-utf8-port>:<random-utf8-string>"**

All parameters are utf8 Strings.

**step 4:** Launcher will wait for a predefined time of **15** seconds for data reception on that port. If it times out it will close the socket (release its binding to it)

**step 5:** App generates a random encryption asymmetric keypair - `<App-Asymm-Keys>`. Then it responds on the socket asking for Launcher to give it an `<App-Specific-Symm-Key>`, its root directory-key and SAFEDrive directory-key, which Launcher had reserved as `XYZ-Root-Dir`
- The payload format for this request shall be CBOR encoded sturcture of the following:
```
struct Request {
    launcher_string      : String, // This must be the one supplied by Launcher
    nonce                : sodiumoxide::crypto::box_::Nonce,
    public_encryption_key: sodiumoxide::crypto::box_::PublicKey, // from <App-Asymm-Keys>
}
```

**step 6:** The Launcher verifies the `launcher_string` field above and generates a strong random symmetric encryption key `<App-Specific-Symm-Key>`. This is encrypted using App's `public_encrytion_key` and `nonce` above.

**step 7:** Launcher gives the App what it requested.
- The payload format for this response shall be CBOR encoded sturcture of the following,
```
struct Response {
    cipher_text       : Vec<u8>, // encrypted symmetric keys
    app_root_dir_key  : DirectoryKey,
    safe_drive_dir_key: Option<DirectoryKey>,
    tcp_listening_port: u16, // <Laucher-Port>
}

Where DirectoryKey consists of NameType, u64 (tag), AccessLevel and bool (versioned or not) as defined in safe_nfs crate.
```
- At this point Launcher closes the UDP socket (ie., is no longer bound to it).

## Reads and Mutations by the App

**step 0:** App will connect to the Launcher Background process using `<Launcher-IP>:<Launcher-Port>`.

**step 1:** All `GET/PUT/POST/DELETE`s will go via the Launcher. The App will essentially encrypt using `<App-Specific-Symm-Key>` and pass it to Launcher Background Process which will decrypt and Re-encrypt and sign it using normal process (currently MAID-keys). For `GET`s it will be the reverse - Launcher will eventually encrypt using `<App-Specific-Symm-Key>` before handing the data over to the App.

**step 2:** Since `<App-Specific-Symm-Key>` is recognised by Launcher only for current session, there is no security risk and the App will not be able to trick Launcher the next time it starts to use the previous keys to mutate network or read data on its behalf.

## Share Directory App Flow

Every time the App tries to access `SAFEDrive` Launcher will check the permission in `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile`.
### Grant Access
- User can grant an App's access of `SAFEDrive` for shared access. Launcher will simply add this information to `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile`.

### Revoke Access
- User can revoke an App's access of `SAFEDrive` for shared access. Launcher will simply remove this information from `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile`.

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
- If the `Reference Count` is **0** it means that this is the last machine where the App was present. The Launcher shall not delete `<APP-ROOT-DIR>` from within `SAFEDrive` folder. It is user's responsibility to do that as it might contain information (like pictures etc) which the user may not want to lose. Instead the Launcher will ask if the user wants to do that and act accordingly. Similary only after user confirmation will Launcher remove the App entry from the `<LAUNCHER-CONFIG-FILE>`, as it contains necessary metadata to decrypt the App specific directories.

**procedure 1:** While the other procedure would work, but there might be occassions when the user wants to immediately remove the app completely (which also translates as revoke App's permission to mutate network on user's behalf). He may not have access to other machines where the App was installed and maybe currently running and the previous procedure requires him to remove it from all machines. Thus there shall be an option in Launcher to remove App completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

In both procedures, Launcher will terminate the TCP connection with the App and forget the `<App-Specific-Symm-Keys>` so effect of removal and hence revokation is immediate.

## Misc
- If the App is added to Launcher in one machine, the mention of this will go into `<LAUNCHER-CONFIG-FILE>` as stated previously. It will thus be listed on every machine when user logs into his/her account via Launcher on that machine. However when the App is attempted to be activated on a machine via Launcher where it was not previously added to Launcher then he/she will be prompted to associate a binary. Once done, the information as usual will go into the `<LOCAL-CONFIG-FILE>` on that machine and the user won't be prompted the next time.
- When the App is started via Launcher, it will first check if the `SHA512(App-Binary)` matches any one of the corresponding entries in the vector in `<LAUNCHER-CONFIG-FILE>`. If it does not Launcher will interpret this as a malacious App that has replaced the App-binary on user's machine, and thus will show a dialog to the user for confirmation  of whether to still continue, because there can be genuine reasons for binary not matching like the App was updated etc.

# Alternatives
None yet.

# Unresolved questions
**(Q0)** How will an App developer register himself?

**(Q1)** How will the app developer be recognised by the Vault on behalf of data storage to reward the developer?
