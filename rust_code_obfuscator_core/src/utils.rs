use rand::{rng, Rng};

const MIN_SUFF_VALUE: u32 = 1000;
const MAX_SUFF_VALUE: u32 = 9999;

pub fn generate_obf_suffix() -> u32 {
    let mut rng = rng();
    let num: u32 = rng.random_range(MIN_SUFF_VALUE..=MAX_SUFF_VALUE);
    num
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_suffix_len() -> usize {
        let len_1 = MIN_SUFF_VALUE.to_string().len();
        let len_2 = MAX_SUFF_VALUE.to_string().len();
        if len_1 != len_2 {
            panic!(
                "
            The lengths of the boundary values are no longer the same!
            Change one of the values or change the test."
            );
        }
        len_1
    }

    #[test]
    fn check_generate_obf_suffix() {
        let suf = generate_obf_suffix();

        let expected_suff_len = get_suffix_len();

        assert_eq!(
            suf.to_string().len(),
            expected_suff_len,
            "Generated suffix must be {} digits long, received {}",
            expected_suff_len.to_string(),
            suf.to_string()
        );
        assert!(
            (MIN_SUFF_VALUE..=MAX_SUFF_VALUE).contains(&suf),
            "Numeric value must be between {} and {}, received {}",
            MIN_SUFF_VALUE,
            MAX_SUFF_VALUE,
            suf
        );
    }
}
