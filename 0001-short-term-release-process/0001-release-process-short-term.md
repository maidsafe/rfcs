# Short-Term Release Process for Autonomi

- Status: proposed
- Type: enhancement
- Related components:
- Start Date: 2024-07-08
- Discussion:
- Supersedes:
- Superseded by:

## Summary

This document intends to provide a detailed development and release process we can follow until the
initial launch of the network in October/November. It will be based on short release cycles that
will last about three weeks. After launch, the cycles would likely be at least double that length
and could be quite different in detail.

It should be clear that we deal here with processes for releasing code, but we don't go into any
detail on how to deploy it. There are issues there we will also have to address, but those will be
for another document.

## Conventions

Uncertainties are indicated using a `*` notation. Rather than have a full `Unresolved Questions`
section at the end, where applicable there is an `Uncertainties` list in each section.

## Motivations

So far, we have failed to arrive at a well-defined process for releasing our code. With launch
imminent, it is my opinion that we've now run out of road for experimentation and we need to settle
on an unambiguous process that we can execute without deliberation. Finalising this document should
be a collaborative process in which we address any uncertainties rather than leaving them open for
further experiment.

## Detailed Design

## General Branching/Merging Techniques

The chosen branching and release model is closely correlated with Gitflow. Gitflow is a mature model
that has proven useful for projects that cannot operate using continuous delivery. [This](https://nvie.com/posts/a-successful-git-branching-model/) article originally defined the model. Reading it will help you familiarise yourself with the general concepts.

In our setup, we would have two permanent branches, `main` and `stable`, where `main` is for
day-to-day development and `stable` is essentially for tracking releases we consider deployable.
Like Gitflow, we would also have some temporary branches; these will be discussed in more detail
later.

### Merging

Historically, our team has preferred `git rebase` to `git merge`; however, Gitflow is better
supported by using `git merge`. Rebasing is fine before submitting upstream, but Gitflow involves
merging commits between different branches. Due to the fact that rebasing rewrites the commit
history, merging between branches can lead to commits that have the same content but different
hashes. This can make subsequent, post-rebase merging more confusing than it needs to be and also
cause the commit history to be littered with duplicates. The primary option for completing a PR
should be to merge it in rather than rebase.

## Release Cycle Overview

This section provides an overview of a release cycle. Certain aspects here merit more detail, but
those will be provided in subsequent sections.

The cycle has the following phases and steps:

* Continual internal development and testing:
    - Feature branches are worked on and merged back into `main`
    - Developers can deploy their own isolated testnets if necessary
* Release candidate (RC) phase (two weeks):
    - Twice a month, roughly corresponding two every two weeks, create a `release-YYYY.MM.rc.1`
      branch from `main`. No new features will be accepted on this branch.
    - Bump version numbers, with an `rc.1` suffix applied.
    - Build RC and release to Github as a public `pre-release` but with no published crates.
    - Deploy RC to `STG-01`.
    - Deploy previous stable release to `STG-02`.
    - Production of changelog begins.
    - Invite community members to test against this network.
    - Initiate comparisons to the previous stable release in `STG-02`.
    - Fixes can be applied to the release branch, resulting in an `rc.2`. This will be
      released/deployed/tested. Repeat if necessary.
* Release and deploy phase (one/two days):
    - If the RC passes two weeks of QA, the release branch is ready to be merged back to `stable`.
      On that branch, modify version numbers to remove the `rc` pre-release specifier.
    - Changelog finalised.
    - Merge the release branch into `stable`.
    - Merge the release branch into `main`.
    - When ready, use a GHA workflow to perform a stable release.
    - Delete the release branch.
    - Deploy the new release to nodes being hosted by Maidsafe.
    - Announce to users.

We can accommodate hotfixes at any point during the cycle.

## Release Cycle Anatomy

We'll now elaborate the release cycle described in the last section, discussing each type of release
in more detail.

### Artifacts

Our releases currently produce eight binary artifacts, so we'll discuss these first.

In Rust, crates must use Semantic Versioning. A binary is defined within a crate, and therefore, by
default, it will also have a Semantic Version; however, it is possible to override the `--version`
argument to provide something custom. It would be useful if we could refer to these collectively
with a single version number and package, where the package name would reflect the version number.
The collective version number will be `YYYY.MM.X.Y`, where `X` is for the release cycle within the
month, and `Y` is a counter that will increment for each RC branch produced within the cycle.

The collective version number is used for a single Github Release. The assets for the release are
the combined binary packages for each platform. The changelog can also be nicely applied to this
combined release.

To accommodate these and some other things, our binaries will now have several version arguments,
which are defined as follows:

* `--package-version`: outputs the collective `YYYY.MM.X.Y` version
* `--crate-version`: outputs the crate's Semantic Version
* `--protocol-version`: outputs the network protocol version (correlated with the Semantic Version)
* `--version`: outputs all of the above

### Alpha Releases

An alpha release would accommodate a scenario in which we wanted to put out quick, experimental code
to have community users test something on an isolated network. We can branch it off `main` and
discard the branch when it's done. The branch is intended to have a very short duration. It will be
possible to apply fixes to it and do a new release on the same branch, in which case the pre-version
specifier will be incremented. If the fixes are good, we can cherry pick the fix commits back into
`main`.

An owner should be designated to the whole experiment. They will produce the release, deploy it, and
coordinate with users.

#### Process

* Prepare a light description that communicates the purpose of the release; we do not need a full
  changelog for this type of release.
* Create and checkout an `YYYY-MM-DD-alpha.1` branch from `main`.
* Use `release-plz update` to bump crate versions.
* Use a script to apply the `alpha` pre-release specifier to each bumped crate.
* Create a `chore(release): alpha-YYYY-MM-DD` commit.
* Push that branch to the upstream `maidsafe` repo, which will kick off the release workflow.
* The release workflow will produce a public `pre-release` on Github, but the crates will *not* be
  published.
* Manually edit the Github Release to provide the description prepared in the first step.
* Use the `Launch Network` workflow to deploy the `alpha` binaries to an isolated network.
* Announce the availability of the binaries to the community. Users can use `safenode-manager`
  and/or `safeup` with `--version` arguments to obtain the alpha binaries.
* Perform testing
* If a problem is identified, there is an opportunity for a small fix/test cycle:
    - On the same alpha branch, apply the fix.
    - Use `cargo release version alpha --package X --package Y` to increment the necessary crates to
      `alpha.2`
    - Deploy the fix either using an upgrade or by launching a new testnet
    - Users can test
    - Repeat if necessary (in practice we should not have many of these)
* If need be, any fix commits applied on the alpha branch should be cherry picked into `main`.
* The experiment is over and the branch should be deleted.

The branch is discarded because we don't want the alpha version bumps back in `main`. Crates were
also not published. A Github Release always creates a tag, so this will function as the historical
record of the existence of the alpha release.

### Release Candidates

A release candidate (RC) is the binary that's intended to be released as a stable version. The set
of features and fixes in the RC is what's included on `main` in the current cycle, i.e., between now
and the last stable release. The release candidate branch will be cut twice a month, roughly every
two weeks. Community users will be invited to participate in testing.

Once the RC branch is started, we won't accept new features on it, only fixes. Feature development
can continue on `main`.

#### Process

* Create and checkout an `YYYY-MM-DD-rc.1` branch from `main`.
* Use `release-plz update` to bump version numbers. These new versions should be the ones used for
  the stable release that will be based on this RC branch.
* Use a custom script to apply `rc.1` to the new versions. We can't use `cargo release` for this
  because it also performs a `PATCH` bump when you apply the pre-release specifier, which is very
  annoying.
* Create a new template entry in the changelog. This can be filled out as an on-going process
  between now and the stable release.
* Create a `chore(release): YYYY-MM-DD-rc.1` with the version bump and changelog and push the branch
  to `origin` or `upstream`.
* That push should trigger a workflow that will:
    - Build the RC
    - Upload the binaries to S3
    - Produce a public `pre-release` Github Release 
    - Crates will NOT be published
* Use a script to produce a list of the commits between now and the last stable version. The commits
  will be grouped by author. This list can be posted in Slack or Discourse to aid developers in
  supplying their contributions for the changelog.
* Use an `Upgrade Network` workflow to deploy the RC to `STG-01`. This will function as a test of
  the upgrade process and help identify breaking changes we may have missed.
* Invite users to participate in testing the RC. They can use `safenode-manager` to obtain the `rc`
  nodes and `safeup` for `rc` clients.
* We can also perform our own QA testing, some of which will come from the metrics that result from
  the comparison to the previous stable release.
* If fixes are necessary, they should be applied to this branch. We then bump to `rc.2` and do
  another release, which again should be deployed to `STG-01`. Users can get the new binaries. This
  part could potentially be repeated, but obviously we want to avoid that. When the fix is verified,
  that commit should be cherry picked back in to the `main` branch.

We would now be in a position where we'd be looking to do a stable release and deploy to production.

### Stable Release

At this point, the release branch still exists. This process is about making a stable release from
that branch.

#### Process

* Still on the release branch, use a script to remove the `rc` pre-release identifier from the
  crates that were bumped.
* If it isn't already, the changelog should now be finalised.
* Create a `chore(release): YYYY.MM.X.Y` commit. Put the crate name and version bumps in the body of
  the commit. Any final additions to the changelog can be part of this commit.
* Create a PR for merging the release branch into `stable`.
* Once it's been merged to stable, also merge the release branch into `main`.
* When ready, kick off a workflow for the stable release. The workflow will:
    - Build the binaries
    - Upload them to S3
    - Public Github Release
    - Publish crates
    - Tag based on combined version
* Manually edit the Github Release to apply the latest changelog entry to the description
* Delete the release branch

We would now be in a position to deploy the stable release to the `PROD-01` (and possibly `PROD-02`)
environment. The production deployment would be covered in another RFC.

## Hotfixes

Hotfixes are intended to quickly fix a severe bug in a stable release. They can occur at any time
throughout the release cycle, although they'd probably more likely be near the beginning.

### Process

* Create and checkout a `hotfix-YYYY.MM.DD` branch from `stable` and push it to `origin`.
* Create an entry in the changelog that describes the fix.
* Use `release-plz update` to get new version numbers for the crates that the fix applies to.
* Use a script to apply an `rc.1` pre-release specifier to the bumped crates.
* Create a `chore(release): hotfix-YYYY.MM.DD` commit with the bumped crates and versions in the
  body of the commit.
* Fetch this branch from `upstream` to a fork and apply the fix.
* PR the commit with the fix to the `upstream` branch. This will enable someone to review it.
* If changes are requested, keep going until those are resolved.
* Use a workflow to deploy the fix to a dev/staging environment to be tested.
* When the fix is confirmed to be working:
    - Remove the `rc` pre-release specifier
    - Create a PR to merge the branch back into `stable`
* Also merge it back into `main`.
* Perform a stable release at the new version number.
* If it's a change to the node, deploy it to production using an upgrade process.
