# Containers and their basic conventions

## Summary

This appendix to the New App Authentication describes how the general NFS and DNS will be working and handled in this new launcher-less approach. After talking about the general cases, this document will also outline how these will be used to manage **App Containers**, **Default Containers** (and which ones will be known to the system from the start), the **Root container** and how containers can be shared between apps.

## Containers

In its core, NFS always has been and will continue to be an emulation on top of generic network data types, which are formed following a specific convention: but while that was previously done with StructuredData, it will now use the new MutableData. Moreover, rather than having a hierarchy of StructuredData pointing to "subdirectories", we will flatten the structure into a single key-value-store mapping and emulate a file system like access on top of that.

We call these "containers" in correspondence with the same concept being used in cloud storage and service providers (some also call these "buckets"), and these emulations "conventions" (or "access convention"). They describe certain protocols that the client should adhere to when access and manipulating the data. Whenever a container is first shared with any third party (for example through the Authenticator URL-Scheme Protocol), the sharing party must indicate in its response any convention it knows about for said container. A container might adhere to many (non-conflicting) conventions at the same time.


### Encryption

Furthermore, containers might be fully encrypted with a key pair that is not available within the container but kept in a different location to prevent vaults from understanding the data itself. If such encryption is used, the same key-pair must be shared across all entities having access to it and each party must encrypt every key and every value with it before transmitting it to the network. We call this the "local key" within this document and the container to be "locally encrypted".

It is the sharing parties responsibility to share that key, too.

#### Nonces

To provide maximum security Nonce should be used when encrypting data. Therefore the entries of a _locally encrypted_ container will be the serialisation of Nonce and the encrypted value as follows:

`serialise(&(nonce, secretbox::seal(&plain_text, &nonce, key)))`

To provide the appropriate look up facility, an encrypted container shall encrypt the keys in a way that can be calculated by any party the access was shared with, while the value must be encrypted with a random Nonce every time. To facilitate this, the container definition may hold a Nonce. If that Nonce is present, the key can be calculated by suffixing that key with that Nonce, hashing the value with SHA256 and take the starting `secretbox::NONCEBYTES` as the Nonce for the calculation.


#### Following the convention

While the client is recommended to adhere to this convention, it will not be enforced by the underlying network, and we recognise that it might sometimes be purposeful to not follow the convention. However, that should be the exception rather than the rule, or a new convention should be specified and agreed upon for other to follow.

### Permissions

As containers are just normal MutableData on the network, thus having the same possible permissions as outlined in the corresponding RFC. At this point in time those are: `set`, `delete`, `insert`, and `managePermissions`. Additionally, as most data will be encrypted, an app can signal with the `read` keyword that requests the decryption key to be able to read a specific container. `read` is implied if any other permission is present.


Therefore a container is defined as:

```
pub enum ContainerConvention {
  LINKS,
  LISTING,
  NFS,
  APP
}

pub struct Container {
    /// Name of the data where the directory is stored.
    pub name: XorName,
    /// Type tag of the data where the directory is stored.
    pub type_tag: u64,
    /// Key to encrypt/decrypt the directory content.
    /// and the nonce to be used for keys
    pub enc_info: Option<(secretbox::Key, Option<secretbox::Nonce>)>,
    pub conventions: Vec<Convention>
}
```

The container is expected to provide helper functions to de- and encrypt entry keys and values.

## Links and App Convention

The simplest of the conventions (and the first listed) is the `LINKS`-Convention: a container that only holds a list of links to other containers. Specifically, its values should always be a serialised `Container`-definition as shown above. This container must not have any other values. It is referred to as a `LinksContainer`. It is used a lot internally as you will see below. 

The `App`-Convention specifies that this is app owned and may adhere to vendor specific rules alongside other conventions or exclusively. It is mainly meant to make clear that the container in question, may have a custom protocol you do not know about.

## NFS Convention

Unlike before, NFS convention-following containers are represented in a flat key-value fashion within one container rather than a hierarchy of linked "directories". Where the key is a UTF-8-String encoded full path filename, mapping to the serialised version of either another `Container` in the network or a serialised file struct like this:

```rust
pub struct File {
  metadata: FileMetadata,
  datamap : DataMap
}
```

The `Container` should point to another container following the same convention as its parents - so at least NFS - or to a serialised file struct as described before.

### Hierarchy File-System Emulation

Through splitting up the Unicode string on the slash (`/`) one can emulate a file system hierarchy. It is recommended that the emulation entity should first fetch all the keys and build the tree in memory to traverse it for performance reasons.

However, no party is obliged to organise its data this way, nor have file paths with slashes at all.

## Root Container

In the session packet of the user, we store the access for two locally encrypted containers: the `user_root` (later "root") and the `config_root` (later "config").

The root container is the main entry point for the user and most apps to interact with. It is a locally encrypted container stored at a random location on the network that only the authenticator has write access to. Keys starting with an underscore (`_`) are reserved for internal usage by the authenticator, while the authenticator may also allow the creation of other keys later. Each key is the name of the available container name,  and the value is a serialised vector of conventions that container follows. It is called the `RootContainer`.

### Config Container

Secondly, the config container is an authenticator `AppContainer`, which is used to store information and links which must never be shared directly outside of the authenticator.

In there, under the entry key `root_keys`, the authenticator links another locally encrypted `LinksContainer` which holds the full access information per each root container key listed. That is called the `RootKeysContainer` and the access to it must never be shared with any other party than the Authenticator.

At the `apps`-key in the config container, the authenticator stores app specific information, mapping the `${appid}/${scope}` to a serialised `AppInfo` as described in the 'Details' Appendix.

### Permissions

Any app may request `read` access to the `RootContainer` container using the defined authentication flow and ask for `_root`. With agreement from the user, the authenticator should share the location and the local key of the root containers. The authenticator should alert the user and require double confirmation should any app require anything other than the `read` access level directly. However, it may still be granted, if the user says so.

### Default containers

When creating a user account the authenticator will create the following minimal set of containers within the root container, each one with its own random network address:

* `_apps/net.maidsafe.authenticator/` : App Container
* `_documents` : NFS Container
* `_downloads` : NFS Container
* `_music` : NFS Container
* `_pictures` : NFS Container
* `_videos` : NFS Container
* `_public` : Links Container
* `_publicNames` : Links Container


## App Containers

Whenever an app asks for permissions to act in the user's name, it may ask for access to any container, and access to a container for the app itself. When this happens the first time, the authenticator creates a new locally encrypted `LinksContainer` of linking to the containers to be shared with the app - called the `AccessContainer` - and will share its address and decryption key through the authentication protocol. This container is owned by the user and the app will only have `read`-rights on it.

If the app further requested to have its own container, the authenticator must create a new random app-container, grant full access to the container to the app, generate a new random symmetric-key-pair and store all this access information in the app's `AccessContainer`. The authenticator must then link that address to the root container under `_apps/${appId}/@{scope}` and store the access information in the `RootAccessContainer`. We call this the `AppContainer`.

If the authenticator knows already of the same app without any scope, it should automatically be granted all rights on that `AppContainer`, too, by putting the access information into that app's `AccessContainer`.

The Authenticator must hold a copy of the app key pair to the app container in the `RootKeysContainer`, its `AccessContainer`, the encryption keys as well as the metadata that app asked to gain access with for later reference and to automatise the authentication process should the app ask again.

It is recommended that the app should encrypt all data it doesn't intend to publicly share with the encryption key it was given for its `AppContainer`.

Authenticator should only create the `AccessContainer` if permissions to any containers were requested and granted. The `AppContainer` must only be created if explicitly asked for it. See the Authenticator protocol for details.


## Sharing Containers

By asking for the `read` permissions to the root container (or by hard coding names), any app may request to gain access to _any container_ of the user. Specifically, the content shared between apps in the default containers, but also any other App. While some might have special restrictions and custom behaviour (see DNS later or the Root Container itself) to be followed, in general, any App is allowed to gain access with through the Authentication Protocol.

### Permissions

It is recommended that apps should only request the rights `read` and `insert` - which we will refer to as the `BASIC`-access. To promote the usage of this, APIs should provide a short-hand for this. Furthermore, the Authenticator may require a multi-step authorization from the user if it encounters an app that asks for more permissions than these and may allow the user to disable specific permissions asked for.

When a container is shared with the app, the Authenticator must add that appkey with permissions granted to the container. It must then store the access information in the apps `AccessContainer` .

### Revoking Permissions

When the user revokes access to a shared container, the apps key must be removed from the list of authorised keys. Furthermore, the user may instruct the Authenticator to re-encrypt the entire container. If requested, the authenticator should remove all current app keys from the container and re-encrypt the data with a new key.

The new key is then distributed in the `AccessContainers` of all apps that still have access to it.

## Public Containers: `_public` and `_publicNames` 

Aside from the App and NFS container, there are two special containers any app can request to: `public` and `publicNames`.

### `_public`

`_public` is a locally encrypted `LinksContainer` in which apps are encouraged to link their content made available publicly so other apps can see and potentially modify them. Thus containers linked in here are usually created by other apps. To gain higher level of access to those containers any app can request the usual permissions to those keys prefixed with `_public/`.


### `_publicNames`
`_publicNames` on the other hand is a `LinksContainer` to public names the user owns. Each entry pointing to a `ServiceContainer` that can (usually) be publicly accessed.

### Permissions

Any app may request to gain access to any of these two containers, in a similar fashion as on the `rootContainer`: by asking for `read`-level access on `_public` and `_publicNames`. Upon that request, the Authenticator must inform the user that this app is asking access to public names and if it should be given the right to do so.

To gain access to a specific entry in either of them, the app may request access using the `${conatinerName}/${entryKey}`, though that does not directly exist in the root directory. The Authenticator, when finding a request for a container starting with `_public[Names]/` must prompt the user about whether access should be given to that particular public name. Any request for more than the `BASIC` permissions as defined above, should be guarded through a double confirmation mode by the user.

### The Browsing Example

Now assuming that through the lookup, the browser/any app might find a container under the `www`-container, it may try to apply the NFS-style convention of browsing said folder. For performance reasons it is recommended it first perform a `keys` (if it has the rights to do so) to prevent lookups that would go nowhere anyways.


---

**Types Summary**

In this documentation, we have defined these public containers following these conventions

* `AppContainer` => `Map<*, *>`, locally encrypted
* `AccessContainer` => `Map<ContainerName, serialised(Container)>`
* `RootContainer` => `Map<ContainerName, serialised(Vec<Conventions>)>`, locally encrypted
* `NFSContainer` => `Map<fileName, serialised(Container || FileStruct)`, locally encrypted
* `LinksContainer` => `Map<*, serialised(Container)>`
* `ServicesContainer` => `Map<serviceName, *>`, not encrypted

And the following authenticator, internal containers:

* `RootKeysContainer` => `Map<ContainerName, serialised(Container)` holding all local container accesses
* `AppAccess` => `Map<${appId}/@{scope}, serialised(AppInfo)>`
