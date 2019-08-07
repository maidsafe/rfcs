# RDF for the Public Name Resolution System

- Status: proposed
- Type: enhancement
- Related components: Safe Browser. Safe App Nodejs / Rust CLI / API
- Start Date: 23/09/2018
- Discussion: https://github.com/maidsafe/rfcs/issues/283
- Supersedes: -
- Superseded by: -

## Summary

This proposal looks to enhance the public name resolution system by using a resource description framework. It also brings the RFC into line with the resolution system in the [SAFE CLI](https://github.com/maidsafe/safe-cli), which API should form the basis of all future resolvers.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- XOR-URL refers to a url generated as part of the content addressable system for accessing `xorname` urls in the safe_browser. As described here: https://forum.safedev.org/t/xor-address-urls-xor-urls/1952
- Data is presented as RDF, serialised in [JSON-LD](https://json-ld.org/) in the examples.
- I'm using MutableData ( `AOD` ) and ImmutableData ( `ID` ) as shorthands.


## Motivation

The aim here is to use [RDF data](https://en.wikipedia.org/wiki/Resource_Description_Framework) for our application data and the SAFE Name Resolution System. This enables any application encountering the data to know it's purpose, and handle it accordingly.

Equally, we have no formal description of the Public Name Resolution System (hereafter: NRS) resolution at this time, so this will codify that too.

It also seeks to propose new terminology to clarify some terms and clashes with traditional nomenclature (offering an alternative to the clearnet domain name system).

This RFC proposes that url schemes such as `safe://somewhere` can have sub-domain's with resolvable services which can be either chosen by the end user application, or default to specific data.

## Detailed design

### Nomenclature

-  `Name Resolution System` (NRS) instead of `DNS` to avoid confusion and clarify this is a SAFE network term, and that it relates to `Public Names`.
	- Here the URL terminology for `host` is equivalent to a SAFE `Public Name`.
	- What is called `subdomains` on the clearnet are referred to as `Sub Names`.
- Introduce `Resolvable Map` schema to describe RDF data on the network that can resolve a given `key` to a XOR-URL. This is described below and can be used by:
	- `Public Name` AODs.
	- A `Files Container` AOD: an alternative to the NFS style container, with similar functionality but described using RDF (and the `Resolvable Map` schema)
	
### Versioned Data / The Perpetual Web 

NRS data on the network, is by its very nature, public. Which means that it cannot be deleted. This concept forms the basis of the Perpetual Web, where data can only be appended to (using AppendOnlyData). As a consequence RDF data described here is a type of 'versioned' data, with the `key` of an AOD being a `UTC timestamp`, and the `value` of an entry being a string representation of the RDF data. This means that _all versions of the data_ can be accessed (via `?v=1` url params, [see xorurl rfc for more info](https://forum.safedev.org/t/xor-address-urls-xor-urls/1952), as well as the order of this data, and the timestamp changes were made. (Although these timestamps are applied client side, and therefore are optional so should not be treated with any reverence.)

### Reference Implementation 

The [SAFE-CLI fetch API](https://github.com/maidsafe/safe-cli/blob/master/src/api/fetch.rs) can be considered the reference implementation. Although, _at this point_ it is implementing pseudo RDF as the SAFE RDF APIs are not yet finalised. This implementation does follow the same structure and nesting as this proposal, just without a fully fledged RDF document. Logic should remain the same however.

### URL resolution

Before diving into data structures, I wanted to described how URL resolution will work in a browser (such as the safe_browser).

#### 1. XOR-URLs.

Any resolver should first attempt to parse a URL for being a valid [XOR-URLs](https://forum.safedev.org/t/xor-address-urls-xor-urls/1952).

If so, it is resolved via XOR-URL. 

- If pointing to a ResolvableMap AOD, resolution continues thereafter as described in the `Resolvable Map` or `Files Container` as appropriate (if the url has a `path` or `url fragments` eg.).
- If another data type is found, say a `FilesContainer` or `ImmutableData`, that data is retrieved (see point 3 below).


#### 2. PublicNameSystem.

Failing to be detected as a XOR-URL, we then parse the url and use the Public Name System to resolve for data.

Here the URL terminology for `host` is equivalent to a SAFE `Public Name`. What are known as `subdomains` on the clearnet are referred to as `SubNames`.

`safe://<subName>.<publicName>`

- GET the Mutable Data for a given `Public Name`.
- Parse the retrieved `Resolvable Map`.
- Resolve the `Sub Name` graph from this `Resolvable Map`.

Unavailability of any data being dereferenced will throw an error.

##### 2.1 No `SubName` aka Default Services.

- GET the Mutable Data for a given `Public Name`.
- Parse the retrieved `Resolvable Map`.
- Resolve the `default` graph / XOR-URL if available.

Unavailability of any data being dereferenced will throw an error.


##### 2.2 Many `SubName`s

`safe://<subName>.<subName>.<subName>.<subName>.<publicName>`

- As above, resolving each additional substring, up to a defined maximum of redirects (implemented in the resolver.)
	- Safe Browser will implement redirect limit of 10 redirects per url resolution. Any more than this would throw an error.

Unavailability of any data being dereferenced will throw an error.


#### 3. `FilesMap` and `Path` resolution

`safe://<subName>.<publicName><path>`, eg `safe://pns.rfc/resolution`

Once the final data has been resolved, if a `Files Container` type of `Resolvable Map` has been located, then the trailing url path would be resolved from that `FilesContainer`, too.


### Data Structures fo Resolution

![Image of PNS Resolution Data Structures](https://raw.githubusercontent.com/joshuef/rfcs/PnsAndResolveableMap/text/0000-RDF-for-public-name-resolution/PNS_data_representation.png)

#### Resolvable Map Structure

The idea for this is an RDF Data Set stored on the safe network. This will follow a newly defined schema, that represents a list of `keys`, which map to XOR-URLs. Each entry can contain more information to aid in resolving data, depending on context / application.

Sample RDF schema can be found: https://github.com/joshuef/sschema/tree/master/src

The RDF document should also contain a `default` entry, which can either point to a SAFE URL (xor or pubName) or alternatively can point to another graph (such as another `Resolvable Map`). The resolver will determine that it is URL to resolve via the presence of `safe://` protocol.

The RDF document will have a version relating to the version of the `Resolvable Map` data structure in use. (starting at `v1`.)

Provides data to be shown at the public name.
 - It must be an RDF data object
 `<safe/ResolvableMap>`, `Sub Name` graphs will pointing to a SAFE Url for data location (could be xor or using a subName).
 - Extra data can be added to the graph for each entry to aid in service discovery for the key.
 - `@id` entries _must_ point to a versioned XOR-URL for consistency (while pubNames may change, _this_ data will not move location);


 For `safe://<subName>.<myPublicName>`


 With the following, `safe://happyurl` would resolve to the `name` graph's entry.
 ```js
 [
	{
		"@context": {
		    "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		  },
	     "@type": "ResolvableMap",
		 "@id": "safe://thisxorurl",
		 "default" : "safe://thisxorurl#somewhere"
	},
	{
		"@context": {
		    "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		  },
		"@type": "ResolvableItem",
		"@id": "safe://thisxorurl#somewhere?v=1",
		"target": "<target graph or safe url (xor or pubName); eg: 'somewhere'>",
        "targetType": "FilesMap"
	},
	{
		"@context": {
		    "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		  },
		"@type": "ResolvableItem",
		"@id": "safe://thisxorurl#email?v=2",
		"target": "<target graph or safe url (xor or pubName); eg: 'email'>",
		"targetType": "http://activitystream/#inbox"
	}
 ]
 ```

 This same structure can be applied to both PNS and XOR-URLs:

 `safe://www.happyurl` is the same as `<safe://asdadfiojf3289ry9uy329ryfhusdhfdsfsdsd#www>`

Providing different `@type` info or other details in the RDF can facilitate service discovery. In the example above, an email application could resolve `safe://happyurl`, and as the `default` value is a `Files Container` (which is does not want), could search remaining keys for something of `type: inbox` and resolve this data automatically.


#### Files Container

A `Files Container` (essentially mappings of `path`s to XOR-URLs ) is another type of resolver.

Currently we have 'NFS containers' which have their own structure which is effectively similar to the proposed `Resolvable Map`.

I would propose that we create a `Files Container` RDF type, which follows the same data structure and resolution patterns as `Resolvable Map` (indeed, it should probably be a subType). Which offers the advantage that the map could also contain more information related to NFS info:


```js
[
   {
	   "@context": {
		   "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		 },
		"@type": "FilesMap",
		"@id": "safe://thisxorurl?v=1",
		"default" : "index.html"
   },
   {
	   "@context": {
		   "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		 },
	   "@type": "FileItem",
	   "@id": "safe://thisxorurl/#/index.html?v=1",
       "targetType": 'html',
       "target": "<XOR-URL location>",
       "size": '22',
       "creationDate": '<UTC>',
       "updateDate": '<UTC>'
   },
   {
	   "@context": {
		   "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		 },
	   "@type": "FileItem",
	   "@id": "safe://thisxorurl/#/some/deep/path/amazing.js?v=1",
       "targetType": 'test/javascript',
       "target": "<XOR-URL location>",
       "size": '22',
       "creationDate": '<UTC>',
       "updateDate": '<UTC>'
   }
]

```

### PublicName Container Structure

The structure of a user's `_publicNames` container (for managing their `Public Names`) must be:

- The Public Name Map is an RDF AOD w/specific type tag (`1500`) stored at the sha3 hash of the `Public Name` string `shahash3('Public Name')`.
- A Public Name must point to a `Resolvable Map` RDF schema. With the target AOD location XOR-URL as the value to the key.
- A user's `Public Names` are saved/managed in the user's `_publicNames` container.
- A user's `_publicNames` container must be encrypted.


## Drawbacks

It changes the current DNS implementation, which will require updates to our libraries.

## Alternatives

One alternative is keeping things as is.

We could use some existing schemas for our RDF representations eg:
- http://schema.org/ItemList
- http://smiy.sourceforge.net/olo/spec/orderedlistontology.html

With default being determined by position (first position for eg).

Though there are disadvantages here in needed to parse the array to retrieve the `Sub Name` substring to use.


## Unresolved questions

- Fully flesh out the schemas for `Resolvable Map` / `Files Container` (if anything need be added for that.)
- Define error messages and codes
