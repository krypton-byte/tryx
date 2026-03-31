# Helpers API

Helpers are available under `tryx.helpers`.

## NewsletterHelpers

- parse binary message payload into protobuf message
- serialize protobuf message back to bytes
- build text-only message payload quickly

## GroupsHelpers

- normalize invite code/url
- build `GroupParticipantOptions`
- build `CreateGroupOptions`

## StatusHelpers

- build status send options
- get default privacy

## ChatstateHelpers

- `composing()`
- `recording()`
- `paused()`

## BlockingHelpers

- compare two JIDs and determine if they point to the same user

## PollsHelpers

- decrypt encrypted vote payload
- aggregate vote state into `PollOptionResult`

## PresenceHelpers

- return default presence status

## Design Goal

Helpers are intentionally stateless and deterministic to make testing simple.
