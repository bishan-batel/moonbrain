use core::f32;
use std::ops::Deref;

use godot::{
    builtin::{math::FloatExt, Vector3},
    classes::{mesh::PrimitiveType, ArrayMesh, SurfaceTool},
    obj::NewGd,
    prelude::*,
};

#[derive(Debug, Clone)]
pub struct Triangle([usize; 3]);

#[derive(Debug, Default, Clone)]
struct Edge(Vec<usize>);

impl Deref for Triangle {
    type Target = [usize; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Edge {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct SphereGenerator {
    // numbering 'constants'
    num_divisions: usize,
    verts_per_face: usize,
    total_verts: usize,
    tri_per_face: usize,

    // buffers
    vertices: Vec<Vector3>,
    triangles: Vec<Triangle>,
}

#[derive(Debug, Clone)]
pub struct OctoSphere {
    pub detail: usize,
    pub verticies: Vec<Vector3>,
    pub triangles: Vec<Triangle>,
}

impl OctoSphere {
    pub fn create_mesh(&self) -> Gd<ArrayMesh> {
        let mut surface_tool = SurfaceTool::new_gd();

        surface_tool.begin(PrimitiveType::TRIANGLES);

        for v in self.verticies.iter() {
            let v = v.normalized_or_zero();
            // let v = v.rotated(Vector3::UP, f32::consts::TAU / self.detail as f32);

            let uv = {
                let theta = v.x.atan2(v.z) / f32::consts::TAU + 0.5;
                let height = 1. - (v.y.asin() / f32::consts::PI + 0.5);

                Vector2::new(theta, height)
            };

            surface_tool.set_uv(uv);
            surface_tool.add_vertex(v);
        }

        for Triangle([v0, v1, v2]) in self.triangles.iter() {
            surface_tool.add_index(*v0 as i32);
            surface_tool.add_index(*v1 as i32);
            surface_tool.add_index(*v2 as i32);
        }

        surface_tool.generate_normals();
        surface_tool.generate_tangents();
        surface_tool.commit().expect("Failed to generate sphere")
    }
}

impl SphereGenerator {
    // Indices of the vertex pairs that make up each of the initial 12 edges
    const VERTEX_PAIRS: &[usize] = &[
        0, 1, 0, 2, 0, 3, 0, 4, 1, 2, 2, 3, 3, 4, 4, 1, 5, 1, 5, 2, 5, 3, 5, 4,
    ];

    // Indices of the edge triplets that make up the initial 8 faces
    const EDGE_TRIPLETS: &[usize] = &[
        0, 1, 4, 1, 2, 5, 2, 3, 6, 3, 0, 7, 8, 9, 4, 9, 10, 5, 10, 11, 6, 11, 8, 7,
    ];

    /// The six initial vertices
    const BASE_VERTICES: &[Vector3] = &[
        Vector3::UP,
        Vector3::LEFT,
        Vector3::BACK,
        Vector3::RIGHT,
        Vector3::FORWARD,
        Vector3::DOWN,
    ];

    pub fn gen(num_divisions: usize) -> OctoSphere {
        let verts_per_face = ((num_divisions + 3).pow(2) - (num_divisions + 3)) / 2;
        let total_verts = verts_per_face * 8 - (num_divisions + 2) * 12 + 6;
        let tri_per_face = (num_divisions + 1).pow(2);

        Self {
            num_divisions,
            verts_per_face,
            total_verts,
            tri_per_face,
            ..Default::default()
        }
        .generate()
    }

    fn generate(mut self) -> OctoSphere {
        self.triangles = Vec::with_capacity(self.tri_per_face * 8);

        // add base vertices
        self.vertices = Vec::with_capacity(self.total_verts);
        self.vertices.extend(Self::BASE_VERTICES);

        // create 12 edges
        let mut edges = vec![Edge::default(); 12];

        for i in (0..Self::VERTEX_PAIRS.len()).step_by(2) {
            let start_vert = self.vertices[Self::VERTEX_PAIRS[i]];
            let end_vert = self.vertices[Self::VERTEX_PAIRS[i + 1]];

            let mut edge_indecies = vec![0usize; self.num_divisions + 2];
            edge_indecies[0] = Self::VERTEX_PAIRS[i];

            for division_idx in 0..self.num_divisions {
                let t = (division_idx as real + 1.) / (self.num_divisions as real + 1.);

                // witchcraft, thank you sebas lague
                edge_indecies[division_idx + 1] = self.vertices.len();
                self.vertices.push(start_vert.slerp(end_vert, t));
            }

            edge_indecies[self.num_divisions + 1] = Self::VERTEX_PAIRS[i + 1];
            edges[i / 2] = Edge(edge_indecies);
        }

        for i in (0..Self::EDGE_TRIPLETS.len()).step_by(3) {
            let face_idx = i / 3;
            let should_reverse = face_idx >= 4;

            self.create_face(
                &edges[Self::EDGE_TRIPLETS[i]],
                &edges[Self::EDGE_TRIPLETS[i + 1]],
                &edges[Self::EDGE_TRIPLETS[i + 2]],
                should_reverse,
            );
        }

        OctoSphere {
            detail: self.num_divisions,
            triangles: self.triangles,
            verticies: self.vertices,
        }
    }

    fn create_face(&mut self, side_a: &Edge, side_b: &Edge, side_c: &Edge, should_reverse: bool) {
        let mut vertex_map = Vec::with_capacity(self.verts_per_face);
        vertex_map.push(side_a[0]);

        for i in 1..(side_a.len() - 1) {
            vertex_map.push(side_a[i]);

            let side_a_vert = self.vertices[side_a[i]];
            let side_b_vert = self.vertices[side_b[i]];

            let inner_points = i - 1;

            for j in 0..inner_points {
                let t = (j as real + 1.) / (inner_points as real + 1.);

                vertex_map.push(self.vertices.len());
                self.vertices.push(side_a_vert.slerp(side_b_vert, t))
            }

            vertex_map.push(side_b[i]);
        }

        for v in side_c.iter() {
            vertex_map.push(*v);
        }

        // Triangulation

        for row in 0..=self.num_divisions {
            let mut top_vert = ((row + 1).pow(2) - row - 1) / 2;
            let mut bottom_vert = ((row + 2).pow(2) - row - 2) / 2;

            for column in 0..=(2 * row) {
                let (v0, v1, v2) = if column % 2 == 0 {
                    top_vert += 1;
                    bottom_vert += 1;
                    (top_vert - 1, bottom_vert, bottom_vert - 1)
                } else {
                    (top_vert, bottom_vert, top_vert - 1)
                };

                let verts = if should_reverse {
                    [v0, v2, v1]
                } else {
                    [v0, v1, v2]
                };

                let tri = verts.map(|i| vertex_map[i]);
                println!("{tri:?}");
                self.triangles.push(Triangle(tri));
            }
        }
    }
}
