// #[glrs::import(path = "examples/structs.glsl")]
// #[derive(Debug, Default)]
// struct Camera;
//
// #[glrs::import(path = "examples/structs.glsl", name = "Player")]
// #[derive(Debug)]
// struct GlPlayer;

glrs::import! {
    #[path = "examples/structs.glsl"]
    #[derive(Debug, Default)]
    struct Camera;

    #[path = "examples/structs.glsl"]
    #[name = "Player"]
    #[derive(Debug)]
    struct GlPlayer;
}

fn main() {
    dbg!(Camera::default());
    dbg!(GlPlayer {
        pos: glam::vec3(0., 0., 0.),
        speed: glam::vec3(0., 0., 0.)
    });
}
