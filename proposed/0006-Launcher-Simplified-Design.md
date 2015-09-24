- Feature Name: SAFE-Launcher
- Type: New Product
- Related components: [safe_client](https://github.com/maidsafe/safe_client), [safe_nfs](https://github.com/maidsafe/safe_nfs), [safe_vault](https://github.com/maidsafe/safe_vault)
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

<3> will manage metadata related to Apps to give uniformity in experience when shifting from one machine to another - eg., if App `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE-Account via Launcher, he/she will be presented with a union/list of all the Apps that were installed on all the machines which access the SAFE-Network on his/her behalf.

<4> along with [safe_vault](https://github.com/maidsafe/safe_vault) will manage the mapping and de-mapping of ownership keys for an App (if the App requires to mutate the network on the user's behalf)

## Expected outcome

SAFE-Launcher

<1> will allow user to create an account and/or log into the SAFE-Network.

<2> will authenticate a user installed App to access SAFE-Network on the user's behalf.

<3> will manage metadata related to Apps to give uniformity in experience when shifting from one machine to another - eg., if App `A` is installed in machine 1 then when the user logs into machine 2 using his/her SAFE-Account via Launcher, he/she will be presented with a union/list of all the Apps that were installed on all the machines which access the SAFE-Network on his/her behalf.

<4> along with [safe_vault](https://github.com/maidsafe/safe_vault) will manage the mapping and de-mapping of ownership keys for an App (if the App requires to mutate the network on the user's behalf)

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

## Start Flow

**step 0:** Start Launcher.

**step 1:** Enter Credentials to either create an account or to log into a previously created one.

**step 2:** If it was a log in, Launcher Fetches and decodes User-Session-Packet (USP).

**step 3:** Launcher Fetches `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>` (See Session Packet description) - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 4:** Launcher Reads the special Directory reserved for it - `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory`; (See Session Packet Description) - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 5:** Launcher Fetches `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>` - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate)

**step 6:** Launcher Checks for special Directory named `SAFEDrive` inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/SAFEDrive` - if not present Launcher will Create it (via [safe_nfs](https://github.com/maidsafe/safe_nfs) crate). This is the directory that the user will usually see it mounted as root.

## Add App Flow

**step 0:** User Drags `XYZ` App binary into the Launcher to add it.

**step 1:** Launcher Creates (if it’s the first time it saw this App):
- Unique Random 64 Byte Id for this app - App-ID (because names of different binaries can be same if they are in different locations on the same machine - thus we need a unique identifier for the binary)
- Unique Directory Id and an associated Unique Root Directory Listing `<APP-ROOT-DIR>` for this app - `XYZ-Root-Dir`. For directory name conflicts append numbers so that they can exist on a file system - eg., `XYZ-1-Root-Dir`. This shall be created inside `<USER’S-PRIVATE-ROOT-DIRECTORY-ID>/`.
- `<APP-ROOT-DIR>` shall be always **unversioned** and **private** (ie., encrypted with Launcher-keys). Any requirement for a versioned or public directory from the App can be managed by the App itself by creating further subdirectories.
- Append this information in `<LAUNCHER-CONFIG-FILE>` = `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/LauncherReservedDirectory/LauncherConfigurationFile` (This is what the DNS crate currently does too inside `<MAIDSAFE-SPECIFIC-CONFIG-ROOT>/DnsReservedDirectory/DnsConfigurationFile`.) - Format shall be CBOR (compact-binary-object-representation).
```
Launcher-keys: (random-encryption-key-pair, random-ownership-key-pair),
[
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR>
    },
    {
        App Name,
        Reference Count,
        Random Unique App ID,    // 64 Bytes
        Vec<SHA512(App-Binary)>,
        <APP-ROOT-DIR>
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

**./path/to/XYZ --launcher “port:33000;protocol:udp;nonce:random-u32-number”**

**step 4:** Launcher will wait for a predefined time of **15** seconds for data reception on that port. If it times out it will close the socket (release its binding to it)

**step 5:** App responds on the socket asking for Launcher to give keys and its root directory, which Launcher had reserved as `XYZ-Root-Dir`
- The payload format for this request shall be CBOR encoded sturcture of the following:
```
struct Request {
    prefix: u8,  // This must be ASCII of '?' (a question mark)
    nonce : u32, // This must be the nonce supplied by Launcher
}
```

**step 6:** Launcher reads the `<LAUNCHER-CONFIG-FILE>` and creates a mapping in the MaidManagers for associating user’s `MAID Keys <-> Launcher-keys` to allow App to PUT/POST/DELETE on behalf of the user in `<APP-ROOT-DIR>`.
- The mapping is done only for the ownership key/s (not encryption keys) and is done only when the first App is added.
- The request for mapping/un-mapping shall be a command to the `MaidManagers`. For commands to `MaidManagers`, `StructuredData` with special Type-Tag will be reserved. On reception of this `StructuredData` via a `POST` message explicity directed towards MaidManagers the vaults will check the payload and act upon the request instead of executing a normal reaction to the usual `POST`s.
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

**step 7:** Launcher gives the App the **Pseudo-MAID Keys** and **App-Dir** to the App and at this point it closes its socket and is no longer associated with the App in anyway.
- The payload format for this response shall be CBOR encoded sturcture of the following, followed by hybrid_encrypt of the stream using App-specific-crypto-keys:
```
struct Response {
    root_directory_id      : NameType,
    root_directory_tag     : u64
    // The Following are the user Launcher-keys
    public_signing_key     : sodiumoxide::crypto::sign::PublicKey,
    private_signing_key    : sodiumoxide::crypto::sign::SecretKey,
    public_encrytion_key   : sodiumoxide::crypto::box_::PublicKey,
    private_encrytion_key  : sodiumoxide::crypto::box_::SecretKey,
}
```

**step 8:** App does what it wants this point onwards.

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
- If the `Reference Count` is **0** it means that this is the last machine where the App was present. The Launcher shall not delete `<APP-ROOT-DIR>` without user-consent. It is user's responsibility to do that as it might contain information (like pictures etc) which the user may not want to lose. Instead the Launcher will ask if the user wants to do that and act accordingly.

**procedure 1:** While the other procedure would work, but there might be occassions when the user wants to immediately remove the app completely. He may not have access to other machines where the App was installed and maybe currently running and the previous procedure requires him to remove it from all machines to actually perform un-mapping of keys in `Vaults`. Thus there shall be an option in Launcher to remove App completely irrespective of if it is installed and/or running in other machines. In such cases Launcher will reduce `Reference Count` to 0 and proceed as above for the detection of zero reference count.

In both cases above if the App-count is reduced to zero, the Launcher shall instruct the vaults to un-map the ownership keys using the same scheme to communicate this which was used for mapping earlier.

## Misc
- If the App is added to Launcher in one machine, the mention of this will go into `<LAUNCHER-CONFIG-FILE>` as stated previously. It will thus be listed on every machine when user logs into his/her account via Launcher on that machine. However when the App is attempted to be activated on a machine via Launcher where it was not previously added to Launcher then he/she will be prompted to associate a binary. Once done, the information as usual will go into the `<LOCAL-CONFIG-FILE>` on that machine and the user won't be prompted the next time.
- When the App is started via Launcher, it will first check if the `SHA512(App-Binary)` matches any one of the corresponding entries in the vector in `<LAUNCHER-CONFIG-FILE>`. If it does not Launcher will interpret this as a malacious App that has replaced the App-binary on user's machine, and thus will show a dialog to the user for confirmation  of whether to still continue, because there can be genuine reasons for binary not matching - like the App was updated etc.

# Alternatives
None yet.

# Unresolved questions

## Drawbacks
Being a Simplified design, this approach has draw-backs:
- Same Pseudo-MAID Keys are given to the App so the Apps can tamper each other's configurations.
- Revocation is unanswered. Process of retieving all data from the network, reassigning encryption and ownership keys and and again putting it all into the network might prove to be too combersome and practical feasibility of that needs to be evaluated. Until then, App continues to have access even after removal from Launcher.
