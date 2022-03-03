use super::*;

pub fn set_up_camera(mut commands: Commands) {

    // Ended up messing around with projections and scooching thing around to get bevy_mod_raycast to actually pick up things that were apparently too close to the camera.
    let perspective_projection = PerspectiveProjection {
        fov: (PI / 4.0) * 0.75,
        aspect_ratio: 1.0,
        near: 0.1,
        far: 1000.0,
    };
    let view_projection = bevy::render::camera::CameraProjection::get_projection_matrix(&perspective_projection);
    let frustum = bevy::render::primitives::Frustum::from_view_projection(
        &view_projection,
        &Vec3::ZERO,
        &Vec3::Z,
        bevy::render::camera::CameraProjection::far(&perspective_projection),
    );

    commands
    .spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(super::playing::LEAN_BACK_POS.into()).with_rotation(Quat::from_scaled_axis(super::playing::LEAN_BACK_ANGLE.into())),
        perspective_projection,
        frustum,
        ..Default::default()
    })
    // Delay makes movement + picking not work for a bit, while the table builds
    .with_delay(4.0, |c| {
        c
        .insert(LerpToTarget::default())
        .insert(SlerpToTarget::default())
        .insert_bundle(PickingCameraBundle::default());
    })
    .insert(Name::new("Camera"));

    commands.spawn().insert(Name::new("Light"))
    .insert_bundle(PointLightBundle {
        transform: Transform::from_xyz(0., 0.6, 0.),
        point_light: PointLight {
            range: 0.77,
            shadows_enabled: true,
            shadow_depth_bias: 0.007,
            shadow_normal_bias: 0.6,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(FlickerLight {
        settings: vec![
            FlickerSettings::Constant {
                intensity: 10.,
                color: vec3(211. / 255., 169. / 255., 71. / 255.),
                range: 1.0,
            },
            FlickerSettings::Sin {
                amplitude: 10.,
                frequency: 0.1,
                color: vec3(236. / 255., 195. / 255., 103. / 255.),
                range: 1.5,
            },
            FlickerSettings::Sin {
                amplitude: 0.5,
                frequency: 2.5,
                color: vec3(211. / 255., 81. / 255., 71. / 255.),
                range: 2.0,
            }
        ]
    });
}

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct MakePickable;

pub fn set_up_checkerboard(
    mut commands: Commands, 
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut check_map: ResMut<CheckerMap>,
    mut place_map: ResMut<PlaceMap>
) {
    let table = assets.load("models/table.glb#Scene0");
    let checkboard = assets.load("models/checkboard.glb#Scene0");
    let black_checker = assets.load("models/check_black.glb#Scene0");
    let red_checker = assets.load("models/check_red.glb#Scene0");
    let black_king = assets.load("models/check_black_king.glb#Scene0");
    let red_king = assets.load("models/check_red_king.glb#Scene0");

    let plane_mesh = meshes.add(shape::Plane { size: 0.061 }.into());

    commands
    .spawn()
    .insert(Name::new("Table"))
    .insert(Transform::from_xyz(0., -1., 0.))
    .with_delay(1.0, |c| {
        c.insert(LerpToTarget { ratio: 5.0, target: Vec3::ZERO});
    })
    .insert(GlobalTransform::default())
    .with_children(|b| {
        b.spawn_scene(table);

        b.spawn()
        .insert(Name::new("Checkboard"))
        .insert(Transform::from_xyz(0., 0.06, 0.))
        .insert(GlobalTransform::default())
        .with_children(|b| {
            b.spawn_scene(checkboard);

            for i in 0..64 {
                let coord = index_checkboard(i);
                let pos = get_checkboard_pos(coord);

                let place = b.spawn()
                .insert(Name::new(format!("CheckerPlace{}", i)))
                .insert(CheckerPlace { pos: coord, jumps: vec![] })
                .insert(Transform::from_xyz(pos.x, 0.025, pos.z))
                .insert(GlobalTransform::default())
                .insert(plane_mesh.clone())
                // .insert(place_mat.clone())
                // Visibility is part of bevy_mod_raycast's criteria for picking things for some reason.
                // Using it to enable/disable picking
                .insert(Visibility { is_visible: false })
                .insert(ComputedVisibility::default())
                .insert_bundle(PickableBundle::default())
                .id();

                place_map.insert(coord, place);
            }
        });
    });

    for i in 0..12 {
        let y = 1.;
        let fall_speed = 5.0;

        let initial_coord = get_initial_checkboard_coord(i, Team::Black);
        
        let model = commands.spawn()
        .insert(Name::new("Model"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .with_children(|b| {
            b.spawn_scene(black_checker.clone());
        }).id();
        let king = commands.spawn()
        .insert(Name::new("KingModel"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .with_children(|b| {
            b.spawn_scene(black_king.clone());
        }).id();

        let checker = commands.spawn()
        .insert(Name::new(format!("BlackChecker{}", i)))
        .insert(Checker { team: Team::Black, pos: initial_coord, alive: true, king: false, model, king_model: king })
        .insert(Transform::from_xyz(0., y, 0.))
        .insert(GlobalTransform::default())
        .with_delay(3.0, move |c| {
            c.insert(LerpToTarget { ratio: fall_speed, target: get_checkboard_pos(initial_coord) });
        })
        .add_child(model)
        .add_child(king)
        .id();
        
        check_map.set_place(initial_coord, Some((checker, Team::Black)));

        let initial_coord = get_initial_checkboard_coord(i, Team::Red);

        let model = commands.spawn()
        .insert(Name::new("Model"))
        .insert(MakePickable)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .with_children(|b| {
            b.spawn_scene(red_checker.clone());
        }).id();
        let king = commands.spawn()
        .insert(Name::new("KingModel"))
        .insert(MakePickable)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .with_children(|b| {
            b.spawn_scene(red_king.clone());
        }).id();

        let checker = commands.spawn()
        .insert(Name::new(format!("RedChecker{}", i)))
        .insert(Checker { team: Team::Red, pos: initial_coord, alive: true, king: false, model, king_model: king })
        .insert(Transform::from_xyz(0., y, 0.).with_rotation(Quat::from_rotation_y(PI)))
        .insert(GlobalTransform::default())
        .with_delay(2.0, move |c| {
            c
            .insert(LerpToTarget { ratio: fall_speed, target: get_checkboard_pos(initial_coord) });
        })
        .add_child(model)
        .add_child(king)
        .id();
        
        check_map.set_place(initial_coord, Some((checker, Team::Red)));
    }
}

#[derive(Default)]
pub struct PickableCount(usize);

pub fn should_add_pickables(count: Res<PickableCount>) -> bevy::ecs::schedule::ShouldRun {
    if count.0 >= 12 {
        return bevy::ecs::schedule::ShouldRun::No;
    }
    return bevy::ecs::schedule::ShouldRun::Yes;
}

// Need to get to parent after selection and this is easiest
#[derive(Component)]
pub struct CheckerMesh(pub Entity);

pub fn set_up_checker_pickables(
    mut commands: Commands, 
    models: Query<(Entity, &Children, &Name), (With<MakePickable>, Changed<Children>)>, 
    parents: Query<&Children>, 
    mut count: ResMut<PickableCount>
) {
    for (ent, children0, name) in models.iter() {
        if children0.len() == 0 { continue; }

        let child0 = children0.first().unwrap();
        let children1 = parents.get_component::<Children>(*child0).unwrap();
        let child1 = children1.first().unwrap();
        let children2 = parents.get_component::<Children>(*child1).unwrap();
        let child2 = children2.first().unwrap();

        commands
        .entity(*child2)
        .insert_bundle(PickableBundle::default())
        .insert(CheckerMesh(ent));
        count.0 += 1;

        commands.entity(ent).remove::<MakePickable>();
    }
}
