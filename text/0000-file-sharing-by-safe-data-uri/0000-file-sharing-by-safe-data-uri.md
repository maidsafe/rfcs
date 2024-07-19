# File Sharing By SAFE Data URI 

- Status: proposed
- Type: new feature
- Related components: SAFE Beaker Browser, safe-js, safe_core
- Start Date: 22-11-2016
- Discussion: (Preliminary to be replaced) - [File sharing at the level of safe: URIs](https://forum.safedev.org/t/file-sharing-at-the-level-of-safe-urls/288?u=happybeing) 
- Supersedes: n/a
- Superseded by: n/a


## Summary

Provide a way to obtain a `safe:` URI based on the SAFE network address for any file stored using SAFE NFS, and which does not rely on a public ID or the SAFE DNS. Such a *SAFE Data URI* would be valid to use as part of a normal HTML page or as a file download link in SAFE Beaker Browser. This  makes sharing files much easier by avoiding the need for a special file sharing app, or for the user to have made a public ID and service for the purpose (see *Alternatives*). All that is needed is a way to generate a SAFE Data URI for a given file. It can then be downloaded using only a standard SAFE Browser, or accessed by any SAFE app. 

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).


## Motivation

Allowing web applications and SAFE Beaker to access files directly using a SAFE Data URI would allow, for example:

- an image captured by a SAFE web app and stored in a user's SAFE NFS storage, to be loaded into the DOM of the app, or into an HTML document generated by the app, and displayed within by the browser as part of the app UI or saved HTML.


- a *Share file...* command, easily implemented in any app, which offers up a SAFE Data URI for a file saved to or chosen from NFS 'drive'. This URI can be sent to anyone who, provided the file is public, could then access the file with just a SAFE web browser.  Private files could also be accessed this way but only by the authorised owner, as part of their browser history, or as quick access links in other applications or documents.

Without this feature, the SAFE NFS user would need to use a specialist file sharing application to share a file, and files accessed through such links would be less secure, hard to trust or guarantee permanence.

Providing access to SAFE files without requiring publication via SAFE DNS, makes it easy to port existing apps, or for someone who already knows how to create and load files using traditional methods to offer this functionality in new apps. 

The ability to guarantee permanence of download links will no doubt also lead to the creation of new and innovative kinds of application.

## Required Functionality

A way of identifying the file to be shared and obtaining a SAFE Data URI for it.

For SAFE Beaker Browser to recognise and process a SAFE Data URI differently from an ordinary `safe:` URI.


## Detailed design

We need a format of `safe:` URI which SAFE Beaker (and all SAFE compatible browsers) can recognise and use to access content stored on SAFE network without reference to a SAFE public ID. 

Each file stored on SAFE network is addressed using a [DataIdentifier](https://github.com/maidsafe/routing/blob/7c00dd14a2c4e4b2a3f7813a13edad119c0efa83/src/data/mod.rs#L111), so we can use that to address the file on the network. 

We must also consider different kinds of file and how to provide expected and appropriate behaviour, depending on how the file is stored (e.g. as immutable data, mutable data, key-value store etc.), how it is identified by the user (e.g. via a directory listing obtained from SAFE NFS), and the type of the file (e.g. text, PDF, HTML, binary, video etc.).

### SAFE Web URI Structure

To recap existing functionality, a `safe:` website URI is similar to a standard web URI, but always begins with `safe` rather than `http` or `https`. This link protocol indicates the resource is to be located on the SAFE network. For a website published on SAFE network, we create a public ID and service using SAFE NFS, and this allows SAFE Beaker to interpret URIs of the following form:

	`safe://[service.]publicid[/filepath]`

`service` is optional, and defaults to `www`

`filepath` is also optional, and if service is ommitted or given as `www`, the filepath defaults to `index.html`

NOTE: In the above and subsequent notation, square brackets denote an optional component. So `[service.]publicid` means either a value which equates to a `service` followed by a '.' followed by a value for a `publicid`, or just a value for a `publicid`. So for example, the following URIs would all access the same file assuming it exists in a directory that has been published at the given service and public ID:

	`safe://rfcs`
	`safe://www.rfcs`
	`safe://rfcs/index.html`
	`safe://www.rfcs/index.html`

All the above is existing functionality, so the format of *SAFE Data URIs* must always be distinguishable from this.

### SAFE Data URI Structure

SAFE Beaker must be able to recognise a SAFE Data URI, parse it, and determine what method to use to access the file, and what if any metadata to provide with the response it delivers. We can also choose to provide a way for a human readable filename to be conveyed as part of the URI, rather than base this on the address or having the user type it in.

**A SAFE Data URI always begins with `safe:<type>:`** This allows us to continue to permit a normal `safe:` URI to omit or include the conventional but redundant '//' that we are accustomed to in web URIs.

The complete structure of a SAFE Data URI is then:

	`safe:type:address[:filename]`

Where:

- `type` is one of `file:` or `folder:` or `dns:`. So far this RFC has only mentioned retrieving regular files accessed using SAFE NFS, but including a type instructs the client on the type of data being retrieved access and how to make use of it, while providing for additional types to be added later, with backwards compatibility.

- `address` is the a base64 encoded serialisation of a [DataIdentifier](https://github.com/maidsafe/routing/blob/7c00dd14a2c4e4b2a3f7813a13edad119c0efa83/src/data/mod.rs#L111).

- `filename` is the URI encoded value of a string that must not include the path separator character '/' (i.e it is a filename without a path, and with an optional file extension).

Examples might be:

	`safe:file:U2VjdXJlIEFjY2VzcyBGb3IgRXZlcnlvbmU=:happybeing.png`
	`safe:file:U2VjdXJlIEFjY2VzcyBGb3IgRXZlcnlvbmU=:Safecoin-white-paper.pdf`
	`safe:file:U2VjdXJlIEFjY2VzcyBGb3IgRXZlcnlvbmU=`


This RFC does not consider the details or use of `folder:` or `dns:` or other types as these can be defined later. For now we only need implement type `file:`

The author believes that this method of addressing SAFE NFS stored files will be feasible according to [RFC0046-new-auth-flow](https://github.com/maidsafe/rfcs/tree/master/text/0046-new-auth-flow) which uses a system of [Containers](https://github.com/maidsafe/rfcs/blob/master/text/0046-new-auth-flow/containers.md#nfs-convention) to emulate file system at the NFS level. 

This though will depend on the decisions taken around implementing emulation NFS and the underlying NFS file structures. 

**Permanent URIs:** SAFE NFS files are immutable data, so a SAFE Data URI will by default be a permanent link to immutable data.

#### Extensibility
This URI scheme is simple and retains the potential to be extended to provide for additional data types, and to control their retrieval and interpretation.

For example, values within a key-value store (in a MutableData object) could be addressed directly by extending the Data URI with parameters. So for a given application, appending `?key=postaladdress&encoding=json` to a URI could be used to retrieve the value of `name` from a MutableData object and parse it ready for use.

### Behaviour Options - Discussion

Assuming the shared file is immutable, we can guarantee that the SAFE Data URI will also be permanent. If the file were either mutable or deliberately referenced via a mutable data structure then a URI could be made impermanent by design. So disregarding at this stage any limitations imposed by the implementation, we should in reviewing this RFC consider the viability and usefulness of designing SAFE Data URIs to be *either* permanent or impermanent.

**IMPORTANT:** At this point both options are on the table so we can debate what is expected and desirable, but as is stands this RFC proposes to implement only permanent SAFE Data URIs. Some points for discussion are presented below.

#### Permanent URIs

If all SAFE Data URIs are valid forever, we have the advantage of certainty, and the feature of permanent access to all files shared in this manner.

It wouldn't ensure that SAFE websites were always available forever and immutable, because generally these utilise ordinary SAFE URIs that map to a mutable file structure, but the advent of *Permanent URIs* would make it *possible* to create a SAFE web app or website that could be inspected to verify it as permanent and immutable.

The question is whether we would want to limit SAFE Data URIs to only allow sharing of immutable data, or to allow permanent *and* impermanent URIs (see next). A guarantee of permanence for all SAFE Data URIs would have to be verifiable (i.e. within safe_core), because it would not be desirable for people to assume that these were permanent and immutable if there was a way of subverting this with a customised SAFE Data URI maker).

#### Impermanent URIs

There are many good things about permanent URIs, but also the disadvantage for users who accidentally share something that can never subsequently be unshared. There are also things people want to be able to share temporarily. For example, sharing a link in order that the intended recipient access a file, which can later be invalidated to reduce the chance that someone else who discovers the URI can also obtain the file.  

In general, what one shares one day, or in one state of mind, might later be regretted and so in certain circumstances it might also be useful to provide a way to generate URIs that can be invalidated. Impermanence is also likely to be expected behaviour, so if it is not, we might need to find ways to make that very clear. 

On the other hand, applications could be developed to extend file sharing to allow the creation of deletable URIs if this is seen as desired, which also helps make it clear that the default is permanence.

If we create built-in support for *both* permanent and impermanent URIs it would be sensible to indicate this as part of the URI so that users know what they are sharing and receiving. For example, by adding '-p' or '-i' to the end of the `type` as in:

	`safe:file-p:U2VjdXJlIEFjY2VzcyBGb3IgRXZlcnlvbmU=:Safecoin-white-paper.pdf`

But this would allow someone to create a Data URI, and then deceive recipients by changing the `file-i` to `file-p` (or vice versa). So there are problems with allowing both: adding the option of impermanent URIs may affect the value and usefulness of permanence.

Adding support for impermanent URIs would require a *delete SAFE Data URI* API, which raises a further question as to whether or not we should allow different files to appear at the same address at different times, and if not then how this could be prevented. It would mean that SAFE Data URIs could not always be guaranteed to be permanent, which might be undesirable. It also creates a problem for anyone in possession of a SAFE Data URI: how to tell if it is permanent or not.

**IMPORTANT:** For simplicity's sake, this RFC mandates only permanent URIs (so only type `file:` and never `file-p:` or `file-i:`) which means that with this proposal we can always guarantee SAFE Data URIs are permanent, and we can leave the tricky questions for a later upgrade discussion.

### SAFE NFS API Changes

Assuming only *permanent SAFE Data URIs* are supported, SAFE NFS API to be extended with:

	`getPermanentLink(fileName: String) -> Future<String>` 

	`fileName`	- the full path of an existing file in the user's NFS storage

This returns a permanently valid URI for fileName formatted as a SAFE Data URI including `type`, `address` and `filename` elements.

If we later support impermanent URIs a suitably named function can be added which would use a type of `file-i` instead of `file`

### safe-js Changes

safe-js to be extended with `safeNFS.getPermanentLink()` which returns a Promise that resolves to a SAFE Data URI as above.

### SAFE Beaker Changes

SAFE Beaker will detect every SAFE Data URI, parse it according to the above, and use this information to:

a) access the file on the network

b) provide error handling where retrieving the file fails or times out

c) deliver the content appropriately (e.g. embedded in HTML content, embedded download link, or location bar access) including where provided: applying metadata to assist in rendering embedded content, and supplying a default filename when offering to download. 


## Drawbacks

Relying on the implementation of SAFE NFS storage means this functionality would be difficult to maintain if that ever changes.

There might be privacy or tracking issues associated with sharing URIs if they could somehow be linked back to a user or account, though none are immediately apparent, except for ill considered naming of files.


## Alternatives

Not doing this would require users to find other ways of sharing files, which since they are not built in to safe_core and SAFE Beaker will be more cumbersome (for users and for app developers). Also, it would be impossible to ensure permanence, and harder to trust in terms of security (i.e. from tracking and surveillance). Making such a widely used feature harder than necessary would reduce the rate and potential level of adoption of the SAFE Network, thereby diminishing its positive overall impact on security and privacy.

Alternatives would probably consume more network resources than providing support for direct links within safe_core, increasing user PUT costs and network load for a feature we can expect would be used by almost every SAFE user.

### Application based or shared DNS

Rather than locating a file using a network address, we might use either the existing SAFE DNS to share files saved in the user or application managed storage. SAFE NFS at present would be cumbersome to use for this, requiring every app which wants to generate file URIs to have access to a user's registered public ID and service so this is not practical.

There are changes being considered which may make a DNS based sharing URIs more practical (see [this dev forum post](https://forum.safedev.org/t/file-sharing-at-the-level-of-safe-URIs/288/7?u=happybeing)) but these have not been decided and have not been considered as part of this RFC. Once clear, it may be worthwhile reviewing this option.	


## Unresolved questions

It needs to be confirmed that proposed modifications to the implementation of SAFE NFS [Containers](https://github.com/maidsafe/rfcs/blob/master/text/0046-new-auth-flow/containers.md#nfs-convention) would support this functionality if adopted.