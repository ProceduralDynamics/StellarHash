use bevy::core_pipeline::bloom::BloomSettings;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// The plugin that encapsulates all camera logicpub struct CameraPlugin;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_camera)
            .add_systems(Update, (move_camera, zoom_camera));
    }
}

#[derive(Component)]
pub struct MainCamera;

fn initialize_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings::NATURAL,
        MainCamera,
    ));
}

fn move_camera(
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut requete_camera: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = requete_camera.single_mut();
    let mut vitesse = 500.0 * transform.scale.x;

    if touches.pressed(KeyCode::ShiftLeft) {
        vitesse *= 2.0;
    }

    let mut direction = Vec3::ZERO;
    if touches.pressed(KeyCode::ArrowLeft) || touches.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if touches.pressed(KeyCode::ArrowRight) || touches.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if touches.pressed(KeyCode::ArrowUp) || touches.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if touches.pressed(KeyCode::ArrowDown) || touches.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * vitesse * temps.delta_seconds();
    }
}

fn zoom_camera(
    mut evenements_molette: EventReader<MouseWheel>,
    mut requete_camera: Query<&mut Transform, With<MainCamera>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
) {
    let mut transform = requete_camera.single_mut();

    let mut zoom_max = 15.0;

    if let Ok(fenetre) = requete_fenetre.get_single() {
        let dimension_max = fenetre.width().max(fenetre.height());

        let distance_max_monde = 100.0 * 80.0;

        zoom_max = (distance_max_monde / (dimension_max / 2.0)) * 0.95;
    }
    for evenement in evenements_molette.read() {
        let facteur_zoom = 1.1;
        let mut nouvelle_echelle = transform.scale.x;

        if evenement.y > 0.0 {
            nouvelle_echelle /= facteur_zoom; // Zoom in
        } else if evenement.y < 0.0 {
            nouvelle_echelle *= facteur_zoom; // Zoom out
        }

        nouvelle_echelle = nouvelle_echelle.clamp(0.1, zoom_max);

        transform.scale = Vec3::splat(nouvelle_echelle);
    }
}

// --- START UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use std::time::Duration;

    // ---Start camera initialization test---
    #[test]
    fn test_initialiser_camera_cree_entite() {
        // Creating an empty Bevy application
        let mut app = App::new();

        // We add our system
        app.add_systems(Startup, initialize_camera);

        // Run the application for one frame to execute the Startup
        app.update();

        // Verify that the camera has been successfully instantiated with its components
        let mut requete = app
            .world_mut()
            .query_filtered::<&Transform, With<MainCamera>>();

        // if there isn't exactly one camera, this will panic (which is what we want in a test)
        let _transform = requete.single(app.world());
    }
    // ---End camera initialization test---

    // ---Start camera movement test---
    #[test]
    fn test_move_camera_vers_la_droite() {
        let mut app = App::new();

        // Initialize the resources required by the system
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<Time>();
        // We manually instantiate our camera
        let camera_entite = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), MainCamera))
            .id();

        app.add_systems(Update, move_camera);

        // Simulate pressing the D key (right)
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyD);

        // Simulate the passage of time (e.g., 0.1 seconds)
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(Duration::from_secs_f32(0.1));

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();

        // The speed is 500.0 * 1.0 (scale); over 0.1 sec, we should be at X = 50.0
        assert_eq!(transform.translation.x, 50.0);
        assert_eq!(transform.translation.y, 0.0);
    }
    // ---End camera movement test---

    // ---Start camera Shift movement test---
    #[test]
    fn test_move_camera_avec_multiplicateur_shift() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>();

        // We add the ": Time" annotation to guide the compiler
        let mut time: Time = Time::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);

        let camera_entite = app
            .world_mut()
            .spawn((Transform::default(), MainCamera))
            .id();
        app.add_systems(Update, move_camera);

        // Simulate pressing D (right) AND Left Shift
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyD);
        input.press(KeyCode::ShiftLeft);

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();
        // Normal speed = 50.0, with Shift (* 2.0) = 100.0
        assert_eq!(transform.translation.x, 100.0);
    }
    // ---End camera Shift movement test---

    // ---Start top and bottom zoom test ---
    #[test]
    fn test_zoom_camera_molette_haut_et_bas() {
        let mut app = App::new();

        // The system needs to read MouseWheel events
        app.add_event::<MouseWheel>();

        let camera_entite = app
            .world_mut()
            .spawn((Transform::default(), MainCamera))
            .id();
        app.add_systems(Update, zoom_camera);

        // --- ZOOM IN (y > 0.0) ---
        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window: Entity::PLACEHOLDER,
        });

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();
        // Zoom IN = divide by 1.1. The default scale is 1.0. (1.0 / 1.1 = ~0.909)
        assert!(transform.scale.x < 1.0);

        // --- ZOOM OUT (y < 0.0) ---
        // Emit an event to zoom out
        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window: Entity::PLACEHOLDER,
        });

        app.update();

        let transform_final = app.world().get::<Transform>(camera_entite).unwrap();
        // We should be back to around 1.0 (0.909 * 1.1)
        let difference = (transform_final.scale.x - 1.0).abs();
        assert!(difference < 0.0001, "L'échelle devrait être revenue à 1.0");
    }
    // ---End top and bottom zoom test ---

    // ---Start of zoom limit test---
    #[test]
    fn test_zoom_camera_respecte_les_limites() {
        let mut app = App::new();
        app.add_event::<MouseWheel>();

        let mut fenetre = Window::default();
        fenetre.resolution.set(800.0, 600.0);
        app.world_mut().spawn((fenetre, PrimaryWindow));

        let camera_entite = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(19.0)), MainCamera))
            .id();

        app.add_systems(Update, zoom_camera);

        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window: Entity::PLACEHOLDER,
        });

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();

        assert_eq!(transform.scale.x, 19.0);
    }
    // ---End of zoom limit test---
}
// --- END UNIT TESTS ---
