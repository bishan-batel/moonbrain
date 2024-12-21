use godot::{
    builtin::Vector3,
    classes::{mesh::PrimitiveType, ArrayMesh, Mesh, SurfaceTool},
    obj::NewGd,
    prelude::*,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

#[derive(GodotClass, Debug)]
#[class(tool, init, base = Resource)]
pub struct PlanetGenerator {
    _base: Base<Resource>,

    #[export]
    #[init(val = Vector3::ONE)]
    scale: Vector3,

    #[export]
    detail: u32,
}

impl PlanetGenerator {
    pub fn generate(&self) -> Gd<ArrayMesh> {
        assert!(self.detail != 0, "Invalid Detail Level: 0");

        let mut vertices = vec![];

        self.gen_face(&mut vertices, Vector3::UP, Vector3::RIGHT, Vector3::FORWARD);
        self.gen_face(&mut vertices, Vector3::RIGHT, Vector3::UP, Vector3::BACK);

        self.gen_face(&mut vertices, Vector3::FORWARD, Vector3::UP, Vector3::RIGHT);
        self.gen_face(&mut vertices, Vector3::UP, Vector3::FORWARD, Vector3::LEFT);

        self.gen_face(
            &mut vertices,
            Vector3::FORWARD,
            Vector3::RIGHT,
            Vector3::DOWN,
        );
        self.gen_face(&mut vertices, Vector3::RIGHT, Vector3::FORWARD, Vector3::UP);

        let scale = self.scale;
        vertices.par_iter_mut().for_each(|v| {
            *v = v.normalized() * scale;

            *v += (v.x / scale.x).sin() * 0.1 * scale;
        });

        let mut mesh = SurfaceTool::new_gd();
        mesh.begin(PrimitiveType::TRIANGLES);
        for vert in vertices {
            mesh.add_vertex(vert);
        }
        mesh.generate_normals();
        mesh.commit().expect("Failed to construct mesh")
    }

    fn gen_face(&self, vertices: &mut Vec<Vector3>, right: Vector3, up: Vector3, forward: Vector3) {
        let offset = -(up + right) * (self.detail as f32 - 1.) / 2.
            + forward * (self.detail as f32 - 1.) / 2.;

        for x in 0..(self.detail - 1) {
            for y in 0..(self.detail - 1) {
                let (x, y) = (x as f32, y as f32);

                vertices.push(offset + right * x + up * y);
                vertices.push(offset + right * x + up * (y + 1.));
                vertices.push(offset + right * (x + 1.) + up * y);

                vertices.push(offset + right * x + up * (y + 1.));
                vertices.push(offset + right * (x + 1.) + up * (y + 1.));
                vertices.push(offset + right * (x + 1.) + up * y);
            }
        }
    }
}
