# Flowerpot

# To-Do

## MVP

big topics:
- [ ] migrate tasks from journal here (IMPORTANT)
- [ ] tile outline rendering
- [ ] player targeting
- [ ] player reach
- [ ] player items/crafting UX
- [ ] heightmap raycasting

- [ ] tech
  - [ ] prefix all tag components with "is_"
  - [ ] crops: deterministic crop tile angles
  - [ ] player: remove `local_player_ref` in favor of `ambient_api`'s `player::get_local()`
  - [ ] player: replace `position` with map ember's `position`
  - [ ] fauna: make names monospace
  - [ ] fauna: add a synced `prefab_path` component
  - [ ] fauna: define and sync fauna classes
  - [ ] fauna: instantiate fauna class prefabs client-side
  - [ ] fauna: copy the placeholder player model from the prototype
- [ ] UX
  - [ ] ui: deduplicate display names
  - [ ] ui: generate colors for display names?
  - [ ] ui: document messages
  - [ ] ui: add current coordinates to game HUD
  - [ ] ui: display player list in tab menu
  - [ ] ui: scroll chat contents?
  - [ ] ui: add "i'm not picky" name button to randomly select nickname
  - [ ] ui: break chat system out into a standard ember
- [ ] crops
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
- [ ] worldgen
  - [ ] make a "procgen" mod
  - [ ] procgen: figure out how to trigger chunk generation here
  - [ ] procgen: instantiate random crops on each chunk

## Pre-Launch

big topics that still need to be planned here:
- [ ] harvesting small crops
- [ ] huge map gen
- [ ] water
- [ ] day/night cycle
- [ ] non-walkable tiles and movement code
- [ ] how do small crops reproduce?
- [ ] road networking and representation
- [ ] optimized diff-based crop sync?
- [ ] seeing items other players are holding
- [ ] crop neighbor counting and available neighbor tags

- [ ] tech
  - [ ] refactor chunk storage away from per-tile entities
  - [ ] player: rubberbanding (how can this be integrated with fauna rubberbanding?)
  - [ ] per-tile entity refactor
  - [ ] rename chunks to regions
  - [ ] break out region networking into its own ember
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
- [ ] launch content
  - [ ] crops: sort out all of the medium crop assets
  - [ ] game: define prototypes for all available medium crop models
  - [ ] game: define prototypes for all medium crop-related items
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
