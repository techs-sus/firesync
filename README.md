# firesync

firesync is a command line program designed to serve bundled files

## Deprecation notice

You should not use this in new work! Use [roblox-ts-with-azalea](https://github.com/techs-sus/roblox-ts-with-azalea) instead.

## how?

- Bundle with darklua
- Replace ns, nls calls with full_moon (require)
- Serve those files

### notes / todo
- you must use .client.lua, .lua, .server.lua suffixes
- everything will be bundled unless you "focus" on a single file
- should we enforce .client.lua + .server.lua for NLS and NS?
