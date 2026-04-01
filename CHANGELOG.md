# CHANGELOG


## v0.3.0 (2026-04-01)

### Features

- **ci**: Reintroduce Linux and Windows wheel builds with updated configurations
  ([`b30808f`](https://github.com/krypton-byte/tryx/commit/b30808fea71d0e95fa4c9de5ae7a3b2190eb6949))


## v0.2.0 (2026-04-01)

### Bug Fixes

- Remove RELEASE_PUSH_TOKEN reference from contributing guidelines
  ([`f92bd1d`](https://github.com/krypton-byte/tryx/commit/f92bd1dedf734ca59c5a0ed45293fbe4a63f26ee))

- **release**: Update CI workflow dependencies and improve error handling for dispatch trigger
  ([`b1acd07`](https://github.com/krypton-byte/tryx/commit/b1acd07befffdb69f0dbcbb3e256bc7e0608b3fb))

- **release**: Update default branch references in workflow and documentation
  ([`bc5a5cc`](https://github.com/krypton-byte/tryx/commit/bc5a5ccc1cb037cf813998d0ad865d66c30c6ae5))

- **test**: Update workflows for commit message validation and release automation
  ([`b940d91`](https://github.com/krypton-byte/tryx/commit/b940d91d89969590af84b56a52316803174c4f65))

### Documentation

- Update contributing guidelines to clarify release automation and required secrets
  ([`cea622c`](https://github.com/krypton-byte/tryx/commit/cea622c1905aee8f461ff87625bddd18ab9b5ec3))

- Update section title in contributing guidelines for clarity
  ([`ef04047`](https://github.com/krypton-byte/tryx/commit/ef040477453e2d24ffcb384138c7d4b2250d116a))

### Features

- **test**: Test
  ([`3bc6c04`](https://github.com/krypton-byte/tryx/commit/3bc6c04f8ce84802947c45ad9e9b8735ec615918))


## v0.1.0 (2026-04-01)

### Bug Fixes

- Ensure key lifetime for builder attributes by leaking owned String
  ([`e06268c`](https://github.com/krypton-byte/tryx/commit/e06268c4169cc60dff3638a6ae769185b086b4a4))

- Specify type parameter for future_into_py_with_locals in TryxClient
  ([`7aa829c`](https://github.com/krypton-byte/tryx/commit/7aa829c635e2d0821a66470ba1b330512f5f4e3a))

- Update dependencies in Cargo.toml and refine ruff configuration in pyproject.toml
  ([`1589d3b`](https://github.com/krypton-byte/tryx/commit/1589d3bcbc2050f764211c20c06b52926946a455))

- **CI**: Aarch64 build
  ([`9831606`](https://github.com/krypton-byte/tryx/commit/9831606fe7a5c6db6b5d2e09abebdf1ad79aa2ab))

- **CI**: Update check_stub_parity command to include --no-project flag and remove unnecessary
  targets
  ([`dda9bd9`](https://github.com/krypton-byte/tryx/commit/dda9bd9e9709d20aaded7fb36638e4d87a2d4e5f))

- **CI**: Update validate stub parity command to remove --no-project flag
  ([`237b082`](https://github.com/krypton-byte/tryx/commit/237b082c9bbba6ad276935ff1db9c1fe49831272))

### Chores

- Update CI workflows to use recursive submodule checkout and fix submodule URL
  ([`68cd99c`](https://github.com/krypton-byte/tryx/commit/68cd99c99d9202094c6a3bff7821e8706df8ccd3))

- Update release workflow to initialize and update git submodules
  ([`f9b11e8`](https://github.com/krypton-byte/tryx/commit/f9b11e83f10a8a366cf38c14c9f2a6374e263d0a))

- Update ruff commands to include version and no-project flag in CI and documentation
  ([`af0bd26`](https://github.com/krypton-byte/tryx/commit/af0bd260c7fbbc591afe6444a2a3758ead406b20))

- Update subproject commit for whatsapp-rust dependency
  ([`246ec16`](https://github.com/krypton-byte/tryx/commit/246ec1619d4973b4a4d682b524b44c84a9e0e44e))

- Update whatsapp-rust subproject to latest commit
  ([`3fcfb4f`](https://github.com/krypton-byte/tryx/commit/3fcfb4f965672f0da3f563b6cb00ab5d05ea24d2))

### Features

- Add chat actions client and centralized proto cache for improved performance and organization
  ([`a860bb2`](https://github.com/krypton-byte/tryx/commit/a860bb274b5a41c95f920df60a9a0dae8ad63242))

- Add ChatActionsClient with methods for managing chat actions and update documentation
  ([`7503411`](https://github.com/krypton-byte/tryx/commit/7503411aa5659f9325db882d9b2633efdaf15009))

- Add chatstate, blocking, polls, and presence namespaces and clients
  ([`d650a8a`](https://github.com/krypton-byte/tryx/commit/d650a8a8adfff77d0dc722271a4611e9ce6b7ae7))

- Introduced `ChatstateClient` for managing chat states (composing, recording, paused). - Added
  `BlockingClient` to handle blocking and unblock users, and manage blocklists. - Implemented
  `PollsClient` for creating polls, voting, and aggregating votes. - Created `PresenceClient` to
  manage user presence status (available, unavailable). - Updated README.md to document new
  namespaces and client functionalities. - Added corresponding helper classes for chatstate,
  blocking, polls, and presence. - Defined new types: `ChatStateType`, `BlocklistEntry`,
  `PollOptionResult`, and `PresenceStatus`. - Refactored existing code to integrate new features and
  ensure compatibility.

- Add command bot example with support for various commands and profile handling
  ([`aa7553d`](https://github.com/krypton-byte/tryx/commit/aa7553d65c7127aa7c1391c80aeb2084021157bd))

- Add delete chat update event and related structures in event handling
  ([`4e388fb`](https://github.com/krypton-byte/tryx/commit/4e388fb081af286d2c088283d2ab0eed4871db89))

- Add delete message for me update event and related structures in event handling
  ([`7f68d51`](https://github.com/krypton-byte/tryx/commit/7f68d5175819d75876c65b74d2dc031f00d45266))

- Add event dispatcher for handling WhatsApp events
  ([`83ef1f2`](https://github.com/krypton-byte/tryx/commit/83ef1f2d57e362d74a9430c46284e3a396f6a7ab))

- Implemented a Dispatcher class to manage event handlers for various WhatsApp events. - Added event
  types including Connected, Disconnected, LoggedOut, PairSuccess, and others. - Introduced methods
  for registering and retrieving event handlers. - Created a mapping from Python event classes to
  internal dispatcher events. - Added support for event types in the types module, including message
  handling and connection failures. - Implemented logging initialization for better traceability.

- Add event types for profile synchronization and updates
  ([`fb4d136`](https://github.com/krypton-byte/tryx/commit/fb4d13694b4caf194d8e924a4c0f70078d5f5108))

- Introduced EvPushNameUpdate and EvSelfPushNameUpdated for push name updates. - Added EvPinUpdate
  and PinUpdatedata for pin updates. - Implemented EvMuteUpdate and MuteUpdateData for mute updates.
  - Created EvMarkChatAsReadUpdate and MarkChatAsReadUpdateData for marking chats as read. - Added
  EvHistorySync for handling history synchronization events. - Introduced EvOfflineSyncPreview and
  EvOfflineSyncCompleted for offline sync events. - Implemented EvDeviceListUpdate for device list
  updates with associated data. - Added EvBusinessStatusUpdate for business status updates with
  detailed information.

- Add get_client method to Tryx for accessing TryxClient
  ([`19c1bb6`](https://github.com/krypton-byte/tryx/commit/19c1bb65b93b671a2aceb40d23fec32118b20a57))

- Add get_info method to TryxClient for retrieving contact information
  ([`a8c8246`](https://github.com/krypton-byte/tryx/commit/a8c82460ce57933000d4d1f7ff733499c9f4c76e))

- Add Groups and Status clients with associated helpers and methods
  ([`e3d0e5c`](https://github.com/krypton-byte/tryx/commit/e3d0e5c954443b45101c865b9fefbcc0e651285e))

- Implemented GroupsClient to manage group-related functionalities including creating, modifying,
  and querying group information. - Added StatusClient to handle status-related operations. -
  Introduced GroupsHelpers and StatusHelpers for constructing options and managing privacy settings.
  - Updated TryxClient to include new GroupsClient and StatusClient. - Enhanced the module structure
  to accommodate new group and status functionalities. - Added necessary enums and data structures
  for group management and status privacy settings.

- Add new event structures for OfflineSyncCompleted and DeviceListUpdate, and introduce KeyIndexInfo
  ([`5f57fec`](https://github.com/krypton-byte/tryx/commit/5f57fec6247bb2b04b86d0c29f504b6922e6cfaa))

- Add new event types and handlers for disappearing mode, contact updates, and star updates
  ([`7f1b4d1`](https://github.com/krypton-byte/tryx/commit/7f1b4d1fc9b41c3aa1b0eb83524c82f28708e076))

- Add newsletter live update event and related structures
  ([`ef7cd71`](https://github.com/krypton-byte/tryx/commit/ef7cd71b540db027f8ca2bc27452355d1842d6a7))

- Add NewsletterClient and related models for newsletter management
  ([`ce91c3e`](https://github.com/krypton-byte/tryx/commit/ce91c3eeab8ccb4537a165b85e39d240d019b901))

- Add ProfilePicture struct and get_profile_picture method in TryxClient
  ([`f0c3b0b`](https://github.com/krypton-byte/tryx/commit/f0c3b0b936ef2966f9f9a4941ee5c5e54ec5ef46))

- Add prost dependency and enhance message handling in types
  ([`a6fcba9`](https://github.com/krypton-byte/tryx/commit/a6fcba985e4bf3a436ca6c1ddadab56860f992f0))

- Add send_image and send_text methods to TryxClient for media messaging
  ([`94392f6`](https://github.com/krypton-byte/tryx/commit/94392f63eccba38def0eef73e8d02529349d79e4))

- Add support for connected event handling and custom exceptions
  ([`a020337`](https://github.com/krypton-byte/tryx/commit/a02033716cc43ebdcd94f5e13f6938628f59a263))

- Add upload functionality and MediaType enum for media handling
  ([`b22e72e`](https://github.com/krypton-byte/tryx/commit/b22e72ee97ec46e6163228508f28cf934164827c))

- Add whatsapp-rust submodule for enhanced messaging capabilities
  ([`7a0f1f6`](https://github.com/krypton-byte/tryx/commit/7a0f1f69ae8b6328f8283ead072183e4083f3af9))

- Enhance API documentation and add new features
  ([`eb61115`](https://github.com/krypton-byte/tryx/commit/eb611152411de85f8e617f9d97a879eb57581946))

- Added detailed documentation for Client API, Events API, Helpers API, and Types API. - Introduced
  new tutorials for command-based bots, media workflows, and group automation. - Implemented a
  performance guide and security practices documentation. - Established a troubleshooting section to
  assist users in resolving common issues. - Created a changelog policy and error handling
  guidelines for better maintenance. - Added a script to check parity between runtime classes and
  .pyi stubs. - Updated CSS styles for improved documentation aesthetics. - Refactored types in
  `src/types.rs` to be publicly accessible for better integration.

- Enhance bot functionality with message sending capability and state management
  ([`6d28bb4`](https://github.com/krypton-byte/tryx/commit/6d28bb45418c63c9e96aa898f205073e63b6dd5d))

- Enhance event handling and add IsOnWhatsApp functionality with JID updates
  ([`f5bdd8f`](https://github.com/krypton-byte/tryx/commit/f5bdd8ff30368e186cdccf6d228a1239d5015f7f))

- Enhance event handling by adding detailed payloads for ConnectFailure and StreamError events
  ([`9d4f447`](https://github.com/krypton-byte/tryx/commit/9d4f447770010472ad5a3550557812b4d1129a65))

- Enhance event handling by adding EvPairSuccess and EvReceipt types, and refactor callback
  execution
  ([`05bb8f6`](https://github.com/krypton-byte/tryx/commit/05bb8f6a2e6197824c105107a4705833eea68ab6))

- Enhance event handling by adding new event types and refactoring existing logic
  ([`f160800`](https://github.com/krypton-byte/tryx/commit/f160800a084724d367c4aee54bddc3cb8159b72a))

- Enhance event handling by adding new structures for GroupUpdate and ContactUpdate events
  ([`7f4fa3d`](https://github.com/krypton-byte/tryx/commit/7f4fa3d9d13f6997563288f3215442a18e09f8ef))

- Enhance event handling by adding new structures for MuteUpdate and MarkChatAsReadUpdate events
  ([`257acb7`](https://github.com/krypton-byte/tryx/commit/257acb7adcceafe7529131bd16d65f2dfee69a8a))

- Enhance event handling by introducing new event structures and improving node handling
  ([`8eb0513`](https://github.com/krypton-byte/tryx/commit/8eb05137157b3cbecd7a69ba8ce35742bb633fe4))

- Enhance event handling for JoinedGroup event and add LazyConversation structure
  ([`cb9f345`](https://github.com/krypton-byte/tryx/commit/cb9f3451a09a885e9a74a60be74db688f98c73d7))

- Enhance JID struct with conversion implementations and update MessageSource getters
  ([`a8ef461`](https://github.com/krypton-byte/tryx/commit/a8ef4612633f458d19b11179ef4ec93dc8733d33))

- Enhance message handling and add send_message functionality in TryxClient
  ([`439137e`](https://github.com/krypton-byte/tryx/commit/439137ef058bade139463aabea24cce782fe88c7))

- Enhance messaging capabilities with new SendResult and MediaReuploadResult types
  ([`05c32b0`](https://github.com/krypton-byte/tryx/commit/05c32b0ba841d501643d60c193c8275a40fdde53))

- Enhance Tokio integration and add tracing support
  ([`85129f8`](https://github.com/krypton-byte/tryx/commit/85129f8e7cd70fd56c9ebbf78bc6a8578b00d2f6))

- Updated `Cargo.toml` to include the `signal` feature for Tokio and added `tracing` and
  `tracing-subscriber` dependencies for better logging. - Introduced a new example script
  `examples.py` demonstrating the usage of the Tryx client with message handling. - Modified
  `pyproject.toml` to include `segno` as a dependency for QR code generation. - Refactored
  `__init__.py` to expose new classes: `JID`, `Message`, `MessageInfo`, `PairingQrCode`, and
  `SqliteBackend`. - Added type hints for new classes in `tryx.pyi` to improve type checking. -
  Enhanced `backend.rs` to include logging for SQLite backend connection attempts. - Refactored
  `client.rs` to improve event handling and logging, including support for graceful shutdown on
  signal interrupts. - Updated `events.rs` to include new event handlers and logging for registered
  callbacks. - Enhanced `lib.rs` to expose new types to Python. - Updated `types.rs` to include
  necessary fields and derive traits for new types. - Updated `uv.lock` to include new dependencies
  and ensure compatibility with Python versions.

- Enhance Tryx SDK with new event structures, user info retrieval, and profile picture handling
  ([`b1d8c97`](https://github.com/krypton-byte/tryx/commit/b1d8c977b65e89c1d93f2f14be3cb137cbc4cb2c))

- Extend event types with EvPairError and EvPairingCode, and update related logic
  ([`3ecb271`](https://github.com/krypton-byte/tryx/commit/3ecb271ab37897f9b8dd4e1d61ac1c1768685b0b))

- Implement media download functionality and enhance TryxClient methods
  ([`ecb828e`](https://github.com/krypton-byte/tryx/commit/ecb828edba1b17012d11d6629ab45045b7067602))

- Implement privacy client and related models for privacy management
  ([`36c3154`](https://github.com/krypton-byte/tryx/commit/36c31546312f9cf1f10b415ef901384e6a6901a1))

- Initialize new Rust project "karat" with Python bindings
  ([`82054d5`](https://github.com/krypton-byte/tryx/commit/82054d534d81fa2c021e290bef028f9055e7cd63))

- Add Cargo.toml for Rust package configuration - Create pyproject.toml for Python packaging with
  maturin - Implement backend module with Sqlite and Postgres backends - Develop client module to
  handle WhatsApp bot interactions - Define event types for WhatsApp messaging - Create types module
  for JID and message handling - Set up main entry point for bot execution - Add necessary
  dependencies for async and database operations

- Introduce CommunityClient and related models for community management
  ([`159b51a`](https://github.com/krypton-byte/tryx/commit/159b51a48dc0d19bbb9f2b92c80347395263fff3))

- Introduce ContactClient for managing contact-related operations and enhance TryxClient with
  contact functionalities
  ([`b369f2e`](https://github.com/krypton-byte/tryx/commit/b369f2ebd009f190edb066bb86e185bbe680b2f7))

- Refactor event structures for PairSuccess and BusinessStatusUpdate, and add BusinessSubscription
  support
  ([`6c94339`](https://github.com/krypton-byte/tryx/commit/6c94339115d274e8587545230fdc916ca97601bf))

- Refactor MediaType handling and introduce Node and NodeContent structures
  ([`7bdbb2e`](https://github.com/krypton-byte/tryx/commit/7bdbb2e2a041366807dab255f4a9e7643238ad06))

- Refactor MessageData and EvMessage to include MessageInfo
  ([`7b86c83`](https://github.com/krypton-byte/tryx/commit/7b86c8305bee0e9ad4318b48e7394b8c0854966e))

- Refactor Node and NodeContent structures for improved type handling and memory efficiency
  ([`983c460`](https://github.com/krypton-byte/tryx/commit/983c4605aae0a64ab17e07cd744d7c0ba4a52b26))

- Update clients to handle message IDs and improve default cases for unknown states
  ([`d0db752`](https://github.com/krypton-byte/tryx/commit/d0db75286c5f2719a86b298857f165bbb8635d7e))

- Update dependencies and improve event handling in Tryx client
  ([`5fef31d`](https://github.com/krypton-byte/tryx/commit/5fef31d504f9bc27fc6c710d947b4ed6631c7ddd))

- **templates**: Add issue and PR templates for better contribution guidelines
  ([`5d82877`](https://github.com/krypton-byte/tryx/commit/5d828773b85bb86575408f3eaedc79b512d00453))

feat(workflows): implement commit message validation and semantic release

docs(CONTRIBUTING): create contributing guide with setup and commit message standards

docs(changelog): add changelog for tracking project changes

feat(pre-commit): configure pre-commit hooks for code quality checks

feat(docs): set up documentation build and deployment workflows

### Refactoring

- Change enum visibility to public in message and updates, and profile sync modules
  ([`20f94b1`](https://github.com/krypton-byte/tryx/commit/20f94b1c7d1bf8568b2f46b0ce3de13e122e356d))

- Clean up Message class and enhance MessageInfo structure
  ([`5d0d014`](https://github.com/krypton-byte/tryx/commit/5d0d01462a2b772a3faad17783048d4c1ebee461))

- Clean up unused code and improve import formatting
  ([`8f54c8b`](https://github.com/krypton-byte/tryx/commit/8f54c8b6adfb5561716468985c7f068d038f888b))

- Improve logging format in command bot and enhance exception handling for unsupported types
  ([`b84476f`](https://github.com/krypton-byte/tryx/commit/b84476f7eceee94d431b3505b4a6ffe36e4ca27d))

- Simplify event dispatching and improve attribute handling in Node
  ([`ad4a3ea`](https://github.com/krypton-byte/tryx/commit/ad4a3eafbf0cf7ef24a1e7e46acfff623109e0c5))
