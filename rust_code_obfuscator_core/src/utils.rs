use rand::{rng, Rng};

pub fn generate_obf_suffix() -> u32 {
    let mut rng = rng();
    rng.random_range(1000..=9999)
}
