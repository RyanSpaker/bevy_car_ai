use std::f32::consts::TAU;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

/// A resource containing information about the track
#[derive(Clone, Debug, Reflect, Resource)]
pub struct TrackConfig{
    /// Logical width and height of the track
    pub logical_size: Vec2,
    /// Scale of the track in the world coordinates
    pub scale: f32
}
impl Default for TrackConfig{
    fn default() -> Self {
        Self{logical_size: Vec2::new(100.0, 100.0), scale: 1.0}
    }
}
impl TrackConfig{
    /// Transforms a track position to a world position
    pub fn track_to_world(&self, track_coords: Vec2)->Vec2{
        (track_coords - self.logical_size*0.5)*self.scale
    }
    /// Transforms a world position to a track position
    pub fn world_to_track(&self, world_pos: Vec2) -> Vec2{
        (world_pos/self.scale) + self.logical_size*0.5
    }
    /// Updates the scale of the track to be as large as possible while keeping the entire track within bounds of the window
    pub fn compute_scale(&mut self, window_size: Vec2){
        self.scale = (window_size / self.logical_size).min_element();
    }
    /// A system which queries for window size and updates the scale accordingly
    pub fn update_scale(mut config: ResMut<Self>, windows: Query<&Window>){
        let Ok(window) = windows.get_single() else {return;};
        config.compute_scale(Vec2::new(window.height(), window.width()));
    }
}

/// A resource containing physical simulations constants such as acceleration and friction
#[derive(Clone, Debug, Reflect, Resource)]
pub struct CarPhysicsConfig{
    pub rotational_acceleration: f32,
    pub max_rotational_velocity: f32,
    pub forward_acceleration: f32,
    pub friction: f32,
    pub drift_factor: f32,
    pub max_forward_velocity: f32
}
impl Default for CarPhysicsConfig{
    fn default() -> Self {
        Self { 
            rotational_acceleration: 50.0, 
            max_rotational_velocity: TAU, 
            forward_acceleration: 200.0, 
            friction: 0.97, 
            drift_factor: 0.99, 
            max_forward_velocity: 150.0
        }
    }
}

/// Tag component for cars
#[derive(Clone, Debug, Default, Reflect, Component)]
pub struct Car;

/// Tag component used for cars controlled by the player
#[derive(Clone, Debug, Default, Reflect, Component)]
pub struct UserControlled;

/// Tag component used for player car
#[derive(Clone, Debug, Default, Reflect, Component)]
pub struct Player;

/// A component holding driving input for cars
#[derive(Clone, Debug, Default, Reflect, Component)]
pub struct CarControls{
    /// turning controls, left, right, assumed 0-1
    pub turn: Vec2,
    /// acceleration controls, forward, back, assumed 0-1
    pub accel: Vec2
}
impl CarControls{
    /// System which updates controls for user controlled cars
    pub fn read_player_input(
        mut controls: Query<&mut CarControls, With<UserControlled>>,
        button_inputs: Res<ButtonInput<KeyCode>>
    ){
        let accel = Vec2::new(
            if button_inputs.pressed(KeyCode::KeyW) {1.0} else {0.0},
            if button_inputs.pressed(KeyCode::KeyS) {1.0} else {0.0}
        );
        let turn = Vec2::new(
            if button_inputs.pressed(KeyCode::KeyA) {1.0} else {0.0},
            if button_inputs.pressed(KeyCode::KeyD) {1.0} else {0.0}
        );
        for mut control in controls.iter_mut(){
            control.accel = accel;
            control.turn = turn;
        }
    }
}

/// Component holding physical position data for cars on the track
#[derive(Clone, Debug, Default, Reflect, Component)]
pub struct TrackTransform{
    /// physical position on the track
    pub position: Vec2,
    /// rotation of the entity in radians
    pub rotation: f32,
    /// velocity of the entity
    pub velocity: Vec2,
    /// Rotational velocity of the entity
    pub rotational_velocity: f32
}
impl TrackTransform{
    /// System to update the world transform using the position in the track transform
    pub fn update_transform(mut cars: Query<(&mut Transform, &TrackTransform)>, track_config: Res<TrackConfig>){
        for (mut transform, track) in cars.iter_mut(){
            *transform = Transform::from_translation(track_config.track_to_world(track.position).extend(0.0))*
                Transform::from_scale(Vec3::from_slice(&[track_config.scale; 3]))*
                Transform::from_rotation(Quat::from_rotation_z(track.rotation));
        }
    }
    /// System to run physics step for Track Transform components using car controls
    pub fn update_physics(
        mut cars: Query<(&mut Self, &CarControls)>,
        config: Res<CarPhysicsConfig>,
        time: Res<Time>
    ){
        let dt = time.delta_seconds();
        for (mut transform, controls) in cars.iter_mut(){
            //Get controls to be normalized
            let accel_control = (controls.accel.x.clamp(0.0, 1.0) - controls.accel.y.clamp(0.0, 1.0)).clamp(-1.0, 1.0);
            let mut turn_control = (controls.turn.x.clamp(0.0, 1.0) - controls.turn.y.clamp(0.0, 1.0)).clamp(-1.0, 1.0);
            if turn_control.abs() < 0.0001 {turn_control = 0.0;}
            // update rotation
            if turn_control == 0.0{
                transform.rotational_velocity = 0.0;
            }else {
                transform.rotational_velocity += turn_control*config.rotational_acceleration*dt;
                transform.rotational_velocity = transform.rotational_velocity.clamp(-config.max_rotational_velocity, config.max_rotational_velocity);
                transform.rotation += transform.rotational_velocity*dt;
                transform.rotation = transform.rotation.rem_euclid(TAU);
            }
            let forward_vector = Vec2::from_angle(transform.rotation);
            // Accelerate
            let acceleration = forward_vector*accel_control*dt*config.forward_acceleration;
            transform.velocity += acceleration;
            // Apply friction
            transform.velocity *= config.friction;
            // Apply drift and clamp speed
            transform.velocity = transform.velocity.project_onto_normalized(forward_vector).lerp(transform.velocity, config.drift_factor).clamp_length_max(config.max_forward_velocity);
            // update position
            transform.position = transform.position + transform.velocity*dt;
        }
    }
}

pub fn spawn_player_car(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>){
    commands.spawn((
        Player, Car, UserControlled, CarControls::default(), TrackTransform::default(),
        MaterialMesh2dBundle{
            mesh: bevy::sprite::Mesh2dHandle(meshes.add(Rectangle{half_size: Vec2::new(5.0, 2.5)})),
            material: materials.add(ColorMaterial{color: Color::RED, texture: None}),
            ..Default::default()
        }
    ));
}

/// Adds Driving simulation components to the game
pub struct CarPlugin;
impl Plugin for CarPlugin{
    fn build(&self, app: &mut App) {
        app
            .register_type::<TrackConfig>()
            .register_type::<CarPhysicsConfig>()
            .register_type::<Car>()
            .register_type::<UserControlled>()
            .register_type::<CarControls>()
            .register_type::<TrackTransform>()
            .register_type::<Player>()
            .init_resource::<CarPhysicsConfig>()
            .init_resource::<TrackConfig>()
            .add_systems(PreUpdate, TrackConfig::update_scale)
            .add_systems(FixedUpdate, (
                CarControls::read_player_input, 
                TrackTransform::update_physics, 
                TrackTransform::update_transform
            ).chain())
            .add_systems(Startup, spawn_player_car);
        
    }
}
