# Flowerpot

# To-Do

## MVP

big topics:
- [ ] migrate tasks from journal here (IMPORTANT)
- [ ] tile outline rendering
- [ ] player targeting
- [ ] player reach
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
  - [x] crops: define `seed` and `next_growth_stage` components
  - [x] crops: implement crop growth and better spread
  - [x] crops: replace `GrowTick` with `age`
  - [ ] crops: sort out all of the medium crop assets
  - [ ] crops: define a `small_crop_class` component
  - [ ] crops: define `SpawnSmallCrops` (can update class) and `DespawnSmallCrops` messages (similar to fauna)
  - [ ] crops: spawn test irises on chunks
  - [ ] crops: reuse fauna chunk occupancy code to track small crops
  - [ ] crops: send small crop update messages on player chunk un/loading
  - [ ] crops: send small crop update messages to observers when they de/spawn
  - [ ] crops: spawn and update small crops client-side
- [ ] items and actions
  - [x] player: define left and right hand components
  - [x] player: initialize left and right hand children for local player
  - [x] make an "items" mod
  - [x] items: define `class` component
  - [x] items: define `held_ref` component
  - [x] items: define server-to-client held item update messages
  - [x] items: send held item update messages on `held_ref` change
  - [x] items: spawn held item models in hands client-side
  - [x] game: define and spawn debug items in all player's hands
  - [x] make an "actions" mod
  - [x] actions: design an action key data type
  - [x] actions: define `RegisterCraftingAction` and `OnCraftingAction` messages
  - [x] actions: create a singleton actions store
  - [x] actions: subscribe to `RegisterCraftingAction`
  - [x] actions: define a `PerformAction` client-to-server message
  - [x] actions: subscribe to `PerformAction` and dispatch `OnCraftingAction`
  - [x] actions: bind `PerformAction` to key
  - [x] make a "crafting" mod
  - [x] crafting: define crafting recipe components
  - [x] crafting: do a `spawn_query` for crafting recipes and register crafting actions
  - [x] crafting: subscribe to `OnCraftingAction` and perform player crafting
  - [ ] crafting: better semantics for secondary items?
  - [ ] items: define a `model_prefab_path` component
  - [ ] game: grab some usable item models and define items for them
  - [ ] game: define some workable crafting recipes
- [ ] launch content
  - [x] make a "game" mod
  - [x] game: port the prototype's entity macros
  - [x] game: define some basic crop prototypes
  - [x] game: spawn beeeeaaaaannnnns
  - [ ] game: define prototypes for all available medium crop models
  - [ ] game: define prototypes for all medium crop-related items
- [ ] worldgen
  - [ ] make a "procgen" mod
  - [ ] procgen: figure out how to trigger chunk generation here
  - [ ] procgen: instantiate random crops on each chunk

## Pre-Playtest

big topics that still need to be planned here:
- [ ] per-tile entity refactor
- [ ] fauna display names
- [ ] player sets display name on join
- [ ] game chat
- [ ] harvesting small crops
- [ ] huge map gen
- [ ] water
- [ ] day/night cycle
- [ ] non-walkable tiles and movement code
- [ ] how do small crops reproduce?
- [ ] road networking and representation
- [ ] optimized diff-based crop sync?
- [ ] seeing items other players are holding

- [ ] tech
  - [ ] refactor chunk storage away from per-tile entities
- [ ] items and actions
  - [ ] crafting: comprehensively document components
- [ ] worldgen
  - [ ] terrain: terrain in the shape of an island
  - [ ] terrain: define a `cut_direction` component
  - [ ] terrain: define a checkerboard `cut_direction` system
  - [ ] terrain: generate meshes according to `cut_direction`
  - [ ] terrain: calculate height according to `cut_direction`
  - [ ] terrain: experiment with different `cut_direction` heuristics
  - [ ] terrain: define and use a terrain seed resource

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
