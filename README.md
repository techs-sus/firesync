# firesync

firesync is a command line program designed to serve bundled files

## how?

- Bundle with darklua
- Replace ns, nls calls with full_moon (require)
- Serve those files

### notes / todo
- you must use .client.lua, .lua, .server.lua suffixes
- everything will be bundled unless you "focus" on a single file
- should we enforce .client.lua + .server.lua for NLS and NS?
