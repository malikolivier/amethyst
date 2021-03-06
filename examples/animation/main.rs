//! Displays a shaded sphere to the user.

extern crate amethyst;
extern crate amethyst_animation;
extern crate genmesh;

use amethyst::assets::{Handle, Loader};
use amethyst::core::{GlobalTransform, Parent, Transform, TransformBundle};
use amethyst::core::cgmath::{Deg, InnerSpace, Vector3};
use amethyst::ecs::{Entity, World};
use amethyst::prelude::*;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, ElementState, Event,
                         KeyboardInput, Light, Mesh, Pipeline, PointLight, PosNormTex, Projection,
                         RenderBundle, Rgba, Stage, VirtualKeyCode, WindowEvent};
use amethyst_animation::{get_animation_set, Animation, AnimationBundle, AnimationCommand,
                         EndControl, InterpolationFunction, Sampler, StepDirection,
                         TransformChannel};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // blue
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0); // near-black
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0); // white
const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // black
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

#[derive(Default)]
struct Example {
    pub sphere: Option<Entity>,
    pub animation: Option<Handle<Animation<Transform>>>,
}

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        // Initialise the scene with an object, a light and a camera.
        self.sphere = Some(initialise_sphere(world));
        self.animation = Some(initialise_animation(world));
        initialise_lights(world);
        initialise_camera(world);
    }

    fn handle_event(&mut self, world: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => Trans::Quit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    match virtual_keycode {
                        Some(VirtualKeyCode::Space) => {
                            get_animation_set::<u32, Transform>(
                                &mut world.write(),
                                self.sphere.unwrap().clone(),
                            ).add_animation(
                                1,
                                self.animation.as_ref().unwrap(),
                                EndControl::Loop(None),
                                0.0,
                                AnimationCommand::Start,
                            );
                        }

                        Some(VirtualKeyCode::Left) => {
                            get_animation_set::<u32, Transform>(
                                &mut world.write(),
                                self.sphere.unwrap().clone(),
                            ).step(1, StepDirection::Backward);
                        }

                        Some(VirtualKeyCode::Right) => {
                            get_animation_set::<u32, Transform>(
                                &mut world.write(),
                                self.sphere.unwrap().clone(),
                            ).step(1, StepDirection::Forward);
                        }

                        _ => {}
                    }

                    Trans::None
                }
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let display_config_path = format!(
        "{}/examples/animation/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target(BACKGROUND_COLOUR, 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let config = DisplayConfig::load(&display_config_path);

    let mut game = Application::build(resources, Example::default())?
        .with_bundle(AnimationBundle::<u32, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(TransformBundle::new().with_dep(&["sampler_interpolation_system"]))?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .build()?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| PosNormTex {
            position: [x, y, z],
            normal: Vector3::from([x, y, z]).normalize().into(),
            tex_coord: [0.1, 0.1],
        })
        .triangulate()
        .vertices()
        .collect()
}

/// This function initialises a sphere and adds it to the world.
fn initialise_sphere(world: &mut World) -> Entity {
    // Create a sphere mesh and material.

    use amethyst::assets::Handle;
    use amethyst::renderer::{Material, MaterialDefaults};

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh: Handle<Mesh> =
            loader.load_from_data(gen_sphere(32, 32).into(), (), &world.read_resource());

        let albedo = SPHERE_COLOUR.into();

        let tex_storage = world.read_resource();
        let mat_defaults = world.read_resource::<MaterialDefaults>();

        let albedo = loader.load_from_data(albedo, (), &tex_storage);

        let mat = Material {
            albedo,
            ..mat_defaults.0.clone()
        };

        (mesh, mat)
    };

    let parent_entity = world
        .create_entity()
        .with(Transform::default())
        .with(GlobalTransform::default())
        .build();

    // Create a sphere entity using the mesh and the material.
    world
        .create_entity()
        .with(Transform {
            translation: [0., 1.0, 0.].into(),
            ..Transform::default()
        })
        .with(GlobalTransform::default())
        .with(Parent {
            entity: parent_entity.clone(),
        })
        .with(mesh)
        .with(material)
        .build();

    /*let mut nodes = HashMap::default();
    nodes.insert(0, parent_entity.clone());
    world
        .write()
        .insert(parent_entity, AnimationHierarchy { nodes });*/
    parent_entity
}

fn initialise_animation(world: &mut World) -> Handle<Animation<Transform>> {
    let loader = world.write_resource::<Loader>();
    let translation_sampler = Sampler {
        input: vec![0., 1., 2., 3., 4.],
        function: InterpolationFunction::Linear,
        output: vec![
            [0., 0., 0.].into(),
            [1., 0., 0.].into(),
            [0., 0., 0.].into(),
            [-1., 0., 0.].into(),
            [0., 0., 0.].into(),
        ],
    };

    /*let scale_sampler = Sampler {
        input: vec![0., 1., 2., 3., 4.],
        ty: InterpolationType::Linear,
        output: AnimationOutput::Scale(vec![
            [1., 1., 1.],
            [0.6, 0.6, 0.6],
            [0.3, 0.3, 0.3],
            [0.6, 0.6, 0.6],
            [1., 1., 1.],
        ]),
    };*/

    use std::f32::consts::FRAC_1_SQRT_2;
    let rotation_sampler = Sampler {
        input: vec![0., 1., 2., 3., 4.],
        function: InterpolationFunction::SphericalLinear,
        output: vec![
            [1., 0., 0., 0.].into(),
            [FRAC_1_SQRT_2, 0., 0., FRAC_1_SQRT_2].into(),
            [0., 0., 0., 1.].into(),
            [-FRAC_1_SQRT_2, 0., 0., FRAC_1_SQRT_2].into(),
            [-1., 0., 0., 0.].into(),
        ],
    };
    let translation_animation_handle =
        loader.load_from_data(translation_sampler, (), &world.read_resource());
    //let scale_animation_handle = loader.load_from_data(scale_sampler, &world.read_resource());
    let rotation_animation_handle =
        loader.load_from_data(rotation_sampler, (), &world.read_resource());
    let animation = Animation {
        nodes: vec![
            (
                0,
                TransformChannel::Translation,
                translation_animation_handle,
            ),
            //(0, scale_animation_handle),
            (0, TransformChannel::Rotation, rotation_animation_handle),
        ],
    };
    loader.load_from_data(animation, (), &world.read_resource())
}

/// This function adds an ambient light and a point light to the world.
fn initialise_lights(world: &mut World) {
    // Add ambient light.
    world.add_resource(AmbientColor(AMBIENT_LIGHT_COLOUR));

    let light: Light = PointLight {
        center: LIGHT_POSITION.into(),
        radius: LIGHT_RADIUS,
        intensity: LIGHT_INTENSITY,
        color: POINT_LIGHT_COLOUR,
        ..Default::default()
    }.into();

    // Add point light.
    world.create_entity().with(light).build();
}

/// This function initialises a camera and adds it to the world.
fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::Matrix4;
    let transform =
        Matrix4::from_translation([0.0, 0.0, -4.0].into()) * Matrix4::from_angle_y(Deg(180.));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}
