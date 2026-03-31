# Glossary

## JID
A WhatsApp identifier containing user and server segments.

## LID
An alternate addressing identity used in some multi-device contexts.

## Pairing
Initial account linking flow between runtime and WhatsApp account.

## Sync Action
State update events propagated from server/app-state (mute, archive, delete, etc.).

## EvMessage
Main incoming message event class.

## MessageInfo
Metadata wrapper containing source, timestamps, message ID, and related flags.

## Newsletter
Channel-like broadcast construct exposed through `client.newsletter`.

## Dispatcher
Internal callback registry mapping event classes to Python functions.

## WACore
Low-level protocol-oriented module exposing node/stanza structures.

## Media Reupload
Workflow to refresh media retrieval paths for expired media references.
