pub use bevy::prelude::*;

#[derive(Debug, Default, Clone, Reflect, PartialEq, Eq, Hash, States, Component)]
pub enum AppState{
    #[default] PlayerControl,
    TrackCreation,
    TrackRendering
}

#[derive(Debug, Clone, Reflect, Resource)]
pub struct Canvas{
    half_extents: Vec2,
    button_width: f32,
    scale: f32,
    buttons: Vec<&'static str>
}
impl Default for Canvas{
    fn default() -> Self {
        Self{
            half_extents: Vec2::new(800.0, 450.0), button_width: 150.0, scale: 1.0,
            buttons: vec!["Play", "Create Track", "Finish Track"]
        }
    }
}
impl Canvas{
    /// Scales parts of the canvas to the correct size, according to the canvas's scale
    pub fn scale_canvas_elements(
        mut items: Query<&mut Style, With<CanvasRoot>>, 
        canvas: Res<Self>,
        mut words: Query<(&mut Text, &CanvasText)>
    ){
        let resolution = (canvas.half_extents*2.0 + Vec2::X*canvas.button_width)*canvas.scale;
        for mut style in items.iter_mut(){
            style.width = Val::Px(resolution.x);
            style.height = Val::Px(resolution.y);
        }
        for (mut text, size) in words.iter_mut(){
            text.sections[0].style.font_size = size.0*canvas.scale;
        }
    }
    /// Sets the canvas scale to be the largest possible every frame
    pub fn update_scale(mut canvas: ResMut<Canvas>, windows: Query<&Window>){
        let Ok(window) = windows.get_single() else {return;};
        canvas.scale = (window.height() / (canvas.half_extents.y*2.0)).min(window.width() / (canvas.half_extents.x*2.0 + canvas.button_width));
    }
    pub fn update_button_colors(
        
    ){}
}

#[derive(Default, Debug, Clone, Reflect, Component)]
pub struct CanvasRoot;

#[derive(Default, Debug, Clone, Reflect, Component)]
pub struct CanvasText(pub f32);

#[derive(Default, Debug, Clone, Reflect, Component)]
pub struct ButtonName(pub &'static str);

pub fn spawn_menu(
    mut commands: Commands,
    canvas: Res<Canvas>
){
    // Spawn UI
    commands.spawn((
        NodeBundle{
            style: Style{
                width: Val::Px(canvas.half_extents.x*2.0 + canvas.button_width),
                height: Val::Px(canvas.half_extents.y*2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        CanvasRoot
    )).with_children(|parent| {
        // Spawn buttons
        parent.spawn(NodeBundle{
            style: Style{
                width: Val::Percent(100.0 * canvas.button_width / (canvas.half_extents.x*2.0+canvas.button_width)),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        }).with_children(|parent| {
            let button_height = (canvas.buttons.len() as f32).recip() * 100.0;
            for button_name in canvas.buttons.iter(){
                parent.spawn((
                    ButtonBundle{
                        style: Style{
                            width: Val::Percent(100.0),
                            height: Val::Percent(button_height),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            border: UiRect::all(Val::Percent(5.0)),
                            ..Default::default()
                        },
                        background_color: BackgroundColor(Color::WHITE),
                        border_color: BorderColor(Color::BLACK),
                        ..Default::default()
                    }, ButtonName(button_name.to_owned())
                )).with_children(|parent| {
                    parent.spawn((TextBundle::from_section(
                        button_name.to_owned(),
                        TextStyle {
                            font_size: 20.0,
                            color: Color::DARK_GRAY,
                            ..Default::default()
                        },
                    ), CanvasText(20.0)));
                });
            }
        });
        // Spawn track area
        parent.spawn(NodeBundle{
            style: Style{
                width: Val::Percent(100.0 * canvas.half_extents.x * 2.0 / (canvas.half_extents.x*2.0+canvas.button_width)),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        });
    });
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin{
    fn build(&self, app: &mut App) {
        app.init_resource::<Canvas>()
        .init_state::<AppState>()
        .add_systems(Startup, spawn_menu)
        .add_systems(PostUpdate, Canvas::scale_canvas_elements)
        .add_systems(PreUpdate, Canvas::update_scale);
    }
}
