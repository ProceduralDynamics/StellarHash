use bevy::prelude::*;

use StellarHash::camera;
use StellarHash::ui;
use StellarHash::univers;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "StellarHash".to_string(),
                mode: bevy::window::WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        // Custom plugins
        .add_plugins(camera::CameraPlugin)
        .add_plugins(univers::UniversPlugin)
        .add_plugins(ui::UiPlugin)
        .add_systems(Update, quit_game)
        .run();
}

// Allows you to quit the game using the Esc key
fn quit_game(
    touches: Res<ButtonInput<KeyCode>>,
    mut evenements_sortie: EventWriter<bevy::app::AppExit>,
) {
    if touches.just_pressed(KeyCode::Escape) {
        evenements_sortie.send(bevy::app::AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::AppExit;

    // --- Utility function to prepare the environment ---
    fn preparer_app_quitter() -> App {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);

        // Initialize the resource that handles keyboard input
        app.init_resource::<ButtonInput<KeyCode>>();

        app
    }

    // --- No event if the key is not pressed ---
    #[test]
    fn test_quitter_jeu_ignore_sans_echap() {
        let mut app = preparer_app_quitter();
        app.add_systems(Update, quit_game);

        // Run the system without touching the keyboard
        app.update();

        // Retrieve the AppExit event queue
        let evenements = app.world().resource::<Events<AppExit>>();
        let lecteur = evenements.get_reader();

        // The queue must be strictly empty
        assert!(
            lecteur.is_empty(evenements),
            "Aucun événement de sortie ne devrait être envoyé"
        );
    }

    // --- Send AppExit::Success event if Esc is pressed ---
    #[test]
    fn test_quitter_jeu_envoie_event_avec_echap() {
        let mut app = preparer_app_quitter();
        app.add_systems(Update, quit_game);

        // Get the keyboard resource to simulate an Escape key press
        let mut touches = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        touches.press(KeyCode::Escape);

        // Execute the system so it reads the input and generates the event.
        app.update();

        // Retrieve the event queue
        let evenements = app.world().resource::<Events<AppExit>>();
        let mut lecteur = evenements.get_reader();

        // Read the events sent during this frame
        let mut nombre_evenements = 0;
        for event in lecteur.read(evenements) {
            assert_eq!(
                *event,
                AppExit::Success,
                "The event must be AppExit::Success."
            );
            nombre_evenements += 1;
        }

        assert_eq!(
            nombre_evenements, 1,
            "Only a single exit event should have been generated."
        );
    }
}
