use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;

use bevy::{
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

use crate::camera::MainCamera;
use crate::generation;

pub struct UniversPlugin;

impl Plugin for UniversPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedSectors>()
            .add_plugins((
                Material2dPlugin::<StarMaterial>::default(),
                Material2dPlugin::<GiantStarMaterial>::default(),
            ))
            .add_systems(Startup, initialize_star_assets)
            .add_systems(
                Update,
                (
                    generate_dynamic_universe,
                    spatial_garbage_collector,
                    handle_star_click,
                    animate_orbits,
                    handle_planet_lod,
                    handle_star_lod.before(spatial_garbage_collector),
                    // --- unused function ---
                    // animate_star_scale,
                ),
            );
    }
}

#[derive(Resource)]
pub struct GlobalSeed(pub u32);

#[derive(Resource, Default)]
pub struct LoadedSectors(pub HashMap<(i32, i32), Option<Entity>>);

#[derive(Component)]
pub struct Star {
    pub grille_x: i32,
    pub grille_y: i32,
}

#[derive(Component)]
pub struct ExpandedSystem;

#[derive(Component)]
pub struct Planet {
    pub rayon_orbite: f32,
    pub angle_actuel: f32,
    pub vitesse_orbite: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct StarMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
}

impl Material2d for StarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/star.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GiantStarMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(0)]
    pub seed: f32,
}

impl Material2d for GiantStarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/plasma.wgsl".into()
    }
}

#[derive(Component)]
pub struct GiantStar {
    pub mat_lourd: Handle<GiantStarMaterial>,
    pub mat_leger: Handle<StarMaterial>,
    pub est_lourd: bool,
}

#[derive(Resource)]
pub struct StarAssets {
    pub mesh_base: Handle<Mesh>,

    pub mat_o_light: Handle<StarMaterial>,
    pub mat_b_light: Handle<StarMaterial>,

    pub mat_a: Handle<StarMaterial>,
    pub mat_f: Handle<StarMaterial>,
    pub mat_g: Handle<StarMaterial>,
    pub mat_k: Handle<StarMaterial>,
    pub mat_m: Handle<StarMaterial>,
}

// unused function
pub fn animate_star_scale(
    temps: Res<Time>,
    mut requete_etoiles: Query<(&Star, &crate::astrophysique::StellarSystem, &mut Transform)>,
) {
    let temps_ecoule = temps.elapsed_seconds();

    for (etoile, systeme, mut transform) in requete_etoiles.iter_mut() {
        let world_x = etoile.grille_x as f32 * 80.0;
        let world_y = etoile.grille_y as f32 * 80.0;

        let seed = world_x * 0.1337 + world_y * 0.7331;

        let twinkle = ((temps_ecoule * 3.0 + seed).sin() + 1.0) * 0.5;

        let taille_visuelle = 8.0 + (systeme.rayon_solaire * 4.0);
        let rayon_base = taille_visuelle / 2.0;

        let echelle = rayon_base * (1.0 + twinkle * 0.00);
        transform.scale = Vec3::splat(echelle);
    }
}

fn initialize_star_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_standard: ResMut<Assets<StarMaterial>>,
) {
    let hdr = 15.0;

    commands.insert_resource(StarAssets {
        mesh_base: meshes.add(Circle::new(1.0)),

        // Initialization of the giants (Light – their colors are identical)
        mat_o_light: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(0.3 * hdr, 0.5 * hdr, 1.0 * hdr, 1.0),
        }),
        mat_b_light: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(0.6 * hdr, 0.8 * hdr, 1.0 * hdr, 1.0),
        }),

        // Initialization of standards
        mat_a: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(1.0 * hdr, 1.0 * hdr, 1.0 * hdr, 1.0),
        }),
        mat_f: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(1.0 * hdr, 1.0 * hdr, 0.8 * hdr, 1.0),
        }),
        mat_g: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(1.0 * hdr, 0.9 * hdr, 0.2 * hdr, 1.0),
        }),
        mat_k: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(1.0 * hdr, 0.5 * hdr, 0.1 * hdr, 1.0),
        }),
        mat_m: materials_standard.add(StarMaterial {
            base_color: LinearRgba::new(0.9 * hdr, 0.2 * hdr, 0.2 * hdr, 1.0),
        }),
    });
}

fn generate_dynamic_universe(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<MainCamera>>,
    graine: Res<GlobalSeed>,
    mut secteurs_charges: ResMut<LoadedSectors>,
    star_assets: Res<StarAssets>,
    mut materials_giant: ResMut<Assets<GiantStarMaterial>>,
    mut derniere_pos_maj: Local<Vec2>,
    mut dernier_zoom_maj: Local<f32>,
) {
    let camera_transform = requete_camera.single();
    let pos_actuelle = camera_transform.translation.truncate();
    let zoom = camera_transform.scale.x;

    if pos_actuelle.distance(*derniere_pos_maj) < 40.0 && (zoom - *dernier_zoom_maj).abs() < 0.1 {
        return;
    }

    *derniere_pos_maj = pos_actuelle;
    *dernier_zoom_maj = zoom;

    let taille_secteur = 80.0;
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);

    let centre_grille_x = (pos_actuelle.x / taille_secteur).round() as i32;
    let centre_grille_y = (pos_actuelle.y / taille_secteur).round() as i32;

    for x in (centre_grille_x - rayon_vision)..=(centre_grille_x + rayon_vision) {
        for y in (centre_grille_y - rayon_vision)..=(centre_grille_y + rayon_vision) {
            if secteurs_charges.0.contains_key(&(x, y)) {
                continue;
            }

            // the probability density map of base occurrence.
            let densite_brute = generation::get_macro_density(x, y, graine.0.wrapping_add(999));

            let densite_contrastee = ((densite_brute - 0.5) * 2.0) + 0.5;
            let densite_contrastee = densite_contrastee.clamp(0.0, 1.0);

            let densite_lissee =
                densite_contrastee * densite_contrastee * (3.0 - 2.0 * densite_contrastee);

            let seuil_apparition = 1.05 - (densite_lissee * 0.25);
            let probabilite = generation::calculate_spatial_hash(x, y, graine.0);
            let mut entite_etoile = None;

            if probabilite > seuil_apparition {
                let systeme_stellaire =
                    crate::astrophysique::generate_characteristics(x, y, probabilite);

                let taille_visuelle = 8.0 + (systeme_stellaire.rayon_solaire * 4.0);
                let rayon_final = taille_visuelle / 2.0;

                // Local offset
                let proba_offset_x =
                    generation::calculate_spatial_hash(x, y, graine.0.wrapping_add(1234));
                let proba_offset_y =
                    generation::calculate_spatial_hash(x, y, graine.0.wrapping_add(5678));

                let offset_x = (proba_offset_x - 0.5) * (taille_secteur * 0.8);
                let offset_y = (proba_offset_y - 0.5) * (taille_secteur * 0.8);

                let pos_x = (x as f32 * taille_secteur) + offset_x;
                let pos_y = (y as f32 * taille_secteur) + offset_y;

                let transform =
                    Transform::from_xyz(pos_x, pos_y, 0.0).with_scale(Vec3::splat(rayon_final));

                let mesh = Mesh2dHandle(star_assets.mesh_base.clone());
                let composant_etoile = Star {
                    grille_x: x,
                    grille_y: y,
                };

                // Routing based on material type
                let entite = match systeme_stellaire.classe {
                    crate::astrophysique::SpectralClass::O
                    | crate::astrophysique::SpectralClass::B => {
                        let hdr = 15.0;

                        // SEED GENERATION: Unique to this precise stellar coordinate
                        let seed_etoile = (x as f32) * 0.1337 + (y as f32) * 0.7331;

                        // Creating the unique heavy material on the fly
                        let (mat_lourd, mat_leger) =
                            if systeme_stellaire.classe == crate::astrophysique::SpectralClass::O {
                                (
                                    materials_giant.add(GiantStarMaterial {
                                        base_color: LinearRgba::new(
                                            0.3 * hdr,
                                            0.5 * hdr,
                                            1.0 * hdr,
                                            1.0,
                                        ),
                                        seed: seed_etoile,
                                    }),
                                    star_assets.mat_o_light.clone(),
                                )
                            } else {
                                (
                                    materials_giant.add(GiantStarMaterial {
                                        base_color: LinearRgba::new(
                                            0.6 * hdr,
                                            0.8 * hdr,
                                            1.0 * hdr,
                                            1.0,
                                        ),
                                        seed: seed_etoile,
                                    }),
                                    star_assets.mat_b_light.clone(),
                                )
                            };

                        commands
                            .spawn((
                                MaterialMesh2dBundle {
                                    mesh,
                                    material: mat_lourd.clone(),
                                    transform,
                                    ..default()
                                },
                                composant_etoile,
                                systeme_stellaire,
                                GiantStar {
                                    mat_lourd,
                                    mat_leger,
                                    est_lourd: true,
                                },
                            ))
                            .id()
                    }
                    _ => {
                        let material = match systeme_stellaire.classe {
                            crate::astrophysique::SpectralClass::A => star_assets.mat_a.clone(),
                            crate::astrophysique::SpectralClass::F => star_assets.mat_f.clone(),
                            crate::astrophysique::SpectralClass::G => star_assets.mat_g.clone(),
                            crate::astrophysique::SpectralClass::K => star_assets.mat_k.clone(),
                            crate::astrophysique::SpectralClass::M => star_assets.mat_m.clone(),
                            _ => unreachable!(),
                        };
                        commands
                            .spawn((
                                MaterialMesh2dBundle {
                                    mesh,
                                    material,
                                    transform,
                                    ..default()
                                },
                                composant_etoile,
                                systeme_stellaire,
                            ))
                            .id()
                    }
                };

                entite_etoile = Some(entite);
            }

            secteurs_charges.0.insert((x, y), entite_etoile);
        }
    }
}

fn handle_star_lod(
    mut commands: Commands,
    requete_camera: Query<&Transform, (With<MainCamera>, Changed<Transform>)>,
    mut requete_etoiles_geantes: Query<(Entity, &mut GiantStar)>,
) {
    if let Ok(camera_transform) = requete_camera.get_single() {
        let zoom = camera_transform.scale.x;
        let seuil_lod = 4.0;

        let veut_lourd = zoom <= seuil_lod;

        for (entite, mut geante) in requete_etoiles_geantes.iter_mut() {
            if geante.est_lourd != veut_lourd {
                if let Some(mut cmd) = commands.get_entity(entite) {
                    if veut_lourd {
                        cmd.remove::<Handle<StarMaterial>>()
                            .insert(geante.mat_lourd.clone());
                    } else {
                        cmd.remove::<Handle<GiantStarMaterial>>()
                            .insert(geante.mat_leger.clone());
                    }
                    geante.est_lourd = veut_lourd;
                }
            }
        }
    }
}

pub fn spatial_garbage_collector(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<MainCamera>>,
    mut secteurs_charges: ResMut<LoadedSectors>,
    mut derniere_pos_maj: Local<Vec2>,
    mut dernier_zoom_maj: Local<f32>,
    mut ancienne_zone: Local<Option<(i32, i32, i32)>>,
) {
    let camera_transform = requete_camera.single();
    let pos_actuelle = camera_transform.translation.truncate();
    let zoom = camera_transform.scale.x;

    if pos_actuelle.distance(*derniere_pos_maj) < 40.0 && (zoom - *dernier_zoom_maj).abs() < 0.1 {
        return;
    }

    *derniere_pos_maj = pos_actuelle;
    *dernier_zoom_maj = zoom;

    let taille_secteur = 80.0;
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);
    let rayon_despawn = rayon_vision + 5;

    let centre_grille_x = (pos_actuelle.x / taille_secteur).round() as i32;
    let centre_grille_y = (pos_actuelle.y / taille_secteur).round() as i32;

    let min_x_nouveau = centre_grille_x - rayon_despawn;
    let max_x_nouveau = centre_grille_x + rayon_despawn;
    let min_y_nouveau = centre_grille_y - rayon_despawn;
    let max_y_nouveau = centre_grille_y + rayon_despawn;

    if let Some((ancien_x, ancien_y, ancien_rayon)) = *ancienne_zone {
        let min_x_ancien = ancien_x - ancien_rayon;
        let max_x_ancien = ancien_x + ancien_rayon;
        let min_y_ancien = ancien_y - ancien_rayon;
        let max_y_ancien = ancien_y + ancien_rayon;

        for x in min_x_ancien..=max_x_ancien {
            for y in min_y_ancien..=max_y_ancien {
                if x < min_x_nouveau || x > max_x_nouveau || y < min_y_nouveau || y > max_y_nouveau
                {
                    if let Some(opt_entite) = secteurs_charges.0.remove(&(x, y)) {
                        if let Some(entite) = opt_entite {
                            commands.entity(entite).despawn_recursive();
                        }
                    }
                }
            }
        }
    } else {
        secteurs_charges.0.retain(|&(x, y), &mut opt_entite| {
            let est_proche = x >= min_x_nouveau
                && x <= max_x_nouveau
                && y >= min_y_nouveau
                && y <= max_y_nouveau;

            if !est_proche {
                if let Some(entite) = opt_entite {
                    commands.entity(entite).despawn_recursive();
                }
            }
            est_proche
        });
    }

    *ancienne_zone = Some((centre_grille_x, centre_grille_y, rayon_despawn));
}

fn handle_planet_lod(
    requete_camera: Query<&Transform, (With<MainCamera>, Changed<Transform>)>,
    mut requete_planetes: Query<&mut Visibility, With<Planet>>,
) {
    if let Ok(camera_transform) = requete_camera.get_single() {
        let zoom = camera_transform.scale.x;
        let seuil_lod = 3.5;
        let visibilite_voulue = if zoom > seuil_lod {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };

        for mut visibilite in requete_planetes.iter_mut() {
            if *visibilite != visibilite_voulue {
                *visibilite = visibilite_voulue;
            }
        }
    }
}

fn handle_star_click(
    mut commands: Commands,
    touches_souris: Res<ButtonInput<MouseButton>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    requete_etoiles: Query<(
        Entity,
        &Transform,
        &crate::astrophysique::StellarSystem,
        Option<&ExpandedSystem>,
        &Star,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !touches_souris.just_pressed(MouseButton::Left) {
        return;
    }

    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    if let Some(position_curseur) = fenetre.cursor_position() {
        if let Some(position_monde) =
            camera.viewport_to_world_2d(camera_transform, position_curseur)
        {
            let taille_secteur = 80.0;
            let clic_grille_x = (position_monde.x / taille_secteur).round() as i32;
            let clic_grille_y = (position_monde.y / taille_secteur).round() as i32;

            for (entite, transform_etoile, systeme, developpe, etoile) in requete_etoiles.iter() {
                if (etoile.grille_x - clic_grille_x).abs() > 1
                    || (etoile.grille_y - clic_grille_y).abs() > 1
                {
                    continue;
                }

                if position_monde.distance(transform_etoile.translation.truncate()) < 25.0 {
                    if developpe.is_none() {
                        let taille_visuelle = 16.0 + (systeme.rayon_solaire * 4.0);
                        let echelle_etoile = taille_visuelle / 2.0;

                        commands
                            .entity(entite)
                            .insert(ExpandedSystem)
                            .with_children(|parent| {
                                for i in 0..systeme.nb_planetes {
                                    let rayon_orbite_voulu = 15.0 + (i as f32 * 5.0);
                                    let rayon_orbite = rayon_orbite_voulu / echelle_etoile;

                                    let angle_depart = (i as f32) * 1.2;
                                    let vitesse = 1.5 / (i as f32 + 1.0);

                                    let echelle_planete = 1.0 / echelle_etoile;

                                    parent.spawn((
                                        MaterialMesh2dBundle {
                                            mesh: Mesh2dHandle(meshes.add(Circle::new(2.0))),
                                            material: materials.add(ColorMaterial::from(
                                                Color::srgb(0.6, 0.8, 0.9),
                                            )),
                                            transform: Transform::from_xyz(
                                                rayon_orbite * angle_depart.cos(),
                                                rayon_orbite * angle_depart.sin(),
                                                1.0,
                                            )
                                            .with_scale(Vec3::splat(echelle_planete)),
                                            ..default()
                                        },
                                        Planet {
                                            rayon_orbite,
                                            angle_actuel: angle_depart,
                                            vitesse_orbite: vitesse,
                                        },
                                    ));
                                }
                            });
                    } else {
                        commands.entity(entite).remove::<ExpandedSystem>();
                        commands.entity(entite).despawn_descendants();
                    }
                    break;
                }
            }
        }
    }
}

pub fn animate_orbits(
    temps: Res<Time>,
    mut requete_planetes: Query<(&mut Transform, &mut Planet)>,
) {
    for (mut transform, mut planete) in requete_planetes.iter_mut() {
        planete.angle_actuel += planete.vitesse_orbite * temps.delta_seconds();
        transform.translation.x = planete.rayon_orbite * planete.angle_actuel.cos();
        transform.translation.y = planete.rayon_orbite * planete.angle_actuel.sin();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrophysique::{SpectralClass, StellarSystem};
    use bevy::render::camera::CameraProjection;
    use bevy::window::PrimaryWindow;
    use std::time::Duration;

    // --- Utility function to prepare the environment ---
    fn preparer_app() -> App {
        let mut app = App::new();

        app.add_plugins((MinimalPlugins, AssetPlugin::default()));

        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();
        app.init_asset::<StarMaterial>();
        app.init_asset::<GiantStarMaterial>();

        app.insert_resource(GlobalSeed(42));
        app.init_resource::<LoadedSectors>();

        app.add_systems(Startup, initialize_star_assets);

        app.world_mut()
            .spawn((Camera2dBundle::default(), MainCamera));

        app.update();

        app
    }

    // --- Utility function for the Garbage Collector ---
    fn preparer_app_gc() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<LoadedSectors>();
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), MainCamera));
        app
    }

    // --- Utility function for clicks ---
    fn preparer_app_clic() -> App {
        let mut app = App::new();

        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();
        app.init_resource::<ButtonInput<MouseButton>>();

        let mut fenetre = Window::default();
        fenetre.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
        app.world_mut().spawn((fenetre, PrimaryWindow));

        let mut camera_bundle = Camera2dBundle::default();
        camera_bundle.projection.update(800.0, 600.0);
        app.world_mut().spawn((camera_bundle, MainCamera));

        app
    }

    // --- Utility function for LOD (zoom) ---
    fn preparer_app_lod() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn test_generer_univers_charge_les_premiers_secteurs() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);
        assert!(app.world().resource::<LoadedSectors>().0.is_empty());
        app.update();
        let secteurs = app.world().resource::<LoadedSectors>();
        assert!(!secteurs.0.is_empty());
    }

    #[test]
    fn test_generer_univers_ignore_les_micro_mouvements() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);
        app.update();
        let secteurs_avant = app.world().resource::<LoadedSectors>().0.len();

        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 15.0;

        app.update();
        let secteurs_apres = app.world().resource::<LoadedSectors>().0.len();
        assert_eq!(
            secteurs_avant, secteurs_apres,
            "The function should have returned early without loading new sectors."
        );
    }

    #[test]
    fn test_generer_univers_reprend_apres_grand_deplacement() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);
        app.update();
        let secteurs_avant = app.world().resource::<LoadedSectors>().0.len();

        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 1000.0;
        transform.translation.y += 1000.0;

        app.update();
        let secteurs_apres = app.world().resource::<LoadedSectors>().0.len();
        assert!(
            secteurs_apres > secteurs_avant,
            "The camera teleported 1,000 units; new sectors should have been loaded."
        );
    }

    #[test]
    fn test_garbage_collector_nettoie_hors_limites() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, spatial_garbage_collector);

        let entite_proche = app
            .world_mut()
            .spawn(Star {
                grille_x: 2,
                grille_y: 2,
            })
            .id();
        let entite_lointaine = app
            .world_mut()
            .spawn(Star {
                grille_x: 50,
                grille_y: 50,
            })
            .id();

        let mut secteurs = app.world_mut().resource_mut::<LoadedSectors>();
        secteurs.0.insert((2, 2), Some(entite_proche));
        secteurs.0.insert((50, 50), Some(entite_lointaine));

        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 100.0;

        app.update();

        assert!(app.world().get_entity(entite_proche).is_some());
        assert!(app.world().get_entity(entite_lointaine).is_none());

        let secteurs_apres = app.world().resource::<LoadedSectors>();
        assert!(secteurs_apres.0.contains_key(&(2, 2)));
        assert!(!secteurs_apres.0.contains_key(&(50, 50)));
    }

    #[test]
    fn test_garbage_collector_ignore_les_micro_mouvements() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, spatial_garbage_collector);

        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 1000.0;
        app.update();

        let entite_lointaine = app
            .world_mut()
            .spawn(Star {
                grille_x: 9999,
                grille_y: 9999,
            })
            .id();

        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x += 10.0;
        app.update();

        assert!(
            app.world().get_entity(entite_lointaine).is_some(),
            "The GC should not have complied."
        );
    }

    #[test]
    fn test_gerer_lod_cache_les_planetes_si_zoom_superieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(4.0)), MainCamera));
        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Inherited,
            ))
            .id();

        app.update();

        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Hidden);
    }

    #[test]
    fn test_gerer_lod_affiche_les_planetes_si_zoom_inferieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(1.0)), MainCamera));
        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Hidden,
            ))
            .id();

        app.update();

        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Inherited);
    }

    #[test]
    fn test_gerer_lod_ne_fait_rien_si_camera_immobile() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        let entite_camera = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(2.0)), MainCamera))
            .id();
        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Hidden,
            ))
            .id();

        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );

        *app.world_mut()
            .get_mut::<Visibility>(entite_planete)
            .unwrap() = Visibility::Hidden;
        app.update();
        let visibilite_finale = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(
            *visibilite_finale,
            Visibility::Hidden,
            "The system ignored the Changed<Transform> filter."
        );

        app.world_mut()
            .get_mut::<Transform>(entite_camera)
            .unwrap()
            .scale
            .x = 2.1;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );
    }

    #[test]
    fn test_gerer_clic_ignore_si_pas_de_clic() {
        let mut app = preparer_app_clic();
        app.add_systems(Update, handle_star_click);

        let entite_etoile = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                StellarSystem {
                    nom: "Beta-Test".to_string(),
                    classe: SpectralClass::G,
                    masse_solaire: 1.0,
                    rayon_solaire: 1.0,
                    nb_planetes: 2,
                    age_milliards_annees: 4.0,
                },
                Star {
                    grille_x: 0,
                    grille_y: 0,
                },
            ))
            .id();

        app.update();
        assert!(app.world().get::<ExpandedSystem>(entite_etoile).is_none());
    }

    #[test]
    fn test_animer_orbites_calcule_position_et_angle_correctement() {
        let mut app = App::new();
        let mut time: Time = Time::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);

        let rayon = 100.0;
        let vitesse = std::f32::consts::PI;

        let entite = app
            .world_mut()
            .spawn((
                Transform::default(),
                Planet {
                    rayon_orbite: rayon,
                    angle_actuel: 0.0,
                    vitesse_orbite: vitesse,
                },
            ))
            .id();

        app.add_systems(Update, animate_orbits);
        app.update();

        let planete = app.world().get::<Planet>(entite).unwrap();
        let transform = app.world().get::<Transform>(entite).unwrap();

        let angle_attendu = vitesse * 0.5;
        assert_eq!(planete.angle_actuel, angle_attendu);

        let difference_x = (transform.translation.x - 0.0).abs();
        let difference_y = (transform.translation.y - 100.0).abs();
        assert!(
            difference_x < 0.0001,
            "The X position calculated using the cosine is incorrect."
        );
        assert!(
            difference_y < 0.0001,
            "The Y position calculated using the sine is incorrect."
        );
    }
}
