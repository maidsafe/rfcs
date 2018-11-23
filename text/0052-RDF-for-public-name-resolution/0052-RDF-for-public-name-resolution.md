# RDF for PublicName Resolution

- Status: proposed
- Type: enhancement
- Related components: Safe Browser. Safe App Nodejs / Client FFI libs.
- Start Date: 23/09/2018
- Discussion: https://github.com/maidsafe/rfcs/issues/283
- Supersedes: -
- Superseded by: -

## Summary

This proposal looks to enhance the domain name system by using a resource description framework.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- XOR-URL refers to a url generated as part of the content addressable system for accessing `xorname` urls in the safe_browser. As described here: https://forum.safedev.org/t/xor-address-urls-xor-urls/1952
- Data is presented as RDF, serialised in [JSON-LD](https://json-ld.org/) in the examples.
- I'm using MutableData ( `MD` ) and ImmutableData ( `ID` ) as shorthands.


## Motivation

The aim here is to use [RDF data](https://en.wikipedia.org/wiki/Resource_Description_Framework) for our application data and the SAFE Domain Name System. This enables any application encountering the data to know it's purpose, and handle it accordingly.

Equally, we have no formal description of the DNS resolution at this time, so this would codify that.

It also seeks to propose new terminology to clarify some terms and clashes with traditional nomenclature (offering an alternative to DNS).

This RFC proposes that url schemes such as `safe://somewhere` can have sub-domain's with resolvable services which can
be either chosen by the end user application, or default to specific data.

## Detailed design

### Nomenclature

- Use `Public Name System` (PNS) instead of `DNS` to avoid confusion and clarify this is a SAFE network term, and that it relates to `Public Names`.
	- Here the URL terminology for `host` is equivalent to a SAFE `Public Name`.
	- What is called `subdomains` on the clearnet are referred to as `Sub Names`.
- Introduce `Resolvable Map` schema to describe RDF data on the network that can resolve a `key` to a XOR-URL. This is described below and can be used by:
	- `Public Name` MDs.
	- A `Files Map`: an alternative to the NFS style container, with similar functionality but described using RDF (and the `Resolvable Map` schema)


### URL resolution

Before diving into data structures, I wanted to described how URL resolution will work in a browser (such as the safe_browser).

#### 1. XOR-URLs.

Any resolver should first attempt to parse a URL for being a valid [XOR-URLs](https://forum.safedev.org/t/xor-address-urls-xor-urls/1952). This is handled via the `webFetch` API (of `safe_node-app` or similar)

If so, it is resolved via XOR-URL and if pointing to a ResolvableMap MD, resolution continues thereafter as described in the `Resolvable Map` or `Files Map` as appropriate (if the url has a `path` or `url fragments` eg.).


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

- As above, resolving each additional `subName` will lead to another `Resolvable Map`... on and on up to a defined maximum of redirects (implemented by the resolver.)
	- eg. Safe Browser will implement redirect limit of 10 redirects per url resolution. Any more than this would throw an error.

Unavailability of any data being dereferenced will throw an error.


#### 3. Path resolution

`safe://<subName>.<publicName><path>`, eg `safe://pns.rfc/resolution`

Once the final MD has been resolved, if a `Files Map` type of `Resolvable Map` has been located, then the trailing `path` of the url would be resolved as part of that `Map`, too.


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
		"@id": "safe://thisxorurl#somewhere",
		"target": "<target graph or safe url (xor or pubName); eg: 'somewhere'>",
        "targetType": "FilesMap"
	},
	{
		"@context": {
		    "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		  },
		"@type": "ResolvableItem",
		"@id": "safe://thisxorurl#email",
		"target": "<target graph or safe url (xor or pubName); eg: 'email'>",
		"targetType": "http://activitystream/#inbox"
	}
 ]
 ```

 This same structure can be applied to both PNS and XOR-URLs:

 `safe://www.happyurl` is the same as `<safe://asdadfiojf3289ry9uy329ryfhusdhfdsfsdsd#www>`

Providing different type of graph can facilitate service discovery. In the example above, an email application could resolve `safe://happyurl`, and as the `default` value is a `Files Map` (which is does not want), could search remaining keys for something of `type: inbox` and resolve this data automatically.


#### Files Map

A `Files Map` (essentially mappings of `path`s to XOR-URLs ) is another type of resolver.

Currently we have 'NFS containers' which have their own structure which is effectively similar to the proposed `Resolvable Map`.

I would propose that we create a `Files Map` RDF type, which follows the same data structure and resolution patterns as `Resolvable Map` (indeed, it should probably be a subType). Which offers the advantage that the map could also contain more information related to NFS info:


```js
[
   {
	   "@context": {
		   "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		 },
		"@type": "FilesMap",
		"@id": "safe://thisxorurl",
		"default" : "index.html"
   },
   {
	   "@context": {
		   "@vocab": "https://raw.githubusercontent.com/joshuef/sschema/master/src/"
		 },
	   "@type": "FileItem",
	   "@id": "safe://thisxorurl/#/index.html",
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
	   "@id": "safe://thisxorurl/#/some/deep/path/amazing.js",
       "targetType": 'test/javascript',
       "target": "<XOR-URL location>",
       "size": '22',
       "creationDate": '<UTC>',
       "updateDate": '<UTC>'
   }
]

```

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

- Fully flesh out the schemas for `Resolvable Map` / `Files Map` (if anything need be added for that.)
- Define error messages and codes
