- Feature Name: Petname System
- Type: feature
- Related components: `safe_launcher`, `safe_client`
- Conflicts with: Decentralized Naming Service (dns); Issue #23
- Start Date: September 27, 2015
- RFC PR: 
- Issue number: 


# Summary

I propose that a Petname System be implemented on the SAFE Network. This Petname System does not compromise any property that is required to ensure that a naming system is specific, human-memorable, globally unique, and decentralized. It does not attempt to do this by instituting one identifier, but rather by implementing many that interact with each other to provide a global, memorable, unique, and decentralized naming system.

The implementation should provide developers with an API to efficiently use the Petname System. This library should handle use cases where either a Key or a Petname can be used interchangeably without any additional programming logic on the developers' shoulders. When taking action on the Nickname, the API should automatically prompt for the creation of the Petname if one is not assigned yet.

The expected outcome is that a standard API be created with the sole purpose of handling Key, Petname, and Nickname conversion. This implementation is analogous to the current DNS proposal in the sense that the Nicknames will be grouped into one public Share. However, the Nicknames need not be unique.

This Petname System should be applicable to both Personas and Shares.

## Definitions

* Key: 64 Byte globally unique identifier - corresponding to the location of the data on the network (see: Unresolved Questions [9])

* Nickname: Publicly suggested name attached to a Key

* Petname: Private user-assigned unique name for a Key

* Persona: Public representation of an individual entity on the SAFE Network

* Share: Public or shared directory

* Referral: Key + Nickname

* Address Book: List of user-assigned Petnames

* Key Book: List of publicly available Nicknames.

* Archive: List of previous versions of publicly available Nicknames.

* Service Map: The list of content that is associated with a Persona or Share; displayed as a Datamap

* Human-memorable: This means that a human being has a chance of remembering the name.

* Securely Unique: This is means that the Key cannot be forged or mimicked.

* Denotate: To specify/signify as a specific type.
    * *He denotated “NSA-friend 1” as his Petname for the Nickname “Google”.*

* Arroba: The 'at sign' ("@")


# Motivation

> Identification in nature is relative, never absolute.

The proposal put forth is motivated by the need to solve the problem known as "Zooko's Triangle". For more information about this and how it works in theory, I invite you to view both [Mark Stiegler's outline](http://www.skyhunter.com/marcs/petnames/IntroPetNames.html) and [Md. Sadek Ferdous's paper](http://www.researchgate.net/publication/221426438_Security_Usability_of_Petname_Systems), both written on the subject. A separate thread for questions specifically aimed at the theory and reasoning behind the Petname System has been opened, and can be found [here](https://forum.safenetwork.io/t/the-petname-system/5111/1).

Zooko's Triangle argues that if a system is to be decentralized, one single namespace cannot be global, securely unique, and memorable, all at the same time. Domain names are an example: they are global, and memorable, but they are neither decentralized nor securely unique. The Petname System aims to both abstract away the globally unique identifiers of users and locations, and present them in a securely unique, memorable way.


# Detailed Design

## Rules of Thumb

1. There cannot be more than one Petname or Nickname per Key.
2. No one user can have two Petnames that are similar. (see: Unresolved Questions [1],[2])
3. Nicknames need not be unique.
4. Any given Petname may be changed at the user's discretion.
5. Any given Petname may be deleted at the user's discretion.
6. Any Persona or Share can have a Nickname, but does not necessarily have to have one.
7. Any Nickname will be a Structured Data type containing at least the Nickname and the Key.
8. Creation of a Persona or Share comes with the option of creating a Nickname for that Persona or Share.
10. The Key Book should only be utilized after checking the user's Address Book for a Petname matching the name submitted.
11. When a Nickname is returned for a given function, any action on that Nickname should trigger a prompt to assign that Nickname to a Petname, defaulting to the Nickname if one is available.

## Nickname Denotation

Nicknames must always be specified as Nicknames - and made brutally obvious to the user that they *are*, in fact, Nicknames. They are to be changed immediately and globally (from the user's point of view) once a user assigns a Petname to them. They are then to be, by default, stored forever as the Petname that the user assigns. Petnames must always be used if previously assigned.

This is the burden that App developers are to shoulder with their UI. The Return Value (see below) of this functionality will provide the developer a way to gather whether the returned name is either a Petname or a Nickname. It is then up to them to present the users with an easy-to-understand way of differentiating the two.

## Absolute/Unique Navigation

Since there is no globally unique human-memorable addresses in the Petname System, URLs/URIs will be deprecated. They will be replaced with a Key input to reference unique data. This will be used sparingly, but it is still relevant in order to be able to navigate to a specific site without any digital Referral. No App developer can use anything other than Keys to reference any other data on the SAFE Network.

This Key input cannot accept Nicknames as Keys. Even though Nicknames have the ability to be indexed into a searchable database, they are not meant to be an absolute resolver because they lack the attribute of being globally unique. While a search engine for Nicknames is not included in this proposal, it is certainly an application that this proposal facilitates.

Any file or directory underneath a Nicknamed Persona or Share can be referenced by its name. Since the Key is globally unique, any name appended to it must therefore be globally unique also just by virtue of being an extension of an already unique pointer.

## Namespace Transmogrification
### From the network to the user

The input to this API should always be a Key. The output of this API should be either a Nickname or Petname depending on whether or not the user has a Petname associated with that particular Key.

It also should be able to produce the Key upon request. This behavior will not be default, but should be accessible through this API.

### From the user to the network

The input to this API should be either a Nickname or Petname, depending on whether or not the user has a Petname associated with that particular Key. The output of this API should always be a Key.

The input should be able to specify that it using a Key upon request. This behavior will not be default, but should be accessible through this API.

## Address Book

The Address Book will be implemented as a directory in the Drive (see: Unresolved Questions [6]). This will be assigned to default location as a private directory in the user's Drive.  Address Books are unique to individual logins, and only that one user can *ever* edit any Petnames that had been specified there. (see: Footnote [3])

Address Books can contain links to others' Address Books as a way to share Petnames between users *only* if all of the Petnames are either unique, or any identical Petnames reference the same Keys.

The Address Book will contain any Structured Data that is a Petname. Multiple Petnames can fit into one piece of Structured Data, so even though a user need not create a new Structured Data type each time a Petname is denotated, it must either be put into an existing Petname Structured Data type or create a new one. (see: Unresolved Questions [7])

### Adding to the Address Book

Adding to the Address Book is a matter of creating a datamap to a directory that a user wishes to denotate with a Petname and specifying that Petname. The datamap can point to anything that is on the network. Whether it be a location, a list of encryption keys, an APP, another datamap, or anything else. If it can be indexed in a datamap, it can be placed into the Address book.

### Symlinking from a Nickname

There will also exist a way of symlinking to an entry in the Key Book for free in that same Petname directory (see: Unresolved Questions [8]). That symlink will indicate that the Nickname of that Key will also be treated by that user as their Petname. The symlink should be able to be either to the entry in full, or only a particular service of that entry.

That would ensure that a user could casually browse the same as they would casually demo an APP on the SAFE Network; i.e. without having to pay for a PUT fee if used in a specific way. At the same time, this would incentivize entities (companies or individuals) to come up with novel Nicknames in hopes that it would be unique in the user's Address Book. If a user can denotate a Petname for free, it stands to reason that they would do so every time it was possible.

This symlink will refer to a specific version of the Key Book entry in order to assure that the reference, once assigned remains static and does not change even though the entry in the Key Book may. (see: Archive)

## Key Book

A reference to the "Phone Book" of olden days, this will hold all of the Nickname-Key pairs that have been published. This index will be public to everyone and be assigned to a default location in order that the network can access it regardless of the client logging in. This will hold the Structured Data that contains the public Nickname that the publisher denotates.

These entries in the index will be versioned so that in the event that a name is changed, if an individual symlinked to a specific Nickname as their Petname, that denotation will not change even if the public lookup has. Version numbers will always increment when a change occurs. Nickname entries will not be "deleted" primarily because this is a shared directory, and the datamap should not be able to be taken away from, as that may open vectors of attack of a devastatingly large nature. The individual entries have the capability to point to either null values or perhaps a creation of a default: "This Nickname has been rescinded, moved, or renamed by the owners" variable is in order. Either way, with the Archive (explained below) there will always be a previous version to view.

In order to facilitate this, an Archive will be created to store older versions of the Nickname index.

## Archive

The Archive (as introduced above) is a way to make sure that if a Petname is symlinked to an entry in the Key Book, that is unchanged even in the event that the entry in the Key Book is modified by the owner of the entry. The Archive will consist of pieces of Immutable Data that are copies of the previous Structured Data which was changed. An Archive version will always be made if a change occurs.

This Archive is an index of Immutable Data. It contains all previous versions of any given entry in the Key Book, (if any) with each pice of Immutable Data containing one version. These entries are created upon modification of the Key Book entry automatically at the cost of the owner/modifier of the entry.

The Archive is not to be used to look up any Nickname requests, and is only used if the Petname symlink requests a version that the current Key Book entry is greater than.

It is unknown at this time if the Key Book entry will be required to specifically link to these Archive entries. (see: Footnote [1])

## Service Map

> What's in a name? That which we call a rose

> By any other name would smell as sweet;

> *-- William Shakespear - Romeo & Juliet*

The actual content of the Nickname or the Petname will be a datamap that points to a `home` directory for a Persona or Share. This directory houses datamaps and/or directories containing any and all content that is linked to that Persona. This will also include the messaging encryption keys and bio (if included).

If this Nickname does *not* correspond with a specific Persona (as an individual user) and rather it is a pointer to an APP or a public Share, there is no such need for personal information, however any content that is related to it (such as binary executable files or documentation) can be pointed to in the contained datamaps. This both allows Petnames and Nicknames to be either Personas or Shares independently, or both simultaneously.

To keep this initial datamap small in order to fit it into a piece of Structured Data, it is encouraged that the datamap will only contain references to other datamaps corresponding to the different types of content that are associated with that Persona. However, particularly small references may prefer to directly link to the data itself.

This enables a services-like aspect to the Petname System. It also allows APPs to direct the user to store their shared information that is relevant to that particular app in a datamap inside of the Nickname Datamap. Thereby associating any data that the user wishes to be linked to the Nickname to be automatically done. APPs only then have to look at the user's datamap to retrieve their data for that particular APP. It also may be restricted so that that APP may only view the information in it's allocated datamap, and only that particular APP. Any configuration or private information can be PUT elsewhere; e.g. in a private directory owned by the user.

## Code

This seems to affect `safe_client`, `safe_launcher`, and potentially `safe_vault`. This will implement many ideas of `safe_dns` while modifying its functionality. This will not affect any low-level crates such as `routing` or `crust` as those act directly on Keys alone. It is unknown at this time if this could affect `safe_nfs`, `drive`, or `self_authentication` for the generation of Nicknames.

### Structured Data

Similar to the proposed `safe_dns` this is a use case for Unified Structured Data. Referencing RFC 0000 for the Unified Structured Data fields, those will be populated as follows:

#### Nicknames

1. The `type_tag` will be "5" (per RFC 0002 - Reserved Names)
2. Identifier field: The Key (see Definitions: Key)
3. The data field:
```
struct naming {
    Name: String
    // This is a datamap of the datamaps (directories) that the Persona or Share has associated with it.
    // The primitive for the datamap is unknown at this time
    Datamap: Vec<T>
    Archive: Hashmap(version_number, hash(archive_immutable_data_object))
}
```
4. Owners' keys is self-explanatory
5. Version is incremented whenever a change is made to the data
6. Previous owner's keys is self-explanatory
7. Signature - a signature of the mutable fields above

To retrieve matching entries for a Key, the application will simply search for that Key in both the user's Address Book, followed by the Key Book if not found previously. To retrieve matching entries for a Nickname, an indexing mechanism must be built to parse the data fields and retrieve the Identifier in order to return the Key.

#### Petnames

The implementation for `petname` is more tentative, as it is Structured Data only because the user must be free to modify a petname without experiencing any negative ramifications, such as cost. The implementation of Structured Data allows having mutable data that is not subject to a cost if modified. If this can be implemented through Immutable Data, that may be a better solution as it can host more data at the same price point than could Structured Data. However, I do not believe that setup to be able to be feasibly attained, and therefore have no option other than utilizing Structured Data.

My initial idea is that the Petname directory will consist of multiple piece of mutable (Structured Data type) data that forms an list of the petname index itself.

A rough outline is as follows:

1. The `type_tag` will be "6" (to be added to RFC 0002 - Reserved Names)
2. The Identifier will be random as it is not necessary
3. The data field:
```
{
    struct naming {
        Name: String
        Datamap: Vec<T>
    }
}
{
    struct naming {
        Name: String
        Datamap: Vec<T>
    }
}
[...etc...]
```
4. Owners' keys will be the user's keys
5. Version is incremented whenever a change is made to the data
6. Previous owner's keys is self-explanatory
7. Signature - a signature of the mutable fields above

### Return Value

The return value of any given function should be a tuple of `(string, bool)`. This corresponds to the name and whether that name is a Petname or a Nickname. This allows App devs to handle the aspect of differentiating Petnames from Nicknames in their UI accordingly.

>The effort spent in fixing a fundamentally flawed model of the "Internet as Television" is wasted in futility and compounds the problems by simply creating new crimes like "Cybersquatting". Instead, this effort can directed to removing the artificial scarcity engendered by this flawed view of the Internet and the Web. The phone books and other directory services are far more effective at handling names and the computers obviate the need to every type or even see the URLs.
*-- [Bob Frankston](http://www.frankston.com/public/?name=dnssafehaven)*

This does imply though, that this kind of system would deviate from the existing paradigm. The existing paradigm being that the specific location is returned given the name. In this case, the name is returned given a specific location.

### Naming Crate

A draft for the `safe_naming` crate is as follows.

The Parent directory will have a global `lib.rs` and `errors.rs`. There will be an initial module located in a subdirectory of `naming_operations`.

The `naming_operations` module will handle the transformation between Name and Key once a matching piece of data has been found. It will also handle the logic to determine when to resort to a Nickname lookup if a Petname lookup has failed. It will have two sub-modules: `petname` and `nickname`.

Both modules will be similar in function, but perform their operations independently. The `petname` module for instance will specify the location of the Petname directory, while the `nickname` module specifies the location of the Nickname directory.

The `petname` module includes functions to:

* Specify the Petname file/directory
* Retrieve Petname entries and their Keys from the Address Book
* Add or Remove Petnames
* Modify Petnames

The `nickname module includes functions to:

* Specify the Nickname file/directory
* Retrieve Nickname entries and their Keys from the Key Book
* Add Nickname entries
* Modify Nicknames
* Perform an Nickname archive upon modification
* Prompt for Petname assignment

Lastly, a visual representation of the module structure draft:

```
      safe_naming
          |
    naming_operations
    |               |
petname         nickname
```


# Drawbacks

## 1. Sharing Keys off-the-grid

Inherently in the Petname System has what I like to call the "Paper Napkin Problem". This also is referred to as the "Moving Bus Problem". I do, however, differentiate the two, as there are two different aspects to this behavior generated by the Petname System.

While this is a distinct drawback, it is one that I do not believe will have much of an impact in the adaptation or proliferation of the SAFE Network. Sharing Referrals to online locations has become increasingly digital. The Petname System encourages by design the ability to share very specific, globally unique Keys to another digitally. These Referrals will become the *de facto* method of referencing specific bits of information on the SAFE Network. (see: Footnote [2])

## 2. Discovery

Initially, the SAFE Network will be difficult to transverse with no Referrals or Keys.

This problem that can be mitigated into any SAFE Browser with a similar functionality to Firefox's "Home" or "New Tab" pages, which provide introductory Referrals. These designs can act as a “yellowpages”-type service, or with similar functionality to the “Hidden Wiki” in TOR. Where the SAFE Network is designed to be decentralized, any of these Referral applications need not be.

## 3. Cost

A user needs to pay for the Structured Data in their Address Book.

To make any new denotation, they would add a distinct Petname to their Address Book. However, if there already exists a Structured Data object with free space, a new Petname can be appended to that at no cost. Also, if a user were to symlink to a Nickname to use as the Petname, there will also be no cost. Lastly, if a user delets a Petname, it *is* actually deleted, which makes space for alternative Petnames to be added to the user's Address Book.

On the flip side, to register a Nickname, whether it be for a Persona or a Share, the user would pay to the network a fee to create a Structured Data chunk and store it in a publicly available Share that can be accessed by anyone. This would not require a payment to use. That data chunk would contain both the Nickname and the Key of that data.

However, whenever changing a Nickname entry, a cost is incurred to create an Archive entry for the previous version. This cost both discourages the changing of Nicknames to ensure a sense of continuity of the network, as well as to facilitate static Petname \<-\> Nickname symlinking.


# Alternatives

## DNS

There have been several other DNS suggestions that have surfaced on the forums as of late. These proposals have requested to be hardcoded into the SAFE Network, but that would be disallowed by the Petname System. Here, the Petname System would supersede them, yet at the same time allow them to be implemented.

While that may seem contradictory, it is actually quite feasible. It would require an App to be built that would index and categorize Keys, providing the users with some form of either shared Petname functionality (similar to Gnunet), an existing DNS-type setup, or something comletely different (e.g. Continuous Bidding). While the Petname System is able to avoid utilizing this, it does not close the door on creating and operating a service such as these could provide.

## Personas

There is already an implementation planned for Personas, and that is to have a non-unique name tied to a 10 digit Identifier. The name is non-unique, but the identifier must be sufficiently different than any other identifier that is tied to that same name. This creates an artificial scarcity for Personas in the far future.

Also, there seems to be no consensus whether creating a Persona should incur a cost, and if so, how much and why that amount (we must try to avoid magic numbers). This system states that there *will* be a cost to create a Persona, but that there is no limit to how many Personas can be created. Also, this allows APPs to either utilize Personas, or to keep the content completely anonymous, depending *solely* on what the user chooses.

This proposed system will use the Identifier as the Key under the hood, and present the user with the Nickname/Petname that the Key corresponds with. If a user wishes, they may inspect the Key to determine the unique identifier of that Persona. However, once an action is taken on that Persona, the Petname system will prompt the user to assign that Persona a Nickname. This ensures a memorable association with that Persona. As well, this proposed API is set up in such a way as to encourage developers to have the UI inspect the Key every time, and if necessary, intelligently present the user with any abnormalities.

# Examples

Since this impacts the user-facing interface most significantly, it may be beneficial to show a mock-up of what this may look like in the most straightforward of examples.

## IM Chat

In this scenario, a entity having the Persona "Bob" has a Petname "Hill" for a specific Persona that he knows. The Nickname for that Persona is "Hillary"

```
"Hill" has initiated a chat with you ->

<Hill> Hi @Bob!
<Bob> Hi @Hill
<Hill> On my screen, my name that you just typed shows up as "Hillary"
<Hill> What does it show on yours?
<Bob> Hill
<Hill> You can call me whatever you want @Bob,
<Hill> Just don't call me late for dinner!
<Bob> And what do you see my name as?
<Hill> Oh, you're just plain ol' "Bob". ;)
```

Compare that to a scenario in which Bob had chosen *not* to change the Nickname when assigning the Petname.

```
"Hillary" has initiated a chat with you ->

<Hillary> Hi @Bob!
<Bob> Hi @Hillary
<Hillary> On my screen, my name shows up as "Hillary"
<Hillary> What does it show on yours?
<Bob> Hillary
<Hillary> I see that I chose a good Nickname!
<Hillary> Hopefully it spreads all throughout the network,
<Hillary> Then I'd be the best-known "Hillary" out there!
<Bob> And what do you see my name as
<Hillary> "Bob who never uses punctuation"
<Hillary> The other "Bob" I know writes as well as Shakespeare himself!
<Hillary> If you ever get around to cleaning up your act I might have to change it! ;)
```

## Referrals

Imagine if you would, getting an HTML encoded email:

```
Congratulations Alice! 

You submitted the winning bid for the mixing bowl and measuring cup set on eBay!

To make your payment final and to recieve these amazing items, please submit your information to Paypa1 now!
**-- click here to pay --**
```

There's two things wrong with this. First of all, it's impossible to see upfront where the link is redirecting the user to. Either the link would be taken at it's word, or have to be examined to see where it was leading to user to. This is a reason why I view HTML encoding as a threat. Not everything is as it seems.

Also, if you thought "Paypa1" looked weird, give yourself a pat on the back. The lowercase "l" that is expected at the end of the word has been replaced by a number one "1". Didn't catch it? Well, don't worry, phishing and other cybercrimes rely on social engineering to "lure" victims into their traps. A relevant example is typo squatting. In the existing internet, when someone wants to look up `wikipedia.org`, but accidently types `wikiepdia.org`, whoever registered that misspelled name will now have direct access to the user's browser.

Let's look at the same email as a email-type message on a SAFE Network that implements the Petname System.

```
Congratulations Alice!

You submitted the winning bid for the mixing bowl and measuring cup set on [Testing this site - SAFEbAY]! 

To make your payment final and to recieve these amazing items, please submit your information to 155F@Paypa1 now!
**--click on the referral to pay --**
```

A couple differences from above. Inside of the brackets, you can see that Alice has already denotated a Petname for this site. That Petname being "Testing this site - SAFEbAY". It appears that she wanted to remind herself that she was only testing the site, perhaps to see if it was a scam? Let's see what she found out.

As a bit of background, Alice had been using Paypal's services on the SAFE Network for a while, and stuck with the original Nickname of "Paypal". So when the Nickname+Identifier popped up, she knew *automatically* that the link was **not** to the site which she was familiar with. Rather, this was a site that she had not made a Petname for, and had probably never visited before.

Lastly, since this is not HTML encoded, the link is the Referral - Key + Nickname - and nothing else.

## Physical World

> Humans have been using parts of petname systems since before the invention of the written word. Human faces were used as keys. These keys resisted forgery far better than most things that pass for security today on computers (except in episodes of Mission Impossible, and the occasional Shakespearian comedy like 12th Night). The referral, "Joe, this is my son Billy, he's great with a club," transferred a key/alleged-name pair. The recipient of this referral typically accepts the alleged name as a petname, though in some cases the recipient may instead choose other petnames, such as, "Bob's big dumb dufus of a son", which is a strictly private petname.
> *-- [Mark Stiegler](http://www.skyhunter.com/marcs/petnames/IntroPetNames.html)*

The Petname System has been used throughout the entire history of humanity, almost transparently! This gives me great hope that if a digital system were to completely fulfill the requirements of creating an implementation of the Petname System, it would be an unmitigated success. It would truly grant secure access for everyone.

# Unresolved questions

[1] What algorithm can be used to determine if a Petname is "too similar" to another?

[2] If a user tries to add a Nickname to their Address Book, but there already exists a Petname that is the same or similar, what is the resolution process?

[6] At what level will the symlink occur? (Datamap, NFS, Drive) 

[7] With the Petname Database spanning across multiple SD Blobs, how will the lookup be implemented?

One way to do this is to create a simple JSON formatted list of Petnames that will mimick the structure that a Nickname entry in the Key Book follows. This way multiple Petnames can be stored without having to assign an individual Structured Data type to each one. The requirement that it be Structured Data is that so it may be modified without incurring a cost to the user.

[8] How can the symlink be made free for the user?

Drive: a symlink to some network location visible on the Drive will presumably be stored in some other StructuredData object, a parent directory for example, the addition of the symlink in that case will be free since the cost is incurred when the parent directory is first PUT onto the network.

>Better to think like this, For any data you store from outside SAFE then you pay. Any data you store/copy from within SAFE is free.
> *-- [@dirvine](https://forum.safenetwork.io/t/abusive-scenario-about-de-duplication/4787/11?u=smacz)*

[9] How will this be derived? Keep in mind that the datamap is ever growing and expanding. And the name is subject to change.

[10] What attacks are inherent to this system?

# Footnotes

[1] If so, that exemplifies the need to prohibit the deletion of a Nickname entry in the Key Book.

[2] This will be quite easy with Petnames, as the user need only specify their particular Petname, and everyone who sees this will interpret it based on if they have a separate Nickname for that entity/data, or whether they resort to viewing that as a Nickname if one is available. If one is not available, it may be the case that only the Key or a portion of the Key. (along with the Nickname denotation reserved character)

This will be harder with Nicknames, as selecting a Nickname would have to invoke a search and selection of which Nickname was meant. Also, per the Rules of Thumb (see: Detailed Design), acting on a Nickname will necessarily prompt the user to assign that Nickname a Petname, thereby alleviating the problem of specifying Nicknames.

[3] This means that if a user adds a Petname, then only that user can modify or delete that Petname. That means that only that user has write access to that directory. As far as *sharing* Address Books go, a shared Address Book will be read-only to any who are copying it. However, it is still modifyable by the original user and as such it is also subject to change. This is to be considered unsafe and only to be used when a user trusts another entity's Address Book. This is a good space for companies to develop a shared Address Book access for sale, with various guarentees to the users regarding the static nature of the Petnames.
