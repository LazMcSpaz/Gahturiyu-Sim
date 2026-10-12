//! Outlines that hug what's drawn (Laz's P3; drawing only).
//!
//! The shapes to outline (the picked, whoever is under the mouse, foes) are
//! copied each frame, in their outline colour, into one mesh that only a
//! second camera sees (`MASK_LAYER`). That camera follows the main one and
//! draws them into an image the size of the window: a picture of exactly
//! where they are on screen. A full-screen layer on the main camera then
//! looks round each pixel of that picture: a pixel just outside a shape takes
//! the shape's colour, so a thin line traces the real edge of whatever was
//! drawn. (Seen through walls, the line still shows: it marks where your
//! people are.)

use bevy::asset::{uuid_handle, RenderAssetUsages};
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, CompareFunction, RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureFormat};
use bevy::shader::{Shader, ShaderRef};
use bevy::window::PrimaryWindow;

use super::app::{MainCamera, View};

/// The render layer only the mask camera sees.
pub const MASK_LAYER: usize = 3;
/// How far the line reaches out from a shape's edge, in pixels.
pub const LINE_PX: f32 = 1.6;
/// How strong the line is (its alpha).
pub const LINE_ALPHA: f32 = 0.9;

const SHADER: Handle<Shader> = uuid_handle!("2f6a9d3e-41c7-4b0e-8d5a-7e1c9b3f6a20");
const MASK_SHADER: Handle<Shader> = uuid_handle!("8c41e2a7-5d3b-4f19-a6e0-2b7d9c4f1e83");

/// The shapes as the mask camera draws them: their outline colour, and in
/// alpha their depth (so the edge layer can tell what's behind a wall).
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct MaskMat {}

impl Material for MaskMat {
    fn fragment_shader() -> ShaderRef {
        MASK_SHADER.into()
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

const MASK_SOURCE: &str = r#"
#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = vec3<f32>(1.0);
#ifdef VERTEX_COLORS
    c = in.color.rgb;
#endif
    return vec4<f32>(c, max(in.position.z, 1e-7));
}
"#;

/// The full-screen layer that traces the edges.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EdgeMat {
    /// (1 / width, 1 / height, line px, alpha).
    #[uniform(0)]
    pub params: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub mask: Handle<Image>,
}

impl Material for EdgeMat {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(ds) = descriptor.depth_stencil.as_mut() {
            ds.depth_compare = Some(CompareFunction::Always);
            ds.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

const SOURCE: &str = r#"
#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::prepass_utils

struct Out {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Params {
    v: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: Params;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var mask: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var mask_sampler: sampler;

// The quad's corners are already the screen's corners.
@vertex
fn vertex(v: Vertex) -> Out {
    var o: Out;
    o.pos = vec4<f32>(v.position.xy, 0.5, 1.0);
    o.uv = vec2<f32>(v.position.x * 0.5 + 0.5, 0.5 - v.position.y * 0.5);
    return o;
}

// Where something of the scene stands in front of a shape, its line is
// faint (Laz: an outline behind a wall dims). The mask holds each shape's
// depth in its alpha; the main camera's depth prepass holds the scene's.
// (Reversed depth: bigger is nearer.)
fn seen(at: vec2<f32>, depth: f32) -> bool {
#ifdef DEPTH_PREPASS
    let scene = prepass_utils::prepass_depth(vec4<f32>(at, 0.0, 0.0), 0u);
    return depth >= scene * 0.996 - 1e-6;
#else
    return true;
#endif
}

@fragment
fn fragment(in: Out) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(mask));
    let px = vec2<i32>(in.pos.xy);
    let here = textureLoad(mask, clamp(px, vec2<i32>(0), size - 1), 0);
    if (here.a > 0.0) {
        discard;
    }
    let reach = params.v.z;
    // Look round the pixel at two distances: a shape within the near ring
    // gives the full line, one only in the far ring a softer edge to it.
    var col = vec3<f32>(0.0);
    var cover = 0.0;
    var front = false;
    for (var i = 0; i < 12; i = i + 1) {
        let a = f32(i) * 0.5235988;
        let d = vec2<f32>(cos(a), sin(a));
        let n_at = in.pos.xy + d * reach * 0.6;
        let near = textureLoad(mask, clamp(vec2<i32>(n_at), vec2<i32>(0), size - 1), 0);
        if (near.a > 0.0 && cover < 1.0) {
            col = near.rgb;
            cover = 1.0;
            front = front || seen(n_at, near.a);
        }
        let f_at = in.pos.xy + d * reach * 1.25;
        let far = textureLoad(mask, clamp(vec2<i32>(f_at), vec2<i32>(0), size - 1), 0);
        if (far.a > 0.0 && cover < 0.45) {
            col = far.rgb;
            cover = 0.45;
            front = front || seen(f_at, far.a);
        }
    }
    if (cover <= 0.0) {
        discard;
    }
    // Behind something: the same line, faint.
    let k = select(0.32, 1.0, front);
    return vec4<f32>(col * 1.6, cover * params.v.w * k);
}
"#;

/// The mask: its image, the mesh of shapes drawn into it this frame, and the
/// edge layer's material.
#[derive(Resource)]
pub struct Mask {
    pub image: Handle<Image>,
    pub mesh: Handle<Mesh>,
    pub mat: Handle<EdgeMat>,
    size: UVec2,
}

#[derive(Component)]
pub struct MaskCamera;

pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EdgeMat>::default());
        app.add_plugins(MaterialPlugin::<MaskMat>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, follow.after(super::app::camera));
    }
}

fn mask_image(size: UVec2) -> Image {
    // (A float picture: its alpha carries depth.)
    let mut img = Image::new_target_texture(size.x.max(1), size.y.max(1), TextureFormat::Rgba16Float, None);
    img.asset_usage = RenderAssetUsages::RENDER_WORLD;
    img
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<MaskMat>>, mut edges: ResMut<Assets<EdgeMat>>, mut shaders: ResMut<Assets<Shader>>, window: Query<&Window, With<PrimaryWindow>>) {
    let _ = shaders.insert(&SHADER, Shader::from_wgsl(SOURCE, "outline_edges.wgsl"));
    let _ = shaders.insert(&MASK_SHADER, Shader::from_wgsl(MASK_SOURCE, "outline_mask.wgsl"));
    let size = window.single().map(|w| UVec2::new(w.physical_width(), w.physical_height())).unwrap_or(UVec2::new(1600, 1000));
    let image = images.add(mask_image(size));
    // The camera that draws only the shapes to outline.
    commands.spawn((
        Camera3d::default(),
        Camera { order: -1, clear_color: ClearColorConfig::Custom(Color::NONE), ..default() },
        RenderTarget::Image(ImageRenderTarget { handle: image.clone(), scale_factor: 1.0 }),
        Projection::Perspective(PerspectiveProjection { fov: super::cam::FOV_DEG.to_radians(), ..default() }),
        Tonemapping::None,
        bevy::camera::Hdr,
        Msaa::Off,
        RenderLayers::layer(MASK_LAYER),
        Transform::default(),
        MaskCamera,
    ));
    // The shapes, one mesh rebuilt each frame (`scene.rs`), flat colour.
    let mesh = meshes.add(super::mesh::Builder::new().mesh());
    let flat = mats.add(MaskMat::default());
    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(flat), RenderLayers::layer(MASK_LAYER), NoFrustumCulling, NotShadowCaster, NotShadowReceiver, Transform::default()));
    // The full-screen layer that traces the edges, on the main camera.
    let mut quad = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    quad.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[-1.0f32, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]]);
    quad.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 0.0, 1.0]; 4]);
    quad.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
    quad.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    let mat = edges.add(EdgeMat { params: params(size), mask: image.clone() });
    commands.spawn((Mesh3d(meshes.add(quad)), MeshMaterial3d(mat.clone()), NoFrustumCulling, NotShadowCaster, NotShadowReceiver, Transform::default()));
    commands.insert_resource(Mask { image, mesh, mat, size });
}

fn params(size: UVec2) -> Vec4 {
    Vec4::new(1.0 / size.x.max(1) as f32, 1.0 / size.y.max(1) as f32, LINE_PX, LINE_ALPHA)
}

/// Each frame: the mask camera sees what the main camera sees, and the mask
/// keeps the window's size.
#[allow(clippy::type_complexity)]
fn follow(
    game: Res<super::app::Game>,
    mut mask: ResMut<Mask>,
    mut images: ResMut<Assets<Image>>,
    mut edges: ResMut<Assets<EdgeMat>>,
    window: Query<&Window, With<PrimaryWindow>>,
    main: Query<(&Transform, &Projection), (With<MainCamera>, Without<MaskCamera>)>,
    mut cam: Query<(&mut Camera, &mut Transform, &mut Projection), With<MaskCamera>>,
) {
    let (Ok((mt, mp)), Ok((mut c, mut t, mut p))) = (main.single(), cam.single_mut()) else { return };
    c.is_active = game.view == View::Scene;
    *t = *mt;
    *p = mp.clone();
    let Ok(w) = window.single() else { return };
    let size = UVec2::new(w.physical_width(), w.physical_height());
    if size != mask.size && size.x > 0 && size.y > 0 {
        mask.size = size;
        if let Some(mut img) = images.get_mut(&mask.image) {
            *img = mask_image(size);
        }
        if let Some(mut m) = edges.get_mut(&mask.mat) {
            m.params = params(size);
        }
    }
}
