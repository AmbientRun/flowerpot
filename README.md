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

- A tile-based, open world
- Chunk-based networking to enable large numbers of players to freely roam
- Perlin noise-based heightmap generation
- First-person movement and controls
- Left and right player hands that can hold items
- Planting seeds onto tiles to place crops
- Harvesting crops to obtain items
- A simple crafting system based on combining two items together
- Extensible content model using the prototype pattern
- Player-selected display names and nameplates on player avatars
- Live game text chat
- Content moderation for display names and game chat using [rustrict](https://crates.io/rustrict)
- Day/night cycle

## Bundled Content

Flowerpot also bundles free, permissively-licensed game assets for helping to
jumpstart the development of your farming game:

- **10 medium crops** with 3D models for all growth stages
- **4 trees** with 3D models for all growth stages and 4 variations
- **item models** associated with all crops

# To-Do

## Misc. Topics

- [ ] add instructions for making a mod from the template
- [ ] switch from per-tile entities to per-chunk entities
- [ ] water
- [ ] animated player character
- [ ] non-walkable tiles and movement code
- [ ] how do small crops reproduce?
- [ ] road networking and representation
- [ ] seeing items other players are holding
- [ ] crop neighbor counting and available neighbor tags
- [ ] animated item/head bobbing during walking
- [ ] held item animations
- [ ] crafting animations
- [ ] harvesting small crops
- [ ] credit use of mononoki font *somewhere*
- [ ] license the codebase appropriately
- [ ] give correct categories to embers

## Tech

- [ ] fauna: rubberbanding
- [ ] rename chunks to regions
- [ ] player: change `local_player_ref` into `is_joined`?
- [ ] add more configuration to nameplate package
- [ ] SFX
- [ ] music

## UX

- [ ] player reach
- [ ] generate colors for display names?
- [ ] document messages
- [ ] display player list in a tab menu
- [ ] scroll chat contents?
- [ ] add "i'm not picky" name button to randomly select nickname

## Crops

- [ ] define a `is_small_crop` component
- [ ] define and spawn test small crops on chunks
- [ ] cover crops
- [ ] large crops

## Items and Actions

- [ ] map items
- [ ] drop map items
- [ ] player map item targeting
- [ ] pick up map items
- [ ] crafting: more flexible pattern matching semantics for secondary items?
- [ ] items: define a `prefab_url` component
- [ ] game: grab some usable item models and define items for them
- [ ] game: define some workable crafting recipes
- [ ] player medium crop targeting
- [ ] player small crop targeting

## Worldgen

- [ ] make a "procgen" mod
- [ ] procgen: figure out how to trigger chunk generation here
- [ ] procgen: instantiate random crops on each chunk
- [ ] terrain: terrain in the shape of an island
- [ ] terrain: define a `cut_direction` component
- [ ] terrain: define a checkerboard `cut_direction` system
- [ ] terrain: generate meshes according to `cut_direction`
- [ ] terrain: calculate height according to `cut_direction`
- [ ] terrain: experiment with different `cut_direction` heuristics
- [ ] terrain: define and use a terrain seed resource
