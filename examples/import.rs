#[glrs::import(path = "examples/structs.glsl")]
#[derive(Debug, Default)]
struct Camera;

#[glrs::import(path = "examples/structs.glsl", name = "Player")]
#[derive(Debug, Default)]
struct GlPlayer;

fn main() {
    dbg!(Camera::default());
    dbg!(GlPlayer::default());
}
