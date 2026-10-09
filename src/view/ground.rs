//! The ground's look: the land is one mesh whose vertex colour carries the
//! palette's colour (rgb) and how rocky the spot is (alpha, 0 turf .. 1
//! bare rock). A small fragment shader lays two tiles cut from the Roduro
//! buildings over it (`assets/textures/roduro_grass.png` from the roofs,
//! `roduro_stone.png` from the walls) and mixes them by that rockiness, so
//! the land is made of the same stuff as the homes. Each tile is sampled at
//! two scales so its repeat doesn't show. Rocks use the same material at
//! full rockiness.

use bevy::asset::uuid_handle;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::{Shader, ShaderRef};

use super::palette::{self, Rgb};

const SHADER: Handle<Shader> = uuid_handle!("2f6c1a4e-7b3d-4c8e-9f10-5d2a8b7c6e41");

/// The stone tile, added to the standard material (which carries the grass
/// tile as its base colour and the stone's normal map).
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct Blend {
    #[texture(100)]
    #[sampler(101)]
    pub stone: Handle<Image>,
}

impl MaterialExtension for Blend {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

pub type GroundMat = ExtendedMaterial<StandardMaterial, Blend>;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GroundMat>::default());
    }
}

/// Metres per repeat of the tiles on the ground, and on rocks.
pub const TILE_M: f32 = 6.0;
pub const ROCK_TILE_M: f32 = 1.6;

/// A repeating tile; `srgb` false for a normal map.
pub fn tile(server: &AssetServer, file: &str, srgb: bool) -> Handle<Image> {
    server
        .load_builder()
        .with_settings(move |s: &mut ImageLoaderSettings| {
            s.is_srgb = srgb;
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat, ..ImageSamplerDescriptor::linear() });
        })
        .load(format!("textures/{file}"))
}

/// The one ground material.
pub fn material(server: &AssetServer, mats: &mut Assets<GroundMat>, shaders: &mut Assets<Shader>) -> Handle<GroundMat> {
    let _ = shaders.insert(&SHADER, Shader::from_wgsl(shader_source(), "ground_blend.wgsl"));
    mats.add(GroundMat {
        base: StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tile(server, "roduro_grass.png", true)),
            normal_map_texture: Some(tile(server, "roduro_grass_n.png", false)),
            perceptual_roughness: 0.95,
            reflectance: 0.15,
            cull_mode: None,
            double_sided: false,
            ..default()
        },
        extension: Blend { stone: tile(server, "roduro_stone.png", true) },
    })
}

/// The vertex colour for a spot: the palette's colour, and how rocky it is.
pub fn vertex(c: Rgb, rocky: f32) -> [f32; 4] {
    let l = palette::lin(c);
    [l[0], l[1], l[2], rocky.clamp(0.0, 1.0)]
}

fn shader_source() -> String {
    let (gm, sm) = (palette::GRASS_TILE_MEAN, palette::STONE_TILE_MEAN);
    let lin = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    format!(
        r#"
#import bevy_pbr::{{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_bindings,
}}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{{
    prepass_io::{{VertexOutput, FragmentOutput}},
    pbr_deferred_functions::deferred_output,
}}
#else
#import bevy_pbr::{{
    forward_io::{{VertexOutput, FragmentOutput}},
    pbr_functions::{{apply_pbr_lighting, main_pass_post_lighting_processing}},
}}
#endif

@group(#{{MATERIAL_BIND_GROUP}}) @binding(100) var stone_tex: texture_2d<f32>;
@group(#{{MATERIAL_BIND_GROUP}}) @binding(101) var stone_samp: sampler;

// Average colour of each tile (linear), so tile / mean * palette colour
// comes out at the palette colour.
const GRASS_MEAN: vec3<f32> = vec3<f32>({g0:.4}, {g1:.4}, {g2:.4});
const STONE_MEAN: vec3<f32> = vec3<f32>({s0:.4}, {s1:.4}, {s2:.4});

// A tile at two scales, so its repeat doesn't show.
fn two_scale(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec3<f32> {{
    let a = textureSample(t, s, uv).rgb;
    let b = textureSample(t, s, uv * 0.23 + vec2<f32>(0.5, 0.25)).rgb;
    return sqrt(a * b);
}}

// The tile laid from above on flat ground and from the side on faces, each
// face taking the projection that suits it, so cliffs aren't smeared.
fn triplanar(t: texture_2d<f32>, s: sampler, p: vec3<f32>, n: vec3<f32>, metres: f32) -> vec3<f32> {{
    var w = pow(abs(n), vec3<f32>(4.0));
    w = w / max(w.x + w.y + w.z, 1e-4);
    let top = two_scale(t, s, p.xz / metres);
    let side_x = two_scale(t, s, p.zy / metres);
    let side_z = two_scale(t, s, p.xy / metres);
    return top * w.y + side_x * w.x + side_z * w.z;
}}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {{
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    var tint = vec3<f32>(1.0);
    var rocky = 0.0;
#ifdef VERTEX_COLORS
    tint = in.color.rgb;
    rocky = in.color.a;
#endif
    let wp = in.world_position.xyz;
    let wn = normalize(in.world_normal);
    let grass = triplanar(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, wp, wn, {tile:.2}) / GRASS_MEAN;
    let stone = triplanar(stone_tex, stone_samp, wp, wn, {rock_tile:.2} + ({tile:.2} - {rock_tile:.2}) * (1.0 - rocky)) / STONE_MEAN;
    // (The material's own colour is white unless the weather darkens it: wet ground.)
    pbr_input.material.base_color = vec4<f32>(mix(grass, stone, rocky) * tint * pbr_bindings::material.base_color.rgb, 1.0);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}}
"#,
        tile = TILE_M,
        rock_tile = ROCK_TILE_M,
        g0 = lin(gm[0]),
        g1 = lin(gm[1]),
        g2 = lin(gm[2]),
        s0 = lin(sm[0]),
        s1 = lin(sm[1]),
        s2 = lin(sm[2]),
    )
}
