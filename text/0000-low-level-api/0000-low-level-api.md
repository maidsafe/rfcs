# Low Level API
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_launcher`
- Start Date: 29-August-2016

## Summary
Exposing low level API to facilitate direct construction of MaidSafe types by Launcher and apps.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
As more and more types become exposed via `safe_core` it would be impractical to provide helper functions geared towards all the use cases of various apps. It would simply lead to expolosion of such functions in the public interface. Instead if `safe_core` could provide direct interfacing with the API's somehow, then the apps/Launcher would simply call what they need and permutations would lie with them as it should be. We already have helper functions to do basic composite dns and nfs operations (like moving a file from one nfs-directory to another etc.). This RFC will deal with exposing the internals of MaidSafe types so that different or more complex operations can be done by Launcher/apps. Also representation of data as nfs-exposed directory hirearchy might not be suitable or applicable for many apps. The low level exposure would allow them to construct their own data representation of choice.

## Detailed design
Currently one of the functionality of `Launcher` is to provide sandboxing. Apps which pass through the Launcher (which is the recommended approach because it is considered bad to give one's credentials to every app) have access to data either within specific folder created for them or within `SAFEDrive` which is where common data is. No app is allowed to access data in a folder reserved for another app. However this guarantee will be broken once the low level API's are exposed because apps will have freedom to create whatever data they want and wherever they want it on the network. Under the current implementation this would mean that private data stored by one app can be potentially compromised (accessed by another app). For e.g. say App-0 creates and stores `StructuredData` `abc` somewhere in the network. If App-1 uses a direct `GET` for `abc` there is no way Launcher knows this should not be allowed. Previously apps were only allowed to travel a directory hirarchy to get data and Launcher could assert it travelled only the permissible ones.

To get around this limitation, Launcher shall enforce a rule of separate `box_::gen_keypair()` for each app. Every app that registers successfully with Launcher gets a new `box_` key-pair. All private data created on the network by the app will use this key-pair to encrypt/decrypt data. These keys will need to be persistant, so Launcher will write the details in its configuration file against the registered app.

Though the keys are persistant, there is no way for Launcher to know if one app is mimicking another during the authentication process. So Launcher will ask user each time an app starts and tries to register with Launcher, _even if Launcher was never killed in the meantime_ (unlike currently).

## Alternatives

### Api's for StructuredData manipulation:
```rust
/// _type_tag_:
///   - 500 for unversioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - The actual data representation is implementation defined.
///   - 501 for versioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - `safe_core` will ensure that data is versioned.
///     - The actual data representation is implementation defined.
///   - above 15000 for user defined (no modifications from safe_core)
///   - anything else is an error
/// _id_: Pointer to array of size 32
/// _is_encrypted_: If data needs to be encrypted.
/// _data_: Actual data to be stored in StructuredData.
/// _size_: Size of actual data to be stored in StructuredData.
/// _sd_: If successful StructuredData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_create(type_tag: u64,
                                            id: *const [u8; 32],
                                            is_encrypted: bool,
                                            data: *const u8,
                                            size: *const u64,
                                            sd: *mut *mut StructuredData) -> i32;


/// _type_tag_: type-tag of StructuredData to be fetched.
/// _id_: Pointer to array of size 32.
/// _sd_: If successful StructuredData will be given via this handle
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get(type_tag: u64,
                                         id: *const [u8; 32],
                                         sd: *mut *mut StructuredData) -> i32;


/// _data_id_: DataIdentifier for the StructuredData to be fetched.
/// _sd_: If successful StructuredData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_with_data_id(data_id: *const DataIdentifier,
                                                      sd: *mut *mut StructuredData) -> i32;


/// _type_tag_: type-tag of StructuredData.
/// _id_: Pointer to array of size 32.
/// _data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_construct_data_id(type_tag: u64,
                                                       id: *const [u8; 32],
                                                       data_id: *mut *mut DataIdentifier) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 500 for unversioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - Previous data will be truncated & overwritten, hence lost.
///     - The actual data representation is implementation defined.
///   - 501 for versioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - `safe_core` will ensure that data is versioned.
///     - New data will be added as most recent version. Previous data will not be lost.
///     - The actual data representation is implementation defined.
///   - above 15000 for user defined (no modifications from safe_core)
///     - Previous data will be truncated & overwritten, hence lost.
///   - anything else is an error
///   - StructuredData will be modified hence a mutable pointee.
/// _is_encrypted_: If data needs to be encrypted.
/// _data_: New data.
/// _size_: Size of new data.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_new_data(sd: *mut StructuredData,
                                              is_encrypted: bool,
                                              data: *const u8,
                                              size: *const u64) -> i32;


/// _sd_: A valid StructuredData handle.
/// _data_id_: DataIdentifier of StructuredData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_data_id(sd: *mut StructuredData,
                                                 data_id: *mut *mut DataIdentifier) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 500 for unversioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit.
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit and versioned. Data pertaining to the most recent version will be
///        returned.
///   - above 15000 for user defined; there will be no modifications from safe_core - data is
///     returned as is.
///   - anything else is an error
/// _is_encrypted_: If data was encrypted (hence needs to be decrypted).
/// _data_: If successful the actual data will be given via this handle.
/// _size_: If successful the actual data size will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_data(sd: *const StructuredData,
                                              is_encrypted: bool,
                                              data: *mut *mut u8,
                                              size: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit. Data pertaining to the most recent version will be returned.
///   - anything else is an error
/// _versions_: If successful the number of available versions will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_num_of_versions(sd: *const StructuredData,
                                                         versions: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit and versioned.
///   - anything else is an error
/// _n_: Data pertaining to the _nth_ version will be returned.
/// _is_encrypted_: If data was encrypted (hence needs to be decrypted).
/// _data_: If successful the actual data will be given via this handle.
/// _size_: If successful the actual data size will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_nth_version(sd: *const StructuredData,
                                                     n: u64,
                                                     is_encrypted: bool,
                                                     data: *mut *mut u8,
                                                     size: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_put(sd: *const StructuredData) -> i32;


/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_post(sd: *mut StructuredData) -> i32;

/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       StructuredData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_delete(sd: *mut StructuredData) -> i32;


/// _sd_: A valid StructuredData handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_free_handle(sd: *mut StructuredData) -> i32;
```

### Api's for AppendableData manipulation:
```rust
/// FFI Handle for a union of AppendableData types to reduce combinatorial explosion of functions.
pub enum AppendableDataHandle {
    Pub(PubAppendableData),
    Priv(PrivAppendableData),
}

#[rep(C)]
pub enum FilterType {
    BlackList,
    WhiteList,
}

/// _name_: Pointer to array of size 32.
/// _filter_type_: The filter criteria to be applied to supplied keys. If it's
///                `FilterType::WhiteList` and `filtered_keys` is null then nothing but owner's
///                changes will pass the filter check.
/// _filtered_keys_: Initial set of keys to apply filter to. Should be null terminated.
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_create(name: *const [u8; 32],
                                                    filter_type: FilterType,
                                                    filtered_keys: *const *const [u8; sign::PUBLICKEYBYTES],
                                                    data: *const AppendableData,
                                                    ad: *mut *mut AppendableDataHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _filter_type_: The filter criteria to be applied to supplied keys. If it's
///                `FilterType::WhiteList` and `filtered_keys` is null then nothing but owner's
///                changes will pass the filter check.
/// _filtered_keys_: Initial set of keys to apply filter to. Should be null terminated.
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_create(name: *const [u8; 32],
                                                     filter_type: FilterType,
                                                     filtered_keys: *const *const [u8; sign::PUBLICKEYBYTES],
                                                     data: *const AppendableData,
                                                     ad: *mut *mut AppendableDataHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _is_private_: If Pub/PrivAppendableData.
/// _data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_construct_data_id(name: *const [u8; 32],
                                                           is_private: bool,
                                                           data_id: *mut *mut DataIdentifier)
                                                           -> i32;


/// _name_: Pointer to array of size 32.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_get(name: *const [u8; 32],
                                                 ad: *mut *mut AppendableDataHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_get(name: *const [u8; 32],
                                                  ad: *mut *mut AppendableDataHandle) -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_add_filter_key(ad: *mut AppendableDataHandle,
                                                        filter_key: *const [u8; sign::PUBLICKEYBYTES)
                                                        -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _num_: Number of AppendedData will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_list_size(ad: *const AppendableDataHandle,
                                                       num: *mut u64) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: DataIdentifier pertaining to the _nth_ AppendedData will be returned.
/// _data_id_: DataIdentifier of actual data pointed to by this AppendedData will be put into this.
///            Decryption will be performed internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_id(ad: *const AppendableDataHandle,
                                                         n: u64,
                                                         data_id: *mut *mut DataIdentifier)
                                                         -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: Sign key pertaining to the _nth_ AppendedData will be returned.
/// _data_id_: Sign key of this AppendedData will be put into this. Decryption will be performed
///            internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_sign_key(ad: *const AppendableDataHandle,
                                                               n: u64,
                                                               sign_key: *mut *mut [u8; sign::PUBLICKEYBYTES])
                                                               -> i32;


/// _ad_: Handle to a valid AppendableData. Will be modified, hence a mutable pointee.
/// _n_: nth AppendedData will be removed. It will be moved into the `deleted_data` field.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_delete_nth_data(ad: *mut AppendableDataHandle,
                                                         n: u64) -> i32;


/// _append_to_: Handle to a valid Pub/PrivAppendableData to which we are to append.
///                - Encryption with throw-away key etc. will all be deduced and done internally.
/// _pointer_: Handle to a valid DataIdentifier (pointing to actual data) to append.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_append(append_to: *const DataIdentifier,
                                                pointer: *const DataIdentifier) -> i32;


/// _ad_: A valid AppendableData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_put(ad: *const AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_post(ad: *mut AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       AppendableData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_delete(ad: *mut AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_free_handle(ad: *mut AppendableDataHandle) -> i32;
```

### Api's for ImmutableData manipulation:
```rust
/// _se_: New Self-Encryptor will be written to this handle.
#[no_mangle]
pub unsafe extern "C" fn immut_data_new_self_encryptor(se: *mut *mut SequentialEncryptor) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _data_: Raw data to be written.
/// _size_: Size of the raw data to be written.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_write_to_self_encryptor(se: *mut SequentialEncryptor,
                                                            data: *const u8,
                                                            size: u64) -> i32;


/// _se_: Valid Self-Encryptor handle. The Self-Encryptor will be destroyed after this and must not
///       be used any further. It is UB to use `se` any further.
/// _is_encrypted_: If the data map should be encrypted.
/// _data_id_: DataIdentifier of final ImmutableData will be put into this.
///              - After self-encryption, the obtained (encrypted or otherwise) data-map will be
///                tested to be <= 1 MiB. If not it will undergo self-encryption again and the
///                process repeats till we get a data-map which is <= 1 MiB. This data-map will be
///                `PUT` to the network as ImmutableData and its DataIdentifier returned.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_close_self_encryptor(se: *mut SequentialEncryptor,
                                                         is_encrypted: bool,
                                                         data_id: *mut *mut DataIdentifier) -> i32;


/// _data_id_: Valid DataIdentifier.
/// _se_: Self-Encryptor created from the data-map extracted via reverse process of creation as
///       detailed above. It is seamless to the app, all work being done by `safe_core`.
#[no_mangle]
pub unsafe extern "C" fn immut_data_get_self_encryptor(data_id: *const DataIdentifier,
                                                       se: *mut *mut SequentialEncryptor) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _size_: Size of total available data will be written to this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_self_encryptor_size(se: *mut SequentialEncryptor,
                                                        size: *mut u64) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _offset_: Offset to read from.
/// _size_: Number of bytes to read from, starting from offset.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_read_self_encryptor(se: *mut SequentialEncryptor,
                                                        offset: u64,
                                                        size: u64,
                                                        data: *mut *mut u8) -> i32;


/// _ad_: A valid Self-Encryptor handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_free_self_encryptor_handle(se: *mut SequentialEncryptor) -> i32;
```

## Drawbacks
- If there are many apps and there are configured to work at system start-up then the user will face a barrage of pop-ups to confirm if an app should have access. This undermines the security due to inconveniencing the user.

## Unresolved Problems
- While other apps will not be able to read the private data created by an app, they can still modify (e.g. overwrite) it, for instance if it were a `StructuredData`. To prevent this not only each app should be given its own `box_` key-pair but also `sign` key-pair. However the problem with that is only one `sign` key-pair is registered in `MaidManagers` for an account. Any other key-pair will be disallowed to create data on the network.
- The required authentication each time app wants to resume connection with Launcher means that Launcher must detect the app having gone offline. One way to do this could be a persistant connection with heartbeats but it is yet to be seen if this is the most ideal approach.

## Alternatives
- Instead of passing pointers to Launcher, use a different mechanism which keeps the pointers internally and passes handle (e.g. u64) to Launcher and maps handles to pointers internally in `safe_core`.
