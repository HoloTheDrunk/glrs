use glam::*;
use glrs::import;

// Source:
// https://stackoverflow.com/questions/
// 64251852/is-there-a-way-to-check-a-struct-has-a-field-and-check-its-type/70978292#70978292
pub trait EqType {
    type Itself;
}

impl<T> EqType for T {
    type Itself = T;
}

fn ty_must_eq<T, U>(_: T)
where
    T: EqType<Itself = U>,
{
}

macro_rules! assert_is_type {
    ($t:ty, $i:ident: $ti:ty) => {
        const _: () = {
            #[allow(unused)]
            fn dummy(v: $t) {
                ty_must_eq::<_, $ti>(v.$i);
            }
        };
    };
}

#[import(path = "tests/structs.glsl", name = "FieldTypes")]
struct Fields;

#[test]
fn basic_struct() {
    assert_is_type!(Fields, vec3_l: f32::Vec3);
    assert_is_type!(Fields, vec3_r: f32::Vec3);

    assert_is_type!(Fields, arr: [f32; 3]);
    assert_is_type!(Fields, arr_arr: [[f32; 3]; 2]);
}

// TODO: use try-build to check error messages
