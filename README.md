# glrs

Compile-time conversion of GLSL structs to glm-rs Rust structs.

# Example

`shaders/utils.glsl`

```glsl
// ...
layout(std140) struct Player {
  vec3 pos;
  vec3 speed;
};
```

`src/main.rs`

```rs
#[glrs::import(path = "shaders/utils.glsl", name = "Player")]
struct GlPlayer;

fn main() {
  dbg!(GlPlayer {
    pos: glm::vec3(0., 0., 0.),
    speed: glm::vec3(0., 0., 0.),
  });
}
```
