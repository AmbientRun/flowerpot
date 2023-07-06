# Flowerpot

# To-Do

- [ ] outline this README
- [ ] license the codebase appropriately
- [ ] migrate tasks from journal here
- [ ] setup CI to test code quality of PRs
- [ ] add clippy to CI
- [ ] make panicking (i.e. use of unwrap() or expect()) a clippy error
- [x] create a "map" mod
- [x] map: write map component and message schemas using chunks
- [x] map: generate a basic, fixed-size map of chunks server-side
- [x] map: synchronize all (for now) chunks with all players
- [x] map: spawn client-side chunk and tile entities in response to updates
- [x] create a "player" mod
- [x] player: construct the provided player entity with transform, avatar, and camera
- [x] player: define mouselook messages and send them on input
- [x] player: define shared movement code and run it client-side
- [x] player: define movement messages and send them from the client
- [ ] make a "fauna" mod
- [ ] fauna: define a fauna tag component
- [ ] fauna: define fauna position and angle components + update messages
- [ ] player: make the player a fauna server-side
- [ ] player: run player movement server-side
- [ ] fauna: define fauna spawn, despawn, and update messages
- [ ] fauna: send fauna update messages for all fauna to all clients server-side
- [ ] fauna: spawn and update puppeted fauna entities client-side
- [ ] player: rubberbanding (how can this be integrated with fauna rubberbanding?)
- [ ] map: add a list component of subscribed clients (player entity IDs; user IDs for message-sending can be retrieved) to chunks
- [ ] map: keep all chunk subscriptions updated with all clients
- [ ] map: define messages for un/subscribing specific clients to specific chunks
- [ ] map: listen to subscription messages and update chunks accordingly (keep values sorted for rapid diffing)
- [ ] fauna: define an `in_chunk` component and update all fauna with it
- [ ] fauna: define an `last_chunk` component
- [ ] fauna: when a fauna moves chunks, diff the subscribers `in_chunk` against `last_chunk` (which are sorted; so use rapid diffing) to spawn and despawn fauna to clients, then update `last_chunk`

Big topics that stil need to be planned:
- [ ] render distance
- [ ] soil types
- [ ] crop definitions, growth, and syncing
- [ ] procedural generation and heightmaps
- [ ] a base content mod
- [ ] roads and foundations
- [ ] items and crafting
- [ ] fauna display names
- [ ] player sets display name on join
- [ ] game chat
