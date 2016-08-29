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
### Api's for `StructuredData` manipulation:
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
pub unsafe extern "C" fn struct_data_delete(sd: *const StructuredData) -> i32;
```

### Api's for `AppendableData` manipulation:
```rust
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
                                                    ad: *mut *mut PubAppendableData) -> i32;


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
                                                     ad: *mut *mut PrivAppendableData) -> i32;


/// _name_: Pointer to array of size 32.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_get(name: *const [u8; 32],
                                                 ad: *mut *mut PubAppendableData) -> i32;


/// _name_: Pointer to array of size 32.
/// _ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_get(name: *const [u8; 32],
                                                  ad: *mut *mut PrivAppendableData) -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_add_filter_key(ad: *mut PubAppendableData,
                                                            filter_key: *const [u8; sign::PUBLICKEYBYTES)
                                                            -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_add_filter_key(ad: *mut PrivAppendableData,
                                                             filter_key: *const [u8; sign::PUBLICKEYBYTES)
                                                             -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _num_: Number of AppendedData will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_get_list_size(ad: *const PubAppendableData,
                                                           num: *mut u64) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _num_: Number of AppendedData will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_get_list_size(ad: *const PrivAppendableData,
                                                            num: *mut u64) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: DataIdentifier pertaining to the _nth_ AppendedData will be returned.
/// _data_id_: DataIdentifier of actual data pointed to by this AppendedData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_get_nth_data_id(ad: *const PubAppendableData,
                                                             n: u64,
                                                             data_id: *mut *mut DataIdentifier)
                                                             -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: DataIdentifier pertaining to the _nth_ AppendedData will be returned. Decrypting will be
///      performed internally by the API.
/// _data_id_: DataIdentifier of actual data pointed to by this AppendedData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_get_nth_data_id(ad: *const PrivAppendableData,
                                                              n: u64,
                                                              data_id: *mut *mut DataIdentifier)
                                                              -> i32;


/// _append_to_: Handle to a valid Pub/PrivAppendableData to which we are to append.
///                - Encryption with throw-away key etc. will all be deduced and done internally.
/// _pointer_: Handle to a valid DataIdentifier (pointing to actual data) to append.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_append(append_to: *const DataIdentifier,
                                                
                                                pointer: *const DataIdentifier) -> i32;
```

### Api's for `ImmutableData` manipulation:
```rust
#[no_mangle]
pub unsafe extern "C" fn immut_data_new_self_encryptor(se: *mut *mut SequentialEncryptor) -> i32;


#[no_mangle]
pub unsafe extern "C" fn immut_data_write_to_self_encryptor(se: *mut SequentialEncryptor,
                                                            data: *const u8,
                                                            size: u64) -> i32;


#[no_mangle]
pub unsafe extern "C" fn immut_data_close_self_encryptor(se: *mut SequentialEncryptor,
                                                         is_encrypted: bool,
                                                         data_id: *mut *mut DataIdentifier) -> i32;


#[no_mangle]
pub unsafe extern "C" fn immut_data_get_self_encryptor(data_id: *const DataIdentifier,
                                                       se: *mut *mut SequentialEncryptor) -> i32;


#[no_mangle]
pub unsafe extern "C" fn immut_data_read_self_encryptor(se: *mut SequentialEncryptor,
                                                        offset: u64,
                                                        size: u64,
                                                        data: *mut *mut u8) -> i32;
```
## Drawbacks
## Alternatives
