# Flowerpot

# To-Do

## MVP

big topics:
- [ ] migrate tasks from journal here (IMPORTANT)
- [ ] tile outline rendering
- [ ] player targeting
- [ ] player reach
- [ ] configurable crop growth tick num before stage change
- [ ] crop neighbor counting and available neighbor tags
- [ ] player items/crafting UX
- [ ] heightmap raycasting
- [ ] cancel large crops before launch?

- [ ] tech
  - [ ] player: rubberbanding (how can this be integrated with fauna rubberbanding?)
  - [ ] lib: write a helper function to track the set of matching entities in each chunk
  - [ ] fauna: define a `last_chunk` component
  - [ ] fauna: when a fauna moves chunks, diff the subscribers of `in_chunk` against `last_chunk` (which are sorted; so use rapid diffing) to spawn and despawn fauna to clients, then update `last_chunk`
  - [ ] crops: deterministic crop tile angles
  - [ ] fauna: add a synced `prefab_path` component
  - [ ] fauna: instantiate fauna prefabs client-side
  - [ ] fauna: copy the placeholder player model from the prototype
- [ ] crops
  - [ ] crops: define `seed_ref` and `next_growth_stage` components
  - [ ] crops: implement crop growth and better spread
  - [ ] crops: define a `small_crop_class` component
  - [ ] crops: define `SpawnSmallCrops` (can update class) and `DespawnSmallCrops` messages (similar to fauna)
  - [ ] crops: spawn test irises on chunks
  - [ ] crops: reuse fauna chunk occupancy code to track small crops
  - [ ] crops: send small crop update messages on player chunk un/loading
  - [ ] crops: send small crop update messages to observers when they de/spawn
  - [ ] crops: spawn and update small crops client-side
- [ ] worldgen
  - [ ] make a "procgen" mod
  - [ ] procgen: figure out how to trigger chunk generation here
  - [ ] procgen: instantiate random crops on each chunk
  - [ ] terrain: define a `cut_direction` component
  - [ ] terrain: define a checkerboard `cut_direction` system
  - [ ] terrain: generate meshes according to `cut_direction`
  - [ ] terrain: calculate height according to `cut_direction`
  - [ ] terrain: experiment with different `cut_direction` heuristics
  - [ ] terrain: define and use a terrain seed resource
- [ ] items
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
- [ ] launch content
  - [x] make a "game" mod
  - [ ] game: port the prototype's entity macros
  - [ ] game: spawn base content server-side
  - [ ] game: define prototypes for all available crop models

## Pre-Playtest

big topics that still need to be planned here:
- [ ] water
- [ ] day/night cycle
- [ ] non-walkable tiles and movement code
- [ ] per-tile entity refactor
- [ ] how do small crops reproduce?
- [ ] harvesting small crops
- [ ] road networking and representation
- [ ] fauna display names
- [ ] player sets display name on join
- [ ] optimized diff-based crop sync?
- [ ] game chat

- [ ] tech
  - [ ] refactor chunk storage away from per-tile entities
- [ ] worldgen
  - [ ] terrain: terrain in the shape of an island

## Pre-Launch

big topics:
- [ ] fences
- [ ] foundation data representation
- [ ] foundation UX
- [ ] workshop data rep
- [ ] workshop placement UX
- [ ] workshop usage UX

- [ ] meta
  - [ ] outline this README
  - [ ] add instructions for making a mod from the template
  - [ ] license the codebase appropriately

## Post-Launch

big topics:
- [ ] cover crops
- [ ] large crops
- [ ] soil types

- [ ] meta
  - [ ] setup CI to test code quality of PRs
  - [ ] add clippy to CI
  - [ ] make panicking (i.e. use of unwrap() or expect()) a clippy error
