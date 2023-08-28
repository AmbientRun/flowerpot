# Flowerpot

Hello and welcome to Flowerpot's source code repository on GitHub!

Flowerpot is a farming game framework for the multiplayer game engine
[Ambient](https://ambient.run). Using Flowerpot and Ambient, game developers
have a strong foundation for building multiplayer farming games that involve
large numbers of simultaneous players and vast, procedurally-generated worlds.

Flowerpot is not only a game framework but a standard interface for Ambient
farming games. A goal of Flowerpot is to support extensive modding. Games that
are built on top of Flowerpot will be able to freely interoperate, enabling
players to mix and match their favorite Flowerpot games together based on what
kind of experience they're looking for.

Flowerpot is also built as a general example for all Ambient game developers
who need a large, functioning game built in Ambient that they may reference for
their own creations. Care is taken to make Flowerpot's code as modular and
simple as possible to maximize its reusability. Additionally, Flowerpot
maintains a small collection of general-purpose Ambient packages that can be
used in other non-farming Ambient games.

## Features

- [x] An efficient, tile-based, open world
- [x] Chunk-based networking to enable large numbers of players to freely roam
- [x] Perlin noise-based heightmap generation
- [x] First-person movement and controls
- [x] Left and right player hands that can hold items
- [ ] Dropping and picking of items on the map
- [x] Planting seeds onto tiles to place crops
- [x] Harvesting crops to obtain items
- [x] A simple crafting system based on combining two items together
- [x] Extensible content model using the prototype pattern
- [ ] Animated player avatars
- [x] Player-selected display names and nameplates on player avatars
- [x] Live game text chat
- [x] Content moderation for display names and game chat using [rustrict](https://crates.io/rustrict)
- [x] Day/night cycle
- [ ] Sound effects
- [ ] Small crops
- [x] Medium crops
- [ ] Large crops

## Bundled Content

Flowerpot also bundles free, permissively-licensed game assets for helping to
jumpstart the development of your farming game:

TODO: update after launch
- [ ] **10 medium crops** with 3D models for all growth stages
- [ ] **item models** associated with all crops

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
