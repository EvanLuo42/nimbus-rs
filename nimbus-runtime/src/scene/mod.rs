use crate::rendering::buffers::VertexBuffer;
use crate::rendering::context::RenderContext;
use crate::rendering::mesh::{Mesh, Submesh};
use crate::resources::material::{MaterialId, MaterialManager};
use crate::scene::object::SceneObject;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::{Format, FormatFeatures};
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::sync::GpuFuture;
use vulkano::sync;

pub mod object;

static MATERIAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_material_id() -> MaterialId {
    MaterialId(MATERIAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Arc<SceneObject>>
}

impl Scene {
    pub fn load(path: impl AsRef<Path>, ctx: &RenderContext, material_manager: Arc<Mutex<MaterialManager>>, graphics_pipeline: Arc<GraphicsPipeline>) -> Self {
        let (document, buffers, images) = gltf::import(path).unwrap();

        let mut objects = Vec::new();

        let mut image_views = vec![];
        
        for image in images {
            let format = image::guess_format(&image.pixels).unwrap();
            let decoded = image::load_from_memory_with_format(&image.pixels, format).unwrap();
            let rgba = decoded.to_rgb8();
            let tex = upload_texture_image(ctx, rgba);
            image_views.push(tex);
        }
        
        let mut material_map = HashMap::new();
        for (i, mat) in document.materials().enumerate() {
            if let Some(texture) = mat.pbr_metallic_roughness().base_color_texture() {
                let tex_index = texture.texture().source().index();
                let (view, sampler) = &image_views[tex_index];

                let descriptor_set = DescriptorSet::new(
                    ctx.descriptor_set_allocator.clone(),
                    graphics_pipeline.layout().set_layouts()[0].clone(),
                    [WriteDescriptorSet::image_view_sampler(0, view.clone(), sampler.clone())],
                    []
                ).unwrap();
                
                let material_id = material_manager.lock().unwrap().insert(descriptor_set);
                material_map.insert(i, material_id);
            }
        }

        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                let transform = Mat4::from_cols_array_2d(&node.transform().matrix());
                
                let mut all_vertices = vec![];
                let mut all_indices = vec![];
                let mut submeshes = vec![];

                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|b| Some(&buffers[b.index()]));

                    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                    let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|tc| tc.into_f32().collect()).unwrap_or_else(|| vec![[0.0; 2]; positions.len()]);
                    let local_indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

                    let vertex_base = all_vertices.len() as u32;

                    let vertices: Vec<VertexBuffer> = positions.iter().zip(normals.iter()).zip(tex_coords.iter())
                        .map(|((pos, norm), uv)| VertexBuffer {
                            position: Vec3::new(pos[0], pos[1], pos[2]),
                            normal: Vec3::new(norm[0], norm[1], norm[2]),
                            uv: Vec2::new(uv[0], uv[1]),
                        })
                        .collect();

                    let adjusted_indices: Vec<u32> = local_indices.iter().map(|i| i + vertex_base).collect();
                    
                    let mat_id = primitive.material().index()
                        .and_then(|i| material_map.get(&i).cloned())
                        .unwrap();

                    let material = material_manager.lock().unwrap().get(mat_id).unwrap();

                    let submesh = Arc::new(Submesh {
                        index_offset: all_indices.len() as u32,
                        index_count: adjusted_indices.len() as u32,
                        material,
                    });

                    all_vertices.extend(vertices);
                    all_indices.extend(adjusted_indices);
                    submeshes.push(submesh);
                }

                let vertex_buffer = Buffer::from_iter(
                    ctx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::VERTEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                        ..Default::default()
                    },
                    all_vertices,
                ).unwrap();

                let index_buffer = Buffer::from_iter(
                    ctx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::INDEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                        ..Default::default()
                    },
                    all_indices,
                ).unwrap();

                let mesh = Arc::new(Mesh {
                    vertex_buffer,
                    index_buffer,
                    submeshes,
                });
                
                let object = Arc::new(SceneObject {
                    mesh,
                    model: transform,
                });

                objects.push(object);
            }
        }
        Scene { objects }
    }
}

fn upload_texture_image(
    ctx: &RenderContext,
    rgba: image::RgbImage,
) -> (Arc<ImageView>, Arc<Sampler>) {
    let (width, height) = rgba.dimensions();
    
    let image = Image::new(
        ctx.memory_allocator.clone(),
        ImageCreateInfo {
            format: Format::R8G8B8_SRGB,
            extent: [width, height, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    ).unwrap();
    
    let staging = Buffer::from_iter(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        rgba.into_raw(),
    ).unwrap();
    
    let mut builder = AutoCommandBufferBuilder::primary(
        ctx.command_allocator.clone(),
        ctx.graphics_queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    ).unwrap();

    builder
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(staging, image.clone()))
        .unwrap();

    let command_buffer = builder.build().unwrap();
    let future = sync::now(ctx.device.clone())
        .then_execute(ctx.graphics_queue.clone(), command_buffer).unwrap()
        .then_signal_fence_and_flush().unwrap();

    future.wait(None).unwrap();
    
    let view = ImageView::new_default(image).unwrap();
    let sampler = Sampler::new(ctx.device.clone(), SamplerCreateInfo {
        mag_filter: Filter::Linear,
        min_filter: Filter::Linear,
        address_mode: [SamplerAddressMode::Repeat; 3],
        ..Default::default()
    }).unwrap();

    (view, sampler)
}
