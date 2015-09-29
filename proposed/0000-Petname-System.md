- Feature Name: Petname System
- Type: feature
- Related components: `safe_dns`, `safe_client`
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

* Key: Public, globally unique identifier in the form of a public key

* Nickname: Publicly suggested name attached to a Key

* Petname: Private user-assigned unique name for a Key

* Persona: Public representation of an individual entity on the SAFE Network

* Share: Public or shared directory

* Referral: Key + Nickname

* Address Book: List of user-assigned Petnames

* Human-memorable: This means that a human being has a chance of remembering
the name.

* Securely Unique: This is means that the Key cannot be forged or mimicked.

* Denotate: To specify/signify as a specific type.
	* *He denotated “NSA-friend 1” as his Petname for the Nickname “Google”.*

* Arroba: The 'at sign' ("@")


# Motivation

>Each computer has a unique view of the network.

The proposal put forth is motivated by the need to solve the problem known as "Zooko's Triangle". For more information about this and how it works in theory, I invite you to view both [Mark Stiegler's outline](http://www.skyhunter.com/marcs/petnames/IntroPetNames.html) and [Md. Sadek Ferdous's paper](http://www.researchgate.net/publication/221426438_Security_Usability_of_Petname_Systems), both written on the subject. A separate thread for questions specifically aimed at the theory and reasoning behind the Petname System has been opened, and can be found [here](https://forum.safenetwork.io/t/the-petname-system/5111/1).

Zooko's Triangle argues that if a system is to be decentralized, one single namespace cannot be global, securely unique, and memorable, all at the same time. Domain names are an example: they are global, and memorable, but they are neither decentralized nor securely unique. The Petname System aims to both abstract away the globally unique identifiers of users and locations, and present them in a securely unique, memorable way.


# Detailed design

## Rules of Thumb

1. There cannot be more than one Petname or Nickname per Key. (see: Unresolved Questions [1])
2. No one user can have two Petnames that are similar. (see: Unresolved Questions [2],[3])
3. Nicknames need not be unique.
4. Any given Petname may be changed at the user's discretion.
5. Any given Petname may be deleted at the user's discretion.
6. Any Persona or Share can have a Nickname, but does not necessarily have to have one.
7. Any Nickname will be a Structured Data type containing the Nickname and the Key.
8. Creation of a Persona or Share comes with the option of creating a Nickname for that Persona.
10. A Nickname should only be utilized after checking the user's Address Book for a Petname assigned to that Key.
11. When a Nickname is returned for a given function, any action on that Nickname should trigger a prompt to assign that Nickname to a Petname, defaulting to the Nickname if one is available.

## Nickname Denotation

Nicknames must always be specified as Nicknames - and made brutally obvious to the user that they *are*, in fact, Nicknames. They are to be changed immediately and globally once a user assigns a Petname to them. They are then to be, by default, stored forever as the Petname that the user assigns. Petnames must always be used if previously assigned. 

There should be a reserved character to denotate that this *is* a Nickname (see: Unresolved Questions [4]). If two nicknames are rendered in the same display, there should be a way to distinguish this as well.  One way to distinguish this is to add a trailing identifier which are the first X number of hexadecimal bytes of that Key. So a persona with the Nickname "Bart" with the Key “518A...” would be "Bart@518A..." if we set the reserved character to be an arroba. (see: Unresolved Questions [5]). If, however, there is no Nickname attached to the provided Key, the name would be represented with only the reserved character and the identifier.

## Absolute/Unique Navigation

Since there is no globally unique human-memorable addresses in the Petname System, URLs/URIs will be deprecated. They will be replaced with a Key input to reference unique data. This will be used sparingly, but it is still relevant in order to be able to navigate to a specific site without any digital referral. No App developer can use anything other than Keys to reference any other data on the SAFE Network.

This Key input cannot accept Nicknames as keys. Even though Nicknames have the ability to be indexed into a searchable database, they are not meant to be an absolute resolver because they lack the attribute of being globally unique. While a search engine for Nicknames is not included in this proposal, it is certainly an application that this proposal facilitates.

Any file or directory inside of a Share can be referenced by its name. Since the Key is globally unique, any pathname appended to it must therefore be globally unique also just by virtue of being an extension of an already unique pointer.

## Namespace Transmogrification
### From the network to the user

The input to this API should always be a Key. The output of this API should be either a Nickname or Petname depending on whether or not the user has a Petname associated with that particular Key.

It also should be able to produce the Key upon request. This behavior will not be default, but should be accessible through this API.

### From the user to the network

The input to this API should be either a Nickname or Petname, depending on whether or not the user has a Petname associated with that particular Key. The output of this API should always be a Key.

The input should be able to specify that it using a Key upon request. This behavior will not be default, but should be accessible through this API.

## Address book

The Address Book will be implemented as a directory in the Drive (see: Unresolved Questions [6]). In there will contain any Structured Data that is a Petname. Multiple Petnames can fit into one piece of Structured Data, so even though a user need not create a new Structured Data type each time a Petname is denotated, it must either be put into an existing Petname Structured Data type or create a new one. (see: Unresolved Questions [7])

There will also exist a way of symlinking to public Structured Data for free in that same Petname directory (see: Unresolved Questions [8],[9]). That symlink will indicate that the Nickname of that Key will also be treated by that user as their Petname. That would ensure that a user could casually browse the same as they would casually demo an APP on the SAFE Network – without having to pay for a PUT fee if used in a specific way. At the same time, this would incentivize entities (companies or individuals) to come up with novel Nicknames in hopes that it would be unique in the user's Address Book. If a user can denotate a Petname for free, it stands to reason that they would do so every time it was possible.

## Code

This seems to affect `safe_client` and potentially `safe_vault`. This will implement many ideas of `safe_dns` while modifying its functionality. This will not affect any low-level crates such as `routing` or `crust` as those act directly on public Keys alone. It is unknown at this time if this could affect `safe_nfs`, `drive`, or `self_authentication` for the generation of Nicknames.


# Drawbacks

## 1. Sharing Keys off-the-grid

Inherently in the Petname System has what I like to call the "Paper Napkin Problem". This also is referred to as the "Moving Bus Problem". I do, however, differentiate the two, as there are two different aspects to this behavior generated by the Petname System.

While this is a distinct drawback, it is one that I do not believe will have much of an impact in the adaptation or proliferation of the SAFE Network. Sharing Referrals to online locations have become increasingly digital. The Petname System encourages by design the ability to share very specific, globally unique Keys to another digitally. These Referrals will become the *de facto* method of referencing specific bits of information on the SAFE Network.

## 2. Discovery

Initially, the SAFE Network will be difficult to transverse with no referrals or Keys. 

This problem that can be mitigated into any SAFE Browser with a similar functionality to Firefox's "Home" or "New Tab" pages, which provide introductory referrals. These designs can act as a “yellowpages”-type service, or with similar functionality to the “Hidden Wiki” in TOR. Where the SAFE Network is designed to be decentralized, any of these Referral applications need not be.

## 3. Cost

A user needs to pay for the Structured Data in their Address Book. 

To make any new denotation, they would add a distinct Petname to their Address Book. However, if there already exists a Structured Data object with free space, a new Petname can be appended to that at no cost. Also, if a user were to symlink to a Nickname to use as the Petname, there will also be no cost.

On the flip side, to register a Nickname, whether it be for a Persona or a Share, the user would pay to the network a fee to create a Structured Data chunk and store it in a publicly available Share that can be accessed by anyone. That data chunk would contain both the Nickname and the Key of that data.


# Alternatives

## DNS

There have been several other DNS suggestions that have surfaced on the forums as of late. These proposals have requested to be hardcoded into the SAFE Network, but that would be disallowed by the Petname System. Here, the Petname System would supersede them, yet at the same time allow them to be implemented.

While that may seem contradictory, it is actually quite feasible. It would require an App to be built that would index and categorize Keys, providing the users with some form of either shared Petname functionality (similar to Gnunet), an existing DNS-type setup, or something comletely different (e.g. Continuous Bidding). While the Petname System is able to avoid utilizing this, it does not close the door on creating and operating a service such as these could provide.

## Personas

There is already an implementation planned for Personas, and that is to have a non-unique name tied to a 10 digit Identifier. The name is non-unique, but the identifier must be sufficiently different than any other identifier that is tied to that same name. This creates an artificial scarcity for Personas in the far future.

This proposed system will use the Public Key as the Identifier under the hood, and present the user with the Nickname that the Public Key corresponds with. If a user wishes, they may inspect the Key to determine the unique identifier of that Persona. However, once an action is taken on that Persona, the Petname system will prompt the user to assign that Persona a Nickname. This ensures a memorable association with that Persona.

# Examples

Since this impacts the user-facing interface most significantly, it may be beneficial to show a mock-up of what this may look like in the most straightforward of examples.

## IM Chat screen

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
<Hillary> "Bob w/o punctuation"
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

To make your payment final and to recieve these amazing items, please submit your information to Paypa1@155F now!
**--click on the referral to pay --**
```

A couple differences from above. Inside of the brackets, you can see that Alice has already denotated a Petname for this site. That Petname being "Testing this site - SAFEbAY". It appears that she wanted to remind herself that she was only testing the site, perhaps to see if it was a scam? Let's see what she found out.

As a bit of background, Alice had been using Paypal's services on the SAFE Network for a while, and stuck with the original Nickname of "Paypal". So when the Nickname+Identifier popped up, she knew *automatically* that the link was **not** to the site which she was familiar with. Rather, this was a site that she had not made a Petname for, and had probably never visited before.

Lastly, since this is not HTML encoded, the link is the referral - Nickname+Identifier - and nothing else.

## Physical World Petnames

From [Mark Stiegler's outline](http://www.skyhunter.com/marcs/petnames/IntroPetNames.html):

> Humans have been using parts of petname systems since before the invention of the written word. Human faces were used as keys. These keys resisted forgery far better than most things that pass for security today on computers (except in episodes of Mission Impossible, and the occasional Shakespearian comedy like 12th Night). The referral, "Joe, this is my son Billy, he's great with a club," transferred a key/alleged-name pair. The recipient of this referral typically accepts the alleged name as a petname, though in some cases the recipient may instead choose other petnames, such as, "Bob's big dumb dufus of a son", which is a strictly private petname.

The Petname System has been used throughout the entire history of humanity, almost transparently! This gives me great hope that if a digital system were to completely fulfill the requirements of creating an implementation of the Petname System, it would be an unmitigated success. It would truly grant secure access for everyone.

# Unresolved questions

[1] Would separating Petnames into both Personas and Shares and allowing duplicate Petnames between them compromise the security aspect of this system? If this is argued to be irrelevant, would it become more relevant if the Petnames were to be split up further (Wallets, APPs, Personas, Shares, etc…)

[2] What algorithm can be used to determine if a Petname is "too similar" to another?

[3] If a user tries to add a Nickname to their Address Book, but there already exists a Nickname the same or similar, what is the resolution process?

[4] What should the reserved Nickname differentiation character be?
	* # - Hash
	* % - Percentage Sign
	* + - Plus Sign
	* @ - Arroba

[5] What number of bytes of the Key should be appended onto the Nickname separated by the reserved character when displayed as Nicknames?

[6] At what level will the symlink occur? (Datamap, NFS, Drive) 

[7] With the Petname Database spanning across multiple SD Blobs, how will the lookup be implemented?

[8] How can the symlink be made free for the user?

Drive: a symlink to some network location visible on the Drive will presumably be stored in some other StructuredData object, a parent directory for example, the addition of the symlink in that case will be free since the cost is incurred when the parent directory is first PUT onto the network.

[9] The public SD must be able to be referenced by it's version. What threat would a changed Nickname represent to a symlinked Petname?

[10] If a Persona publicly owns a Share, can they have the same Petname? This would implement the `www.`, `blog.`, `xray.` in the Nickname space, which is currently avoided by the proposed system.

[11] What attacks are inherent to this system?
