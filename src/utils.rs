use rand::{thread_rng, Rng};

pub fn generate_obf_suffix() -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(1000..=9999)
}