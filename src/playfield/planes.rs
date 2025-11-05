use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::orientation::OrientationConfig;
use crate::playfield::Boundary;

pub struct PlanesPlugin;

impl Plugin for PlanesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_box_planes)
            .register_type::<PlaneConfig>()
            .init_resource::<PlaneConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<PlaneConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::PlanesInspector)),
            );
    }
}

// you can't use an #[inspector()] w/attenuation_distance
// because you have to use a logarithmic range to reach f32::INFINITY which is
// its default problem for another day...
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct PlaneConfig {
    pub front:                 bool,
    pub back:                  bool,
    pub top:                   bool,
    pub bottom:                bool,
    pub left:                  bool,
    pub right:                 bool,
    pub alpha_mode:            Option<AlphaMode>,
    pub base_color:            Color,
    #[reflect(ignore)]
    pub cull_mode:             Option<Face>,
    pub double_sided:          bool,
    pub emissive:              LinearRgba,
    pub attenuation_distance:  f32,
    #[inspector(min = 1.0, max = 3.0, display = NumberDisplay::Slider)]
    pub ior:                   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub diffuse_transmission:  f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub metallic:              f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub perceptual_roughness:  f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub reflectance:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub specular_transmission: f32,
    #[inspector(min = 0.001, max = 10.0, display = NumberDisplay::Slider)]
    pub(crate) thickness:      f32,
}

impl Default for PlaneConfig {
    fn default() -> Self {
        Self {
            front:                 false,
            back:                  false,
            left:                  false,
            right:                 false,
            top:                   false,
            bottom:                false,
            alpha_mode:            None,
            attenuation_distance:  f32::INFINITY,
            base_color:            Color::from(LinearRgba::new(1., 1., 1., 1.)),
            cull_mode:             Some(Face::Back),
            diffuse_transmission:  0.,
            double_sided:          false,
            emissive:              LinearRgba::BLACK,
            ior:                   1.5,
            metallic:              0.,
            perceptual_roughness:  0.5,
            reflectance:           0.5,
            specular_transmission: 0.,
            thickness:             0.001,
        }
    }
}

#[derive(Component)]
struct BoxPlane {
    plane_type: PlaneType,
}

#[derive(PartialEq, Eq, Hash)]
enum PlaneType {
    Back,
    Front,
    Bottom,
    Top,
    Left,
    Right,
}

#[allow(clippy::too_many_arguments)]
fn create_or_update_plane(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    planes_config: &PlaneConfig,
    size: Vec3,
    position: Vec3,
    axis: Vec3,
    plane_type: PlaneType,
    existing_entity: Option<Entity>,
) -> Entity {
    let cuboid = Cuboid {
        half_size: size / 2.0,
    };
    let mesh = meshes.add(Mesh::from(cuboid));
    let material_handle = get_plane_material(materials, planes_config);
    let rotation = Quat::from_axis_angle(axis, 0.);
    let transform = Transform::from_translation(position).with_rotation(rotation);

    if let Some(entity) = existing_entity {
        commands
            .entity(entity)
            .insert(Mesh3d(mesh.clone()))
            .insert(MeshMaterial3d(material_handle.clone()))
            .insert(transform)
            .id()
    } else {
        commands
            .spawn(BoxPlane { plane_type })
            .insert(Mesh3d(mesh.clone()))
            .insert(MeshMaterial3d(material_handle.clone()))
            .insert(transform)
            .id()
    }
}

fn manage_box_planes(
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    orientation: Res<CameraOrientation>,
    planes_config: Res<PlaneConfig>,
    planes: Query<(Entity, &BoxPlane)>,
) {
    if !planes_config.is_changed() {
        return;
    }

    let plane_specifications = get_plane_specifications(
        &planes_config,
        boundary.transform.scale,
        &orientation.config,
    );

    for (plane_type, enabled, size, position, axis) in plane_specifications {
        let existing_plane = planes.iter().find(|(_, bp)| bp.plane_type == plane_type);

        if enabled {
            let existing_entity = existing_plane.map(|(entity, _)| entity);

            create_or_update_plane(
                &mut commands,
                &mut meshes,
                &mut materials,
                &planes_config,
                size,
                position,
                axis,
                plane_type,
                existing_entity,
            );
        } else if let Some((entity, _)) = existing_plane {
            commands.entity(entity).despawn();
        }
    }
}

fn get_plane_specifications(
    config: &Res<PlaneConfig>,
    box_size: Vec3,
    orientation: &OrientationConfig,
) -> [(PlaneType, bool, Vec3, Vec3, Vec3); 6] {
    [
        (
            PlaneType::Back,
            config.back,
            Vec3::new(box_size.x, box_size.y, config.thickness),
            Vec3::new(0., 0., -box_size.z / 2.),
            orientation.axis_profundus,
        ),
        (
            PlaneType::Front,
            config.front,
            Vec3::new(box_size.x, box_size.y, config.thickness),
            Vec3::new(0., 0., box_size.z / 2.),
            orientation.axis_profundus,
        ),
        (
            PlaneType::Bottom,
            config.bottom,
            Vec3::new(box_size.x, config.thickness, box_size.z),
            Vec3::new(0., -box_size.y / 2., 0.0),
            orientation.axis_mundi,
        ),
        (
            PlaneType::Top,
            config.top,
            Vec3::new(box_size.x, config.thickness, box_size.z),
            Vec3::new(0., box_size.y / 2., 0.0),
            orientation.axis_mundi,
        ),
        (
            PlaneType::Left,
            config.left,
            Vec3::new(config.thickness, box_size.y, box_size.z),
            Vec3::new(-box_size.x / 2., 0., 0.0),
            orientation.axis_orbis,
        ),
        (
            PlaneType::Right,
            config.right,
            Vec3::new(config.thickness, box_size.y, box_size.z),
            Vec3::new(box_size.x / 2., 0., 0.0),
            orientation.axis_orbis,
        ),
    ]
}

fn get_plane_material(
    materials: &mut Assets<StandardMaterial>,
    config: &PlaneConfig,
) -> Handle<StandardMaterial> {
    let mut material = StandardMaterial {
        attenuation_distance: config.attenuation_distance,
        base_color: config.base_color,
        cull_mode: config.cull_mode,
        diffuse_transmission: config.diffuse_transmission,
        double_sided: config.double_sided,
        emissive: config.emissive,
        ior: config.ior,
        metallic: config.metallic,
        perceptual_roughness: config.perceptual_roughness,
        reflectance: config.reflectance,
        specular_transmission: config.specular_transmission,
        thickness: config.thickness,
        ..default()
    };

    // Only set alpha_mode if it's Some
    if let Some(alpha_mode) = config.alpha_mode {
        material.alpha_mode = alpha_mode;
    }

    materials.add(material)
}
