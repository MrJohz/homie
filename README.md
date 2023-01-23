# Homie

## TODOs

- [ ] Come up with a better name
- [ ] Allow connections from local network without token auth?
- [ ] Loading indicators on buttons (changing background?)
- [ ] Animation for input errors
- [ ] Animation for done tasks
- [ ] Handle connection/auth/etc errors more clearly
- [ ] Desktop version
- [x] Filters/sorting (and searching?)
  - [ ] But actually implement it
- [x] Logout functionality
- [ ] Some level of token expiry and deletion
- [ ] Move tasks into SQLite
- [ ] Load users from a file somewhere
- [ ] History (and undo)
- [ ] Auto build/deploy (via Docker?)

## How to build

### Building for a Raspberry Pi

This is going to depend a lot on the laptop, environment, latest versions, etc. What works right now may need adjusting in the future.

- Problems I ran into
  - The Raspberry Pi glibc version is older than the ones supported by Rust at the time of writing (I think?) therefore I needed to use musl to get this working.
  - I do not have the relevant musl headers installed on my computer (PopOS), so I needed to install them via [musl-cross-make](https://github.com/richfelker/musl-cross-make/).
  - I need to build and link C sources (sqlite), which seems to make things more complicated. I initially used the latest version of musl, which could build everything successfully, but the Rust musl target uses a [specific version](https://github.com/rust-lang/rust/blob/stable/src/ci/docker/scripts/musl.sh), and because these versions are not compatible, this breaks. Therefore, I needed to select version 1.1.24 in my `config.mak` while building musl.
  - I tried using the locally built version of musl, but while compiling `sqlite`, this seemed to run into issues. Therefore I needed to also install the finished files to `/usr/local` before everything was happy.

```bash
$ cargo build --target armv7-unknown-linux-musleabihf
# Or, takes longer but produces a much smaller binary
$ cargo build --target armv7-unknown-linux-musleabihf --release
```
