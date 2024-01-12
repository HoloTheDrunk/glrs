#[glrs::import(path = "examples/structs.glsl")]
#[derive(Debug, Default)]
struct Camera;

fn main() {
    dbg!(Camera::default());
}
