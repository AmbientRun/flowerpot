# Flowerpot

# To-Do

- [ ] outline this README
- [ ] add instructions for making a mod from the template
- [ ] license the codebase appropriately
- [ ] migrate tasks from journal here
- [ ] setup CI to test code quality of PRs
- [ ] add clippy to CI
- [ ] make panicking (i.e. use of unwrap() or expect()) a clippy error
- [ ] player: rubberbanding (how can this be integrated with fauna rubberbanding?)
- [ ] terrain: define a `cut_direction` component
- [ ] terrain: experiment with different `cut_direction` heuristics
- [ ] terrain: define and use a terrain seed resource
- [ ] fauna: position fauna on the map using their chunk's terrain
- [ ] make a "crops" mod
- [ ] crops: define cover, small, medium, and large tag components
- [ ] crops: define a `medium_crop_occupant` tile component
- [ ] crops: define a `class_ref` crop component
- [ ] crops: spawn a dummy medium crop prototype and instantiate it to test
- [ ] crops: define a `UpdateMediumCrops` chunk message
- [ ] map: add `in_chunk` to all chunk tiles server- and client-side
- [ ] crops: update newly-connected clients with full `UpdateMediumCrops` messages
- [ ] crops: broadcast `UpdateMediumCrops` in `change_query` on `medium_crop_occupant`
- [ ] crops: update `medium_crop_occupant` from `UpdateMediumCrops` client-side
- [ ] crops: draw placeholder medium crops for `medium_crop_occupant`
- [ ] crops: despawn client-side occupants of despawning tiles
- [ ] crops: define a `GrowTick` message
- [ ] map: server-side tile neighbors?
- [ ] crops: subscribe to `GrowTick` and just duplicate class IDs to neighbors
- [ ] make a "core" mod
- [ ] core: port the prototype's entity macros
- [ ] core: spawn base content server-side
- [ ] make an "items" mod
- [ ] items: define crafting recipe-related components
- [ ] items: port over the prototype's crafting store

Chunk un/loading:
- [ ] map: add a list component of subscribed clients (player entity IDs; user IDs for message-sending can be retrieved) to chunks
- [ ] map: keep all chunk subscriptions updated with all clients
- [ ] map: define messages for un/subscribing specific clients to specific chunks
- [ ] map: listen to subscription messages and update chunks accordingly (keep values sorted for rapid diffing)
- [ ] fauna: define a `last_chunk` component
- [ ] fauna: when a fauna moves chunks, diff the subscribers of `in_chunk` against `last_chunk` (which are sorted; so use rapid diffing) to spawn and despawn fauna to clients, then update `last_chunk`
- [ ] crops: send `UpdateMediumCrops` when players load chunks
- [ ] crops: update `UpdateMediumCrops` using chunk subscriptions

Big topics that stil need to be planned:
- [ ] render distance
- [ ] fauna models
- [ ] player avatars
- [ ] soil types
- [ ] day/night cycle
- [ ] optimized crop sync?
- [ ] configurable crop growth tick num before stage change
- [ ] crop neighbor counting and available neighbor tags
- [ ] cover crops
- [ ] small crops
- [ ] large crops
- [ ] road networking and representation
- [ ] player items/crafting UX
- [ ] fauna display names
- [ ] player sets display name on join
- [ ] game chat
- [ ] heightmap raycasting
- [ ] player targeting
- [ ] player reach
- [ ] tile outline rendering
- [ ] foundation data representation
- [ ] foundation UX
- [ ] workshop data rep
- [ ] workshop placement UX
- [ ] workshop usage UX
