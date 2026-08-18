// Deterministic spatial hash function
// Takes the (X, Y) coordinates and the global seed, and returns a "percentage" between 0.0 and 1.0.
pub fn calculate_spatial_hash(x: i32, y: i32, graine: u32) -> f32 {
    // Magic Constants: Large prime numbers.
    let prime_x: u32 = 374761393;
    let prime_y: u32 = 668265263;

    // Conversion to bits (to handle negative coordinates)
    let x_bits = x as u32;
    let y_bits = y as u32;

    // The Mixer
    let mut hash = graine;
    hash ^= x_bits.wrapping_mul(prime_x);

    // CORRECTION: Shift the bits circularly to break the symmetry.
    // before applying the XOR to Y. This destroys the sign-mirroring effect.
    hash = hash.rotate_left(17);

    hash ^= y_bits.wrapping_mul(prime_y);

    // The Avalanche: Bits are shifted and re-mixed.
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(1274126177);
    hash ^= hash >> 16;

    // Normalization: Returns a float between 0.0 and 1.0
    (hash as f32) / (std::u32::MAX as f32)
}

// Generates continuous value noise between 0.0 and 1.0.
// Allows for the deterministic creation of dense and empty zones.
pub fn get_macro_density(x: i32, y: i32, graine: u32) -> f32 {
    let echelle = 5; // Size of a "galactic cluster" in number of cells

    // Define the macro-grid coordinates
    let macro_x = x.div_euclid(echelle);
    let macro_y = y.div_euclid(echelle);

    // Define the fractional position within the macro-cell
    let frac_x = (x.rem_euclid(echelle) as f32) / (echelle as f32);
    let frac_y = (y.rem_euclid(echelle) as f32) / (echelle as f32);

    // Interpolation smoothing (Smoothstep) for an organic look
    let u = frac_x * frac_x * (3.0 - 2.0 * frac_x);
    let v = frac_y * frac_y * (3.0 - 2.0 * frac_y);

    // Hashing the 4 corners of the current macro-cell
    let h00 = calculate_spatial_hash(macro_x, macro_y, graine);
    let h10 = calculate_spatial_hash(macro_x + 1, macro_y, graine);
    let h01 = calculate_spatial_hash(macro_x, macro_y + 1, graine);
    let h11 = calculate_spatial_hash(macro_x + 1, macro_y + 1, graine);

    // Bilinear interpolation between the 4 corners
    let nx0 = h00 * (1.0 - u) + h10 * u;
    let nx1 = h01 * (1.0 - u) + h11 * u;

    nx0 * (1.0 - v) + nx1 * v
}

// --- START UNIT TESTS ---
mod tests {
    use super::*;

    // ---Start hashing test with limits---
    #[test]
    fn test_hachage_dans_les_limites() {
        // Verify that the generated hash is always between 0.0 and 1.0 inclusive
        let points_a_tester = [
            (0, 0, 12345),
            (10, 20, 9876),
            (-150, -300, 42),
            (std::i32::MAX, std::i32::MIN, std::u32::MAX),
        ];

        for (x, y, graine) in points_a_tester {
            let resultat = calculate_spatial_hash(x, y, graine);

            assert!(
                resultat >= 0.0,
                "The hashing value must be greater than or equal to 0.0."
            );
            assert!(
                resultat <= 1.0,
                "The hash must be less than or equal to 1.0."
            );
        }
    }
    // ---End hashing test with limits---

    // ---Start of deterministic hashing tests---
    #[test]
    fn test_hachage_deterministe() {
        // Ensure that calling the function multiple times with the exact same parameters yields the exact same float
        let x = 42;
        let y = -84;
        let graine = 9999;

        let premier_appel = calculate_spatial_hash(x, y, graine);
        let deuxieme_appel = calculate_spatial_hash(x, y, graine);

        // We can safely use assert_eq! here because we expect the exact same bit-for-bit float output
        assert_eq!(
            premier_appel, deuxieme_appel,
            "Hashing must be strictly deterministic for the same inputs."
        );
    }
    // ---Start of deterministic hashing tests---

    // ---Start the test of hashing differences with different seeds---
    #[test]
    fn test_graine_modifie_resultat() {
        // Check that altering the seed changes the output hash for the same coordinates
        let x = 100;
        let y = 100;

        let hachage_graine_1 = calculate_spatial_hash(x, y, 1000);
        let hachage_graine_2 = calculate_spatial_hash(x, y, 1001);

        assert_ne!(
            hachage_graine_1, hachage_graine_2,
            "Two different seeds must produce distinct hashes."
        );
    }
    // ---End the test of hashing differences with different seeds---

    // ---Start of the test comparing hashing differences with their negative values---
    #[test]
    fn test_coordonnees_differentes_et_negatives() {
        // Validate that negative coordinates don't crash and that symmetric coordinates do not produce collisions
        let graine = 12345;

        let hachage_positif = calculate_spatial_hash(10, 10, graine);
        let hachage_negatif = calculate_spatial_hash(-10, -10, graine);
        let hachage_mixte = calculate_spatial_hash(-10, 10, graine);

        assert_ne!(
            hachage_positif, hachage_negatif,
            "The symmetrical coordinates (10, 10) and (-10, -10) must not collide."
        );
        assert_ne!(
            hachage_positif, hachage_mixte,
            "Each quadrant of the spatial grid must generate unique values."
        );
    }
    // ---End of the test comparing hashing differences with their negative values---
}
// --- END UNIT TESTS ---
