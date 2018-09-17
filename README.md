# SAFE Network RFCs
_This process attempts to emulate the success of the Rust programming language
and as such has almost mirrored the [RFC process the Rust developers
use](https://github.com/rust-lang/rfcs), which is tried and tested and appears
to work very well._

# Introduction
[Introduction]: #introduction
Many changes, including bug fixes and documentation improvements can be
implemented and reviewed via the normal GitHub pull request workflow.

Some changes though are "substantial", and we ask that these be put
through a bit of a design process and produce a consensus among the
community and the core team.

The "RFC" (request for comments) process is intended to provide a
consistent and controlled path for new features to enter the network
and core libraries, so that all stakeholders can be confident about
the direction in which the network is evolving.

## Table of Contents
[Table of Contents]: #table-of-contents

- [Active RFCs](#list-of-active-rfcs)
- [When you need to follow this process](#when-you-need-to-follow-this-process)
- [Before creating an RFC](#before-creating-an-rfc)
- [What the process is](#what-the-process-is)
- [The RFC life-cycle](#the-rfc-life-cycle)
- [Reviewing RFCs](#reviewing-rfcs)
- [Implementing an RFC](#implementing-an-rfc)
- [RFC Postponement](#rfc-postponement)
- [Help! This is all too informal](#help-this-is-all-too-informal)

## List of active RFCs
All currently-active RFCs can be found on the [status page](./RFCs-by-status.md#active-rfcs).

## When you need to follow this process
[When you need to follow this process]: #when-you-need-to-follow-this-process
You need to follow this process if you intend to make "substantial"
changes to SAFE libraries, dependencies, algorithms or the RFC process itself.
What constitutes a "substantial" change is evolving based on community norms,
but may include the following:

- Any semantic or syntactic change to the existing algorithms and process
  that is not a bug fix.
- Any proposed additions to existing algorithms
- Any proposed additional functionality
- Anything that reduces interoperability (e.g. changes to the wire protocol or data serialisation.)

Some changes do not require an RFC:

- Rephrasing, re-organising, refactoring, or otherwise "changing shape
  does not change meaning"
- Additions that strictly improve objective, numerical quality
  criteria (warning removal, speedup, better platform coverage, more
  parallelism, trap more errors…etc…)
- Additions only likely to be _noticed by_ other developers-of-safe,
  invisible to app-developers-of-safe (i.e. API users)

If you submit a pull request to implement a new feature without going
through the RFC process, it may be closed with a polite request to
submit an RFC first.

## Before creating an RFC
[Before creating an RFC]: #before-creating-an-rfc
A hastily proposed RFC can hurt its chances of acceptance. Low quality
proposals, proposals for previously rejected features, may be quickly
rejected, which can be demotivating for the unprepared contributor.
Laying some groundwork ahead of the RFC can make the process smoother.

Although there is no single way to prepare for submitting an RFC, it
is generally a good idea to pursue feedback from other project
developers beforehand to ascertain that the RFC may be desirable.
Having a consistent impact on the project requires concerted effort
toward consensus-building.

The most common preparations for writing and submitting an RFC include
filing and discussing ideas on the [RFC issue tracker][issues], and occasionally
posting "pre-RFCs" on the [SAFE Dev Forum][rfcs category] for early
review.

As a rule of thumb, receiving encouraging feedback from long-standing
project developers, and particularly members of the core team or existing
contributors, is a good indication that the RFC is worth pursuing.

## What the process is
[What the process is]: #what-the-process-is
In short, to get a major feature added, one must first get the
RFC merged into the RFC repo as a markdown file. At that point, the RFC
is "proposed" and may be implemented with the goal of eventual inclusion
into SAFE, at which point it becomes "active".

- Fork the RFC repo [RFC repository](https://github.com/maidsafe/rfcs)
- Decide on a clear and brief title for the new rfc, make it descriptive and unique. Copy `0000-template.md` to `text/0000-my-new-rfc/0000-my-new-rfc.md`, where 'my-new-rfc' is the [kebab-cased version](https://en.wikipedia.org/wiki/Letter_case#Case_styles) of the RFC title, any non-letters removed. Don't assign an RFC number yet!
- Fill in the RFC. Put care into the details: RFCs that do not present
  convincing motivation, demonstrate understanding of the impact of the
  design, or are disingenuous about the drawbacks or alternatives tend to be
  poorly-received.
- Submit a pull request. As a pull request the RFC will receive design
  feedback from the larger community, and the author should be prepared to
  revise it in response.
- Each pull request will be labelled with the most relevant sub-team, which
  will lead to its being triaged by that team in a future meeting and assigned
  to a member of the sub-team.
- Build consensus and integrate feedback. RFCs that have broad support are
  much more likely to make progress than those that don't receive any
  comments. Feel free to reach out to the RFC assignee in particular to get
  help identifying stakeholders and obstacles.
- The sub-team will discuss the RFC pull request, as much as possible in the
  comment thread of the pull request itself. Offline discussion will be
  summarised on the pull request comment thread.
- RFCs rarely go through this process unchanged, especially as alternatives
  and drawbacks are shown. You can make edits, big and small, to the RFC to
  clarify or change the design, but make changes as new commits to the pull
  request, and leave a comment on the pull request explaining your changes.
  Specifically, do not squash or rebase commits after they are visible on the
  pull request.
- At some point, a member of the sub-team will propose a "motion for final
  comment period" (FCP), along with a *disposition* for the RFC (merge, close,
  or postpone).
  - This step is taken when enough of the tradeoffs have been discussed that
    the sub-team is in a position to make a decision. That does not require
    consensus amongst all participants in the RFC thread (which is usually
    impossible). However, the argument supporting the disposition on the RFC
    needs to have already been clearly articulated, and there should not be a
    strong consensus *against* that position outside of the sub-team. Sub-team
    members use their best judgement in taking this step, and the FCP itself
    ensures there is ample time and notification for stakeholders to push back
    if it is made prematurely.
  - For RFCs with lengthy discussion, the motion to FCP is usually preceded by
    a *summary comment* trying to lay out the current state of the discussion
    and major tradeoffs/points of disagreement.
  - Before actually entering FCP, *all* members of the sub-team must sign off;
    this is often the point at which many sub-team members first review the RFC
    in full depth.
- The FCP lasts ten calendar days, so that it is open for at least 5 business
  days. This way all stakeholders have a chance to lodge any final objections before a decision
  is reached.
- In most cases, the FCP period is quiet, and the RFC is either merged or
  closed. However, sometimes substantial new arguments or ideas are raised,
  the FCP is cancelled, and the RFC goes back into development mode.

## The RFC life-cycle
[The RFC life-cycle]: #the-rfc-life-cycle
Once an RFC becomes active then authors may implement it and submit
the feature as a pull request to the repo. Being "active" is not
a rubber stamp and in particular still does not mean the feature will
ultimately be merged. It does mean that in principle all the major
stakeholders have agreed to the feature and are amenable to merging
it.

Furthermore, the fact that a given RFC has been accepted and is
"active" implies nothing about what priority is assigned to its
implementation, nor does it imply anything about whether a
developer has been assigned the task of implementing the feature.
While it is not *necessary* that the author of the RFC also write the
implementation, it is by far the most effective way to see an RFC
through to completion. Authors should not expect that other project
developers will take on responsibility for implementing their accepted
feature.

Modifications to active RFCs can be done in follow up PRs.  We strive
to write each RFC in a manner that it will reflect the final design of
the feature, however, the nature of the process means that we cannot expect
every merged RFC to actually reflect what the end result will be at
the time of the next major release. We therefore try to keep each RFC
document somewhat in sync with the network feature as planned,
tracking such changes via followup pull requests to the document.

An RFC that makes it through the entire process to implementation is
considered "implemented" and is moved to the "implemented" folder. An RFC
that fails after becoming active is "rejected" and moves to the
"rejected" folder.

## Reviewing RFCs
[Reviewing RFCs]: #reviewing-rfcs
While the RFC pull request is up, the sub-team may schedule meetings with the
author and/or relevant stakeholders to discuss the issues in greater detail,
and in some cases the topic may be discussed at a sub-team meeting. In either
case a summary from the meeting will be posted back to the RFC pull request.

A sub-team makes final decisions about RFCs after the benefits and drawbacks
are well understood. These decisions can be made at any time, but the sub-team
will regularly issue decisions. When a decision is made, the RFC pull request
will either be merged or closed. In either case, if the reasoning is not clear
from the discussion in thread, the sub-team will add a comment describing the
rationale for the decision.

## Implementing an RFC
[Implementing an RFC]: #implementing-an-rfc
Some accepted RFCs represent vital features that need to be
implemented right away. Other accepted RFCs can represent features
that can wait until some arbitrary developer feels like doing the
work. Every accepted RFC has an associated issue tracking its
implementation in the affected repositories. Therefore, the
associated issue can be assigned a priority via the triage process that
the team uses for all issues in the appropriate repositories.

The author of an RFC is not obligated to implement it. Of course, the
RFC author (like any other developer) is welcome to post an
implementation for review after the RFC has been accepted.

If you are interested in working on the implementation for an "active"
RFC, but cannot determine if someone else is already working on it,
feel free to ask (e.g. by leaving a comment on the associated issue).

## RFC Postponement
[RFC Postponement]: #rfc-postponement
Some RFC pull requests are tagged with the "postponed" label when they are
closed (as part of the rejection process). An RFC closed with "postponed" is
marked as such because we want neither to think about evaluating the proposal
nor about implementing the described feature until some time in the future, and
we believe that we can afford to wait until then to do so. Historically,
"postponed" was used to postpone features until after 1.0. Postponed pull
requests may be re-opened when the time is right. We don't have any formal
process for that, you should ask members of the relevant sub-team.

Usually an RFC pull request marked as "postponed" has already passed an
informal first round of evaluation, namely the round of "do we think we would
ever possibly consider making this change, as outlined in the RFC pull request,
or some semi-obvious variation of it." (When the answer to the latter question
is "no", then the appropriate response is to close the RFC, not postpone it.)

### Help! This is all too informal
[Help this is all too informal!]: #help-this-is-all-too-informal
The process is intended to be as lightweight as reasonable for the
present circumstances. As usual, we are trying to let the process be
driven by consensus and community norms, not impose more structure than necessary.
