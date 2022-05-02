use ash::vk;
use nalgebra_glm::TMat4;

use crate::vulkan_context::VulkanContext;

pub struct Model {
    pub name: String,
    pub transform: nalgebra_glm::TMat4<f32>,
    pub mesh: Mesh,
}

impl Model {
    pub fn new(
        name: String,
        transform: nalgebra_glm::TMat4<f32>,
        primitives: Vec<Primitive>,
    ) -> Self {
        let mesh = Mesh { primitives };
        Self {
            name,
            transform,
            mesh,
        }
    }
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub index_offset: u32,
    pub vertex_offset: u32,
    pub num_indices: u32,
}

pub fn import_models(vulkan_context: &VulkanContext) -> Vec<Model> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let (gltf, buffers, _images) = gltf::import("assets/BoomBoxWithAxes.gltf").unwrap();
    let mut models = Vec::new();
    let mut index_offset = 0;
    let mut vertex_offset = 0;

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            import_node(
                node,
                &mut indices,
                &mut vertices,
                &buffers,
                &mut models,
                &mut index_offset,
                &mut vertex_offset,
                &nalgebra_glm::identity(),
            );
        }
    }

    // Copy indices and vertices into buffers.
    unsafe {
        vulkan_context.index_buffer.overwrite(&indices);
        vulkan_context.vertex_buffer.overwrite(&vertices);
    };

    models
}

fn import_node(
    node: gltf::Node,
    indices: &mut Vec<u32>,
    vertices: &mut Vec<Vertex>,
    buffers: &[gltf::buffer::Data],
    models: &mut Vec<Model>,
    index_offset: &mut u32,
    vertex_offset: &mut u32,
    parent_transform: &nalgebra_glm::TMat4<f32>,
) {
    let local_transform: TMat4<f32> = node.transform().matrix().into();
    let transform = parent_transform * &local_transform;
    let name = if let Some(name) = node.name() {
        name.to_string()
    } else {
        format!("Node {}", node.index())
    };

    println!("Importing {} with transform {:?}", name, local_transform);

    if let Some(mesh) = node.mesh() {
        let mut primitives = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let mut num_indices = 0;
            let mut num_vertices = 0;
            for i in reader.read_indices().unwrap().into_u32() {
                num_indices += 1;
                indices.push(i);
            }

            if let Some(colours) = reader.read_colors(0) {
                for (colour, position) in
                    colours.into_rgb_f32().zip(reader.read_positions().unwrap())
                {
                    num_vertices += 1;
                    vertices.push(Vertex::new_coloured(
                        position[0],
                        position[1],
                        position[2],
                        colour[0],
                        colour[1],
                        colour[2],
                    ));
                }
            } else {
                for position in reader.read_positions().unwrap() {
                    num_vertices += 1;
                    vertices.push(Vertex::new(position[0], position[1], position[2]));
                }
            }

            primitives.push(Primitive {
                index_offset: *index_offset,
                vertex_offset: *vertex_offset,
                num_indices,
            });

            assert_eq!(indices.len(), (*index_offset + num_indices) as usize);
            assert_eq!(vertices.len(), (*vertex_offset + num_vertices) as usize);

            *index_offset += num_indices;
            *vertex_offset += num_vertices;
            println!("Offset is now: {}", index_offset);
        }

        models.push(Model::new(name, transform, primitives));
    }

    for node in node.children() {
        import_node(
            node,
            indices,
            vertices,
            buffers,
            models,
            index_offset,
            vertex_offset,
            &local_transform,
        );
    }
}

pub struct VertexInputDescription {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

#[repr(C, align(16))]
#[derive(Debug, Clone)]
pub struct Vertex {
    vx: f32,
    vy: f32,
    vz: f32,
    r: f32,
    g: f32,
    b: f32,
}

impl Vertex {
    pub fn description() -> VertexInputDescription {
        VertexInputDescription {
            bindings: vec![vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as _,
                input_rate: vk::VertexInputRate::VERTEX,
            }],
            attributes: vec![
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 0,
                },
                vk::VertexInputAttributeDescription {
                    binding: 0,
                    location: 1,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: (std::mem::size_of::<f32>() * 3) as u32,
                },
            ],
        }
    }

    pub fn new(vx: f32, vy: f32, vz: f32) -> Self {
        Self {
            vx,
            vy,
            vz,
            ..Default::default()
        }
    }

    pub fn new_coloured(vx: f32, vy: f32, vz: f32, r: f32, g: f32, b: f32) -> Self {
        Self {
            vx,
            vy,
            vz,
            r,
            g,
            b,
            ..Default::default()
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            vx: 0.,
            vy: 0.,
            vz: 0.,
            r: 1.,
            g: 1.,
            b: 1.,
        }
    }
}
