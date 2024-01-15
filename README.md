# Making Rust and OpenGL kiss

[<img alt="github" src="https://img.shields.io/badge/github-holothedrunk/glrs-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/HoloTheDrunk/glrs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/paste.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/glrs)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-glrs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/glrs)

Passing data between Rust and OpenGL is error-prone and mistakes can be hard to debug.

This crate provides compile-time conversion of GLSL constructs to Rust based on the [glam] crate.

[glam]: https://github.com/bitshifter/glam-rs

```toml
[dependencies]
glrs = "0.1"
```

Only tested on rustc 1.75.

# Roadmap

- [ ] Layout aware importing
  - [ ] Add padding bytes according to GLSL spec's defined layout modes
- [ ] Structs
  - [x] Basic struct importing
  - [ ] Nested struct importing?
- [ ] Uniforms
  - [ ] Single file uniforms importing
  - [ ] Full shader chain uniforms checking and importing
- [ ] Defines
  - [ ] Single file defines importing (possible collisions when importing from multiple ones)
- [ ] Interface blocks

# Importing a struct

`shaders/utils.glsl`

```glsl
struct Player {
  vec3 pos;
  vec3 speed;
};
```

`src/main.rs`

```rust
#[glrs::import(path = "shaders/utils.glsl", name = "Player")]
#[derive(Debug)]
struct GlPlayer;

fn main() {
    dbg!(GlPlayer {
        pos: glam::vec3(0., 0., 0.),
        speed: glam::vec3(0., 0., 0.),
    });
}
```
