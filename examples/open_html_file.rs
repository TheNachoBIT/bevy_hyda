use bevy::prelude::*;
use bevy_hyda::BevyHydaPlugin;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {

    let mut camera = Camera2dBundle::default();
    commands.spawn(camera);

    let open_html = bevy_hyda::html_file("assets/test.html".to_string());
    open_html.spawn_ui(&mut commands, &asset_server);
}

fn main() {
    App::new()
        .add_plugins((
                DefaultPlugins.set(ImagePlugin::default_nearest()),
                BevyHydaPlugin,
            ))
        .add_systems(Startup, setup)
        .run();
}
