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
- [ ] make a "core" mod
- [ ] core: port the prototype's entity macros
- [ ] core: spawn base content server-side
- [ ] core: define prototypes for all available crop models
- [ ] crops: define `seed_ref` and `next_growth_stage` components
- [ ] crops: implement crop growth
- [ ] player: define left and right hand components
- [ ] player: initialize left and right hand children for local player
- [ ] make an "items" mod
- [ ] items: define `held_ref` component
- [ ] items: spawn held item models in hands
- [ ] items: hold debug items in local player's hands
- [ ] items: define crafting recipe-related components
- [ ] items: port over a shared version of the prototype's crafting components and store
- [ ] items: define and send a crafting input message
- [ ] items: run crafting client-side
- [ ] items: define fauna update messages for held items
- [ ] items: respond to crafting inputs server-side
- [ ] items: respond to held item updates client-side (no-op on identical items)
- [ ] crops: deterministic crop tile angles

Chunk un/loading:
- [x] map: define a list component of subscribed clients (player entity IDs; user IDs for message-sending can be retrieved) to chunks
- [x] map: keep all chunk subscriptions updated with all clients
- [x] map: define event messages on un/subscribing of specific clients to specific chunks
- [x] map: listen to subscription messages and update chunks accordingly (keep values sorted for rapid diffing)
- [x] Move all chunk subscriptions from map mod to player mod
- [x] player: define a component to track each player's current list of loaded chunks
- [x] player: every frame, recompute every player's render distance, diff against loaded chunks, emit un/load events, and update the list
- [x] terrain: despawn client-side meshes for unloaded chunks
- [x] crops: update `UpdateMediumCrops` using chunk subscriptions
- [ ] lib: write a helper function to track the set of matching entities in each chunk
- [ ] fauna: define a `last_chunk` component
- [ ] fauna: when a fauna moves chunks, diff the subscribers of `in_chunk` against `last_chunk` (which are sorted; so use rapid diffing) to spawn and despawn fauna to clients, then update `last_chunk`

Big topics that stil need to be planned:
- [ ] better heuristics for player chunk loading?
- [ ] fauna models
- [ ] player avatars
- [ ] soil types
- [ ] day/night cycle
- [ ] optimized diff-based crop sync?
- [ ] configurable crop growth tick num before stage change
- [ ] crop neighbor counting and available neighbor tags
- [ ] cover crops
- [ ] small crops
- [ ] large crops
- [ ] procedural world generation
- [ ] water
- [ ] non-walkable tiles and movement code
- [ ] fences
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
