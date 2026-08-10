use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::time::SystemTime;

use crate::astrophysique::StellarSystem;
use crate::camera::MainCamera;
use crate::univers::GlobalSeed;
use crate::univers::Star;

const ANECDOTE_FILE: &str = include_str!("../assets/anecdotes.txt");

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .insert_resource(ChronoAnecdote(Timer::from_seconds(
                60.0,
                TimerMode::Repeating,
            )))
            .add_systems(
                Startup,
                (
                    initialize_fps,
                    initialize_seed_display,
                    initialize_info_panel,
                    initialize_trivia_panel,
                ),
            )
            .add_systems(Update, (update_fps, manage_mouse_hover, update_anecdotes));
    }
}

#[derive(Component)]
struct TexteFps;

#[derive(Component)]
pub struct PanneauInfo;

#[derive(Component)]
pub struct TexteInfo;

#[derive(Resource)]
struct ChronoAnecdote(Timer);

#[derive(Component)]
struct TexteAnecdote;

fn initialize_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");
    // We create an interface box anchored to the top left.
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // We add our text inside this box
            parent.spawn((
                TextBundle::from_section(
                    "FPS: calculation...",
                    TextStyle {
                        font: police,
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ),
                TexteFps,
            ));
        });
}

fn initialize_seed_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    graine: Res<GlobalSeed>,
) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Seed: ",
                    TextStyle {
                        font: police.clone(),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    graine.0.to_string(),
                    TextStyle {
                        font: police,
                        font_size: 24.0,
                        color: Color::srgba(0.4, 0.8, 1.0, 1.0),
                    },
                ),
            ]));
        });
}

fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut requete_texte: Query<&mut Text, With<TexteFps>>,
) {
    for mut texte in &mut requete_texte {
        // Retrieve the FPS data from the engine
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            // smoothed() returns a smoothed average
            if let Some(valeur) = fps.smoothed() {
                texte.sections[0].value = format!("FPS: {:.1}", valeur);
            }
        }
    }
}

fn initialize_info_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 1.0)),
                ..default()
            },
            PanneauInfo,
        ))
        .with_children(|parent| {
            // The text inside
            parent.spawn((
                TextBundle::from_section(
                    "Stellar Data",
                    TextStyle {
                        font: police,
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ),
                TexteInfo,
            ));
        });
}

pub fn manage_mouse_hover(
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

    requete_etoiles: Query<(Entity, &Transform, &StellarSystem, &Star)>,
    mut requete_panneau: Query<&mut Style, With<PanneauInfo>>,
    mut requete_texte: Query<&mut Text, With<TexteInfo>>,

    mut derniere_etoile_survolee: Local<Option<Entity>>,
) {
    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    // We check whether the mouse is actually inside the window.
    if let Some(position_curseur_ecran) = fenetre.cursor_position() {
        // Screen pixels are converted into 2D world coordinates (mathematical raycasting).
        if let Some(position_monde) =
            camera.viewport_to_world_2d(camera_transform, position_curseur_ecran)
        {
            let mut etoile_survolee = None;

            // We calculate the square where the mouse is located.
            let taille_secteur = 80.0;
            let souris_grille_x = (position_monde.x / taille_secteur).round() as i32;
            let souris_grille_y = (position_monde.y / taille_secteur).round() as i32;

            // We are testing all the currently displayed stars to see if the mouse is over them.
            for (entite, transform_etoile, systeme, etoile) in requete_etoiles.iter() {
                // Any stars not located within the mouse's grid cell or the adjacent cells are immediately disregarded.
                if (etoile.grille_x - souris_grille_x).abs() <= 1
                    && (etoile.grille_y - souris_grille_y).abs() <= 1
                {
                    let distance = position_monde.distance(transform_etoile.translation.truncate());

                    if distance < 25.0 {
                        etoile_survolee = Some((entite, systeme));
                        break;
                    }
                }
            }

            let mut style_panneau = requete_panneau.single_mut();
            let mut texte = requete_texte.single_mut();

            // Hovering over a star updates the text and displays the panel beneath the mouse.
            if let Some((entite_actuelle, systeme)) = etoile_survolee {
                style_panneau.display = Display::Flex;

                // Shift the panel slightly so it isn't hidden by the mouse cursor.
                style_panneau.left = Val::Px(position_curseur_ecran.x + 15.0);
                style_panneau.top = Val::Px(position_curseur_ecran.y + 15.0);

                if *derniere_etoile_survolee != Some(entite_actuelle) {
                    texte.sections[0].value = format!(
                        "System: {}\nClass: {:?}\nSolar Mass: {:.2} MS\nPlanets: {}\nAge: {:.1} Ga",
                        systeme.nom,
                        systeme.classe,
                        systeme.masse_solaire,
                        systeme.nb_planetes,
                        systeme.age_milliards_annees
                    );

                    // Update the history
                    *derniere_etoile_survolee = Some(entite_actuelle);
                }
            } else {
                // If in the vacuum of space, hide the panel
                style_panneau.display = Display::None;
            }
        }
    }
}

fn initialize_trivia_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");

    let lignes: Vec<&str> = ANECDOTE_FILE.lines().filter(|l| !l.is_empty()).collect();
    let texte_initial = if !lignes.is_empty() {
        let temps_actuel = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;

        let index_aleatoire = temps_actuel % lignes.len();

        lignes[index_aleatoire]
    } else {
        "Empty database."
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Percent(40.0),
                max_width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 1.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    texte_initial,
                    TextStyle {
                        font: police,
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                TexteAnecdote,
            ));
        });
}

fn update_anecdotes(
    temps: Res<Time>,
    mut chrono: ResMut<ChronoAnecdote>,
    mut requete_texte: Query<&mut Text, With<TexteAnecdote>>,
) {
    chrono.0.tick(temps.delta());

    if chrono.0.just_finished() {
        let lignes: Vec<&str> = ANECDOTE_FILE.lines().filter(|l| !l.is_empty()).collect();

        if lignes.is_empty() {
            return;
        }

        let temps_actuel = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        let index_aleatoire = temps_actuel % lignes.len();

        // Update the text
        let mut texte = requete_texte.single_mut();
        texte.sections[0].value = format!("{}", lignes[index_aleatoire]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::diagnostic::Diagnostic;

    #[test]
    fn test_initialize_fps_cree_composants() {
        // Create a new empty Bevy App
        let mut app = App::new();

        // Add the AssetServer plugin (required for loading fonts)
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        // Add our system to the Startup schedule
        app.add_systems(Startup, initialize_fps);

        // Run the app for one frame to execute the Startup schedule
        app.update();

        // Query to check if a node with TexteFps component was created
        let mut requete_texte = app.world_mut().query_filtered::<&Text, With<TexteFps>>();

        // If the query fails to find exactly one entity, the test will panic
        let texte = requete_texte.single(app.world());

        // Verify that the initial text is correct
        assert_eq!(texte.sections[0].value, "FPS: calculation...");
    }

    #[test]
    fn test_initialize_fps_position_correcte() {
        let mut app = App::new();

        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        app.add_systems(Startup, initialize_fps);
        app.update();

        // The TexteFps is a child of the main NodeBundle.
        // We need to find the parent node and check its style.
        // First, let's find the entity with TexteFps
        let mut query = app.world_mut().query_filtered::<Entity, With<TexteFps>>();
        let entity_texte = query.single(app.world());

        // Now, get its Parent component
        let parent_component = app.world().get::<Parent>(entity_texte).unwrap();
        let entity_parent = parent_component.get();

        // Finally, get the Style component of the parent node
        let style = app.world().get::<Style>(entity_parent).unwrap();

        // Assert the positioning values we set in the function
        assert_eq!(style.position_type, PositionType::Absolute);
        assert_eq!(style.top, Val::Px(10.0));
        assert_eq!(style.left, Val::Px(10.0));
    }

    #[test]
    fn test_mettre_a_jour_fps_affiche_valeur_arrondie() {
        let mut app = App::new();

        // Instantiate the text entity with the required components
        let entite = app
            .world_mut()
            .spawn((
                TextBundle::from_section("FPS: calculation...", TextStyle::default()),
                TexteFps,
            ))
            .id();

        // Simulate engine diagnostics
        let mut diagnostics = DiagnosticsStore::default();
        let mut diagnostic_fps = Diagnostic::new(FrameTimeDiagnosticsPlugin::FPS);

        // Manually inject a dummy FPS measurement (e.g., 60.48)
        diagnostic_fps.add_measurement(bevy::diagnostic::DiagnosticMeasurement {
            time: bevy::utils::Instant::now(),
            value: 60.48,
        });
        diagnostics.add(diagnostic_fps);

        // We insert this mock database into the test application.
        app.insert_resource(diagnostics);

        // Add and execute our system
        app.add_systems(Update, update_fps);
        app.update();

        // Check the result
        let texte = app.world().get::<Text>(entite).unwrap();

        // Your system's {:.1} format must round 60.48 to 60.5
        assert_eq!(texte.sections[0].value, "FPS: 60.5");
    }

    #[test]
    fn test_mettre_a_jour_fps_ignore_si_pas_de_donnees() {
        let mut app = App::new();

        // Start with the default text
        let entite = app
            .world_mut()
            .spawn((
                TextBundle::from_section("FPS: calculation...", TextStyle::default()),
                TexteFps,
            ))
            .id();

        // We prepare the FPS container, but this time we add NO measurements.
        let mut diagnostics = DiagnosticsStore::default();
        let diagnostic_fps = Diagnostic::new(FrameTimeDiagnosticsPlugin::FPS);
        diagnostics.add(diagnostic_fps);
        app.insert_resource(diagnostics);

        app.add_systems(Update, update_fps);
        app.update();

        // Retrieve the text after system execution
        let texte = app.world().get::<Text>(entite).unwrap();

        // Since `fps.smoothed()` returned None, the text must not have changed.
        assert_eq!(texte.sections[0].value, "FPS: calculation...");
    }

    #[test]
    fn test_initialiser_panneau_info_est_cache_par_defaut() {
        let mut app = App::new();

        // Essential configuration to load the font without errors
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        // Adding and executing the system
        app.add_systems(Startup, initialize_info_panel);
        app.update();

        // Retrieve the Style component from our information panel
        let mut requete_panneau = app
            .world_mut()
            .query_filtered::<&Style, With<PanneauInfo>>();

        // If there isn't exactly one PanneauInfo, the test will fail.
        let style_panneau = requete_panneau.single(app.world());

        // CRITICAL CHECKS:
        // The panel MUST be invisible (Display::None) at startup
        assert_eq!(style_panneau.display, Display::None);

        // The panel must have absolute positioning to float on the screen
        assert_eq!(style_panneau.position_type, PositionType::Absolute);

        // Internal elements must be stacked in a column
        assert_eq!(style_panneau.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_initialiser_panneau_info_cree_le_texte_enfant() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        app.add_systems(Startup, initialize_info_panel);
        app.update();

        // Look for the entity that has the TexteInfo marker
        let mut requete_texte = app.world_mut().query_filtered::<&Text, With<TexteInfo>>();
        let texte = requete_texte.single(app.world());

        // Check the default content
        assert_eq!(texte.sections[0].value, "Stellar Data");

        // Check the text style (size and color)
        assert_eq!(texte.sections[0].style.font_size, 18.0);
        assert_eq!(texte.sections[0].style.color, Color::WHITE);
    }
}
