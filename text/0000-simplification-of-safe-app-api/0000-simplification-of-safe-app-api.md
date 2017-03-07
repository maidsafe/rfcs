# Simplification of the safe-app Library API

- Status: proposed
- Type: enhancement
- Related components: `safe_app`, `safe_app_nodejs`
- Start Date: 07-03-2017
- Discussion: https://forum.safedev.org/t/simplification-of-the-mutabledata-api/480 
- Supersedes:
- Superseded by:

## Summary

This RFC proposes a refactoring on the `safe-app` library and the provision of a much simpler to use API to access the SAFE network, exposing it as a Rust crate while at the same time sufficing the FFI functions required for the language bindings.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

A developer can implement a client application to interact with the SAFE network either by interfacing directly with the `safe_core` Rust crate, or by interfacing with the `safe_app` library (part of the `safe_client_libs`) which is intended to simplify the client development process.

Based on th assumption that the `safe_app` library's main purpose is to aid the development of SAFE applications, it shall then expose a much simpler and easier to use API so any developer with no backgorund/knowledge of how the SAFE network internally works can quickly start creating applications for the SAFE network. 

A developer shall only need to know that data can be stored in/shared thru the network with this API, that's why functions' names must to not only be picked very carefully but they also have to be abstracted as much as possible from SAFE network internal mechanisms, functioning, and vocabulary. 

Using the `MutableData` API as an example, at the moment a developer needs to handle several objects/entities for doing simple tasks like setting permissions or performing a single entry mutation, i.e. he/she needs to get references to the `Permissions`, `PermissionsSet`, `Entries` and/or `EntryMutationTransaction` objects for doing so. 

Currently, the following steps are required to create a `MutableData` with specific permissions:

1. Create a `MutableData` object
2. Create a new `PermissionsSet` object
3. Set each of the permissions in the `PermissionsSet` object (with `setAllow` and `setDeny` methods)
3. Create a new `Permissions` object
4. Insert the `PermissionsSet` object (from step 2) into the `Permissions` object
5. Set the `Permissions` object into the `MutableData` and send the request to the network

By simplifying the `safe_app` API, the above steps can be narrowed down to the ones below:

1. Create a `MutableData` object
2. Set each of the permissions by calling one of the following functions directly on the `MutableData` object: `setAllow(action, signKey)` and `setDeny(action, signKey)`

An analogous comparison can be made between the current way of applying mutations to the `MutableData`'s key-value entries.

This simplification could be done at the language bindings layer (e.g. in `safe_app_nodejs` package), but the abstractions mechanisms will need to be replicated in each of the bindings. Therefore implementing this simplification at the `safe_app` layer will save the efforts of creating the bindings for all different entities/objects but just for the simplified API functions.


## Detailed design

### Overview

Even that the `MutableData` entity will be used herein to detail the proposal, it's the intention of this proposal to be applicable in an analogous manner to the rest of functions/entites/objects exposed by the `safe_app` library.

The list below details the set of functions that the `safe_app` library shall expose for the `MutableData` as per this proposal.

The factory functions return a `MutableData` object handle, and all other functions expect just a `MutableData` object handle (apart from their corresponding input parameters), i.e. no need to provide object handles for the `Permissions`, `PermissionsSet`, etc..

**Factory functions:**

- `createPubMutableData(name, typeTag, entries)` / `createPrivMutableData(name, typeTag, entries)`
This function instantiates a `MutableData` object, sets full access permissions for the application by default, and it populates its entries.

- `createPubMutableData(name, typeTag)` / `createPrivMutableData(name, typeTag)`
This function instantiates a `MutableData` object, sets full access permissions for the application by default, with no entries.

- `createPubMutableData(name, typeTag, permissions)` / `createPrivMutableData(name, typeTag, permissions)`
This function instantiates a `MutableData` object, sets the specific access permissions for the application, with no entries.


**MutableData Info:**

- `getVersion()`

- `getNameAndTag()`

**Entries functions:**

- `insert(key, value)`

- `remove(key)`

- `update(key, value)`

- `applyMutations(mutationsObj)`
It applies a bulk of mutations in a single trasaction.

- `get(key) -> (value, version)`
It returns the latest version.

- `get(key, version) -> (value, version)`

- `getEntriesLength()`

- `forEachEntry( function(key, value, version)=>{...} )`

- `forEachKey( function(key)=>{...} )`

- `forEachValue( function(value, version)=>{...} )`

**Permissions functions:**

- `getPermissionsLength()`

- `forEachAllowPermission( function(action, signKeys)=>{...} )`

- `forEachDenyPermission( function(action, signKeys)=>{...} )`

- `setAllowPermission(action, Anyone | signKey)`

- `setDenyPermission(action, Anyone | signKey)`

- `clearPermission(action, Anyone | signKey)`

- `removeUserPermissions(Anyone | signKey)`

- `setUserPermissions(permissionsObj, Anyone | signKey)`


### Implementation Details


The following "flavoured" pseudo-code details the implementation by showing how a couple of `MutableData` related functions shall be implemented at each of the different layers of the software stack. It's followed by a diagram of how the software architecture would look like with the proposed refactoring.

Note that the proposed API is also exposed as a Rust crate, at the same time as sufficing the FFI required for the language bindings.

---

**A node.js application**

`safeapp.mutableData` is of type `MutableDataInterface` from the node.js binding which exposes just the static factory functions.

```node
safeapp.mutableData.createPubMutableData("mypublicId", 15001)
  .then((md) => md.insert("key1", "value1"))
  .then(() => {
    console.log("I created a MutableData and inserted entry!");
    console.log("Everything was already sent to the network.");
  })
);
```

---

**node.js binding**

```rust
const lib = ffi.Library("libsafe_app.so");

class MutableDataInterface {

  createPubMutableData(name, typeTag) {  
    return lib.mdata_info_new_public(name, typeTag)
        .then((md_handle) => new MutableData(md_handle) );
  }
  ...
}

class MutableData {
  // this.md_handle holds the MutableDataHandle
  // passed at construction time.

  insert(key, value) {
    return lib.mdata_insert_entry(this.md_handle, key, value);
  }
  ...
}
```

---

**safe_app FFI**

`MutableData` is a struct in the `safe_app` crate.
`MutableDataHandle` is a type of cache handle to hold the `MutableData` info in the memory cache.

```rust
pub type MutableDataHandle = ObjectHandle;

pub unsafe extern "C" fn mdata_info_new_public(name, type_tag,
                                callback: extern "C" fn(MutableDataHandle) )
{
    // Get a new MutableData object from the safe_app crate
    let md = safe_app::MutableData::new_public( name, type_tag );

    // Store the MutableData in cache
    let md_handle = object_cache.store( md );

    // Return the MutableDataHandle thru the callback function
    callback( md_handle );
};

pub unsafe extern "C" fn mdata_insert_entry(md_handle, key, value)
{
    // Fetch the MutableData from cache
    let md = object_cache.get( md_handle );

    // Insert the key-value. This invokes safe_app::MutableData.insert_entry method
    md.insert_entry(key, value);
}
```

---

**safe_app Rust crate**

It is hereby proposed to expose the same API also in the Rust layer so Rust apps can be linked directly with this crate rather than being forced either use the FFI or to  interface with the `safe_core` crate.

```rust
pub struct MutableData {
    info: safe_core::MDataInfo,
}

impl MutableData {
    pub fn new_public(name, type_tag) -> MutableData {
        // Send MutableData creation request to the network
        let md_info = safe_core::MDataInfo::new_public( XorName(name), type_tag );

        // Return an instance of MutableData struct
        MutableData { info: md_info }
    }

    pub fn insert_entry(&self, key, value) {
      // Create mutation transaction
      let mut actions = Default::default();
      actions.insert('insert', key, value);

      // Send it to the network
      safe_core::client.mutate_mdata_entries(self.info.name,
                                             self.info.type_tag, actions);
    }
}
```

Note that the rest of the `MutableData` information like `Permissions`, `PermissionsSet`, `Entries`, etc., shall be kept within the scope of the methods that need them instead of keeping them all in the FFI's memory cache. This will significantly reduce the amount of objects that need to be kept in the `safe_app` FFI layer.
E.g. a `foreach` implementation shall be as follows:
```rust
    pub fn foreach_entry(&self, foreach_callback) {
        // Fetch the entries from the network and keep them in scope
        let entries = safe_core::client.list_mdata_entries(self.info.name,
                                                           self.info.type_tag)
        for entry in entries {
            foreach_callback(entry.key, entry.value);
        }
    }
```

### Proposed Software Architecture

![safe-app Software Architecture](proposed-safe-app-architecture.png)

## Drawbacks

Why should we *not* do this?

## Alternatives

What other designs have been considered? What is the impact of not doing this?


## Unresolved questions

What parts of the design are still to be done?
