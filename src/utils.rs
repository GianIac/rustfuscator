use rand::{thread_rng, Rng};

/// Genera un suffisso numerico randomico tra 1000 e 9999.
/// Usato per costruire nomi offuscati validi.
pub fn generate_obf_suffix() -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(1000..=9999)
}