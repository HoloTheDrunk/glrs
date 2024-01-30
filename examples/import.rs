glrs::import_many! {
    #[path = "examples/structs.glsl"]
    #[derive(Debug, Default)]
    struct Camera;

    #[path = "examples/structs.glsl"]
    #[name = "Player"]
    #[derive(Debug)]
    struct GlPlayer;
}

#[glrs::import(path = "examples/structs.glsl", name = "Light")]
#[derive(Debug, Default)]
struct PointLight;

fn main() {
    dbg!(Camera::default());
    dbg!(GlPlayer {
        pos: glam::vec3(0., 0., 0.),
        speed: glam::vec3(0., 0., 0.)
    });
    dbg!(PointLight::default());
}
