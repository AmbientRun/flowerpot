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
  - [ ] fauna: color names white
  - [ ] fauna: make names monospace
  - [ ] fauna: center names
  - [ ] player: rubberbanding (how can this be integrated with fauna rubberbanding?)
  - [ ] fauna: add a synced `prefab_path` component
  - [ ] fauna: instantiate fauna prefabs client-side
  - [ ] fauna: copy the placeholder player model from the prototype
  - [ ] reorganize codebase into embers
  - [ ] lib: write a helper function to track the set of matching entities in each chunk
  - [ ] fauna: define a `last_chunk` component
  - [ ] fauna: when a fauna moves chunks, diff the subscribers of `in_chunk` against `last_chunk` (which are sorted; so use rapid diffing) to spawn and despawn fauna to clients, then update `last_chunk`
  - [ ] crops: deterministic crop tile angles
- [ ] UX
  - [ ] ui: deduplicate display names
  - [ ] ui: generate colors for display names?
  - [ ] ui: document messages
  - [ ] ui: add current coordinates to game HUD
  - [ ] ui: display player list in tab menu
  - [ ] ui: clean up appearance of chat window
  - [ ] ui: upper cap on chat history length
  - [ ] ui: scroll chat contents?
  - [ ] ui: add "i'm not picky" name button to randomly select nickname
  - [ ] ui: break chat system out into a standard ember
- [ ] crops
  - [ ] crops: sort out all of the medium crop assets
  - [ ] crops: define a `small_crop_class` component
  - [ ] crops: define `SpawnSmallCrops` (can update class) and `DespawnSmallCrops` messages (similar to fauna)
  - [ ] crops: spawn test irises on chunks
  - [ ] crops: reuse fauna chunk occupancy code to track small crops
  - [ ] crops: send small crop update messages on player chunk un/loading
  - [ ] crops: send small crop update messages to observers when they de/spawn
  - [ ] crops: spawn and update small crops client-side
- [ ] items and actions
  - [ ] crafting: better semantics for secondary items?
  - [ ] items: define a `model_prefab_path` component
  - [ ] game: grab some usable item models and define items for them
  - [ ] game: define some workable crafting recipes
- [ ] launch content
  - [ ] game: define prototypes for all available medium crop models
  - [ ] game: define prototypes for all medium crop-related items
- [ ] worldgen
  - [ ] make a "procgen" mod
  - [ ] procgen: figure out how to trigger chunk generation here
  - [ ] procgen: instantiate random crops on each chunk

## Pre-Playtest

big topics that still need to be planned here:
- [ ] game chat
- [ ] per-tile entity refactor
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
