# RDF for PublicName Resolution

- Status: proposed
- Type: enhancement
- Related components: Safe Browser. Safe App Nodejs / Client FFI libs.
- Start Date: 23/09/2018
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

## Summary

This proposal looks to enhance the domain name system by using a resource description framework.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- CAS refers to a content addressable system for accessing `xorname` urls in the safe_browser. As described here: https://forum.safedev.org/t/xor-address-urls-xor-urls/1952
- Data is presented in a json, as RDF , as described over here: <ADD URL>
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
- Introduce `Resolvable Map` schema to describe RDF data on the network that can be resolved a `key` to a CAS url. This is described below and can be used by:
	- `Public Names` containers.
	- `Files Container` an alternative to the NFS style container, with similar functionality but described using RDF (and the `Resolvable Map` schema)


### URL resolution

Before diving into data structures, I wanted to described how URL resolution will work in a browser (such as the safe_browser).

#### 1. CAS.

Any resolver should first attempt to parse a URL for being a valid [CAS URLs](https://forum.safedev.org/t/xor-address-urls-xor-urls/1952). This is handled via the `webFetch` API (of `safe_node-app` or similar)

If so, it is resolved via CAS and if pointing to a ResolvableMap MD, resolution continues thereafter as described in the `Resolvable Map` or `Files Container` as appropriate (if the url has a `path` or `url fragments` eg.).


#### 2. PublicNameSystem.

Failing to be detected as a CAS url, we then parse the url and use the Public Name System to resolve for data.

Here the URL terminology for `host` is equivalent to a SAFE `Public Name`. What are known as `subdomains` on the clearnet are referred to as `SubNames`.

`safe://<subName>.<host>`

- GET the `Public Name` container for `host`.
- GET the `Resolvable Map` for this container
- Resolve the `Sub Name` graph from this `Resolvable Map`.

Unavailability of any data being dereferenced will throw an error.

##### 2.1 No `SubName` aka Default Services.

- GET the `Public Name` container for `host`.
- GET the `Resolvable Map` for this container
- Resolve the `:default` CAS url if available.

Unavailability of any data being dereferenced will throw an error.


##### 2.2 Many `SubName`s

`safe://<subName>.<subName>.<subName>.<subName>.<host>`

- As above, resolving each additional substring, up to a defined maximum of redirects (implemented in the resolver.)
	- I propose a Safe Browser redirect limit of 20 redirects per url resolution. Any more than this would throw an error.

Unavailability of any data being dereferenced will throw an error.


#### 3. Path resolution

`safe://<subName>.<host><path>`, eg `safe://pns.rf/resolution`

Once the final data has been resolved in a browser, if a `FilesContainer` type of `Resolvable Map` has been located, then the trailing url path would be resolved, too.


#### Data Structures


##### PublicName Structure

- The PublicName is an RDF MD w/specific type tag (`1500`) stored at the shahash3 of the `Public Name` string `shahash3('Public Name')`.
- The PublicName MD must be a `Resolvable Map`, with the hashed MD location CAS url as the value to the `Public Name` key.
- A user's `_publicNames` container must be encrypted.



#### Resolvable Map Structure

An RDF Graph stored on the safe network. This will follow a newly defined schema, that represents a list of
`keys`, which map to (CAS) `urls`. Each entry can contain more information to aid in resolving data, depending on context / application.

The RDF document will also contain a `:default` graph, which points to the desired resolution if no `key` is provided.

The RDF document will have a version relating to the version of the `Resolvable Map` data structure in use. (starting at `v1`.)

Provides data to be shown at the public name.
 - It must be an RDF data object
 `<safe/ResolvableMap>`, `Sub Name` graphs will pointing to a CAS url for data location.
 - Extra data can be added to the graph for each entry to aid in service discovery for the key.


 For `safe://<subName>.<myPublicName>`


 With the following, `safe://happyurl` would resolve to the `name` graph's entry.
 ```js
 {
     // context+info
     @type : 'safe/ResolvableMap',
     subtype : 'safe/ResolvableMap',
	 version : 1,
     // this is what 'www' was doing previously in our DNS setup.
     :default :  {
         @id: '<this xor url>',
		 uri: '<target xor>',
         @type: 'NFS',
     },
     somewhere : {
         @id: '<this xor url>',
		 uri: '<target xor>',
         @type: 'NFS'
     },
     email : {
         @id: 'xor url#name'
         uri: '<target xor>'
         @type: 'inbox'
     }
 }
 ```

 This same structure can be applied to both PNS and CAS:

 `safe://www.happyurl` is the same as `<safe://asdadfiojf3289ry9uy329ryfhusdhfdsfsdsd#www>`

Providing different `@type` info or other details in the RDF can facilitate service discovery. In the example above, an email application could resolve `safe://happyurl`, and as the `:default` value is an NFS container, could search remaining keys for something of `type: inbox` and resolve this data automatically.


#### Files Container

A `Files Container` (essentially mappings of `path`s to CAS urls ) is another type of resolver.

Currently we have NFS containers which have their own structure which is effectively similar to the proposed `Resolvable Map`.

I would propose that we create a `Files Container` RDF type, which follows the same data structure and resolution patterns as `Resolvable Map` (indeed, it should probably be a subType). Which offers the advantage that the map could also contain more information related to NFS info:


```js
{
  "@context": "safe/ResolvableMap",
  "@type": "safe/ResolvableMap",
  "subtype": "FilesContainer",
  "url": "<xor url of this>",
  "numberOfItems": "315",
  "default" : 3,
  "/some/website/index.html" : {
      filename: 'index.html'
      @type: 'html',
      @id: "<XORURL location>",
      size: '22',
      creationDate: '<UTC>',
      updateDate: '<UTC>',
  },
  "/some/website/amazing.js" : {
      filename: 'amazing.js'
      @type: 'text/javascript',
      @id: "<XORURL location>",
      size: '22',
      creationDate: '<UTC>',
      updateDate: '<UTC>',
  }
}
}

```



## Drawbacks

It changes the current DNS implementation, which will require updates to our libs and applications.

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
