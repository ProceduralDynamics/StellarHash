use bevy::prelude::*;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

// Import the necessary elements from your game
use StellarHash::camera::MainCamera;
use StellarHash::univers::{LoadedSectors, Star, spatial_garbage_collector};

/// Utility function that recreates a test environment
fn prepare_world_overload() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<LoadedSectors>();

    app.world_mut()
        .spawn((Transform::from_xyz(5000.0, 0.0, 0.0), MainCamera));

    for x in -50..50 {
        for y in -50..50 {
            let entite = app
                .world_mut()
                .spawn(Star {
                    grille_x: x,
                    grille_y: y,
                })
                .id();

            app.world_mut()
                .resource_mut::<LoadedSectors>()
                .0
                .insert((x, y), Some(entite));
        }
    }

    app.add_systems(Update, spatial_garbage_collector);

    app
}

fn bench_garbage_collector(c: &mut Criterion) {
    c.bench_function("garbage_collector_10000_stars", |b| {
        b.iter_batched_ref(
            || prepare_world_overload(),
            |app| {
                // Measured step: Launch 1 frame.
                // The engine will detect the 10,000 distant entities, destroy them, and filter the HashSet.
                app.update();
            },
            // Tells Criterion that the preparation requires a lot of RAM.
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, bench_garbage_collector);
criterion_main!(benches);
