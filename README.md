# Flowerpot

# To-Do

## MVP

big topics:
- [ ] migrate tasks from journal here (IMPORTANT)
- [ ] player reach
- [ ] player medium crop targeting
- [ ] player small crop targeting
- [ ] map items
- [ ] drop map items
- [ ] player map item targeting
- [ ] pick up map items

- [ ] tech
  - [ ] give more appropriate/distinctive ember IDs to all embers
  - [ ] break out region networking into its own ember
  - [ ] break chat and naming system out into a standard ember
- [ ] UX
  - [ ] ui: generate colors for display names?
  - [ ] ui: document messages
  - [ ] ui: display player list in tab menu
  - [ ] ui: scroll chat contents?
  - [ ] ui: add "i'm not picky" name button to randomly select nickname
- [ ] crops
  - [ ] crops: define a `is_small_crop_class` component
  - [ ] crops: define a `prefab_model_url` component
  - [ ] crops: sync small crop prefabs
  - [ ] crops: define and spawn test irises on chunks
  - [ ] crops: reuse fauna chunk occupancy code to track small crops
  - [ ] crops: spawn and update small crops client-side
- [ ] items and actions
  - [ ] crafting: better semantics for secondary items?
  - [ ] items: define a `prefab_url` component
  - [ ] game: grab some usable item models and define items for them
  - [ ] game: define some workable crafting recipes
- [ ] worldgen
  - [ ] make a "procgen" mod
  - [ ] procgen: figure out how to trigger chunk generation here
  - [ ] procgen: instantiate random crops on each chunk

## Pre-Launch

big topics that still need to be planned here:
- [ ] huge map gen
- [ ] water
- [ ] animated player character
- [ ] day/night cycle
- [ ] non-walkable tiles and movement code
- [ ] how do small crops reproduce?
- [ ] road networking and representation
- [ ] optimized diff-based crop sync?
- [ ] seeing items other players are holding
- [ ] crop neighbor counting and available neighbor tags
- [ ] animated item/head bobbing during walking
- [ ] held item animations
- [ ] crafting animations
- [ ] crop and item raycast targeting
- [ ] harvesting small crops
- [ ] SFX
- [ ] music
- [ ] credit use of mononoki font *somewhere*

- [ ] tech
  - [ ] fauna: rubberbanding
  - [ ] player: input prediction
  - [ ] per-tile entity refactor
  - [ ] rename chunks to regions
  - [ ] player: change `local_player_ref` into `is_joined`?
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
  - [ ] add more configuration to nameplate package

## Post-Launch

big topics:
- [ ] cover crops
- [ ] large crops
- [ ] soil types

- [ ] meta
  - [ ] setup CI to test code quality of PRs
  - [ ] give correct categories to embers
  - [ ] add clippy to CI
  - [ ] make panicking (i.e. use of unwrap() or expect()) a clippy error
