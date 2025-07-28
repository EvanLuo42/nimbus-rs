use wgpu::{BindGroupLayoutEntry, BindingType, Color, PushConstantRange, Sampler, SamplerBindingType, ShaderModule, ShaderStages, Texture, TextureSampleType, TextureView, TextureViewDimension};

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub vertex_shader: ShaderModule,
    pub fragment_shader: ShaderModule,
    pub ty: MaterialType,
}

impl Material {
    pub fn is_transparent(&self) -> bool {
        match &self.ty {
            MaterialType::Unlit { base_color } => match base_color {
                BaseColorType::Factor { color } => color.a < 1.0,
                BaseColorType::Texture { .. } => true,
            },
            MaterialType::Pbr {
                base_color,
                ..
            } => match base_color {
                BaseColorType::Factor { color } => color.a < 1.0,
                BaseColorType::Texture { .. } => true,
            },
            MaterialType::Custom => {
                unimplemented!()
            },
        }
    }
}

#[derive(Clone)]
pub enum MaterialType {
    Pbr {
        base_color: BaseColorType,
        metallic_roughness: MetallicRoughnessType
    },
    Unlit {
        base_color: BaseColorType
    },
    Custom
}

impl MaterialType {
    pub fn bind_group_layout_entries(&self) -> Vec<BindGroupLayoutEntry> {
        let mut entries = vec![];

        match self {
            MaterialType::Pbr { base_color, metallic_roughness } => {
                if let BaseColorType::Texture { .. } = base_color {
                    Self::push_entry(&mut entries, 0);
                }

                if let MetallicRoughnessType::Texture { .. } = metallic_roughness {
                    Self::push_entry(&mut entries, 3)
                }
            }

            MaterialType::Unlit { base_color } => {
                if let BaseColorType::Texture { .. } = base_color {
                    Self::push_entry(&mut entries, 0);
                }
            }

            MaterialType::Custom => {
                unimplemented!()
            }
        }

        entries
    }

    pub fn push_constant_ranges(&self) -> Vec<PushConstantRange> {
        match self {
            MaterialType::Pbr { .. } | MaterialType::Unlit { .. } => vec![PushConstantRange {
                stages: ShaderStages::FRAGMENT,
                range: 0..64,
            }],
            MaterialType::Custom => vec![],
        }
    }

    fn push_entry(entries: &mut Vec<BindGroupLayoutEntry>, binding: u32) {
        entries.push(BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: true
            },
            count: None
        });
        entries.push(BindGroupLayoutEntry {
            binding: binding + 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        });
    }
}

#[derive(Clone)]
pub enum BaseColorType {
    Factor {
        color: Color
    },
    Texture {
        texture: Texture,
        texture_view: TextureView,
        sampler: Sampler,
    }
}

#[derive(Clone)]
pub enum MetallicRoughnessType {
    Texture {
        texture: Texture,
        texture_view: TextureView,
        sampler: Sampler
    },
    Factor {
        metallic: Option<f32>,
        roughness: Option<f32>
    }
}

pub type RawMaterial<'a> = gltf::Material<'a>;
