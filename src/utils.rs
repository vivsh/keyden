
use rand::Rng;
use std::time::Duration;

const CHARSET_: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                         ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                         0123456789\
                         !@#$%^&*(-_=+)";

/// Generates a random secret string of the specified length.
/// The secret is generated using a cryptographically secure random number generator.
pub fn generate_secret(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET_.len());
            CHARSET_[idx] as char
        })
        .collect()
}


pub fn recommended_rotation_interval(ttl_secs: u64, count: usize) -> Duration {
    Duration::from_secs((ttl_secs * 2) / count as u64)
}


#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret(32);
        assert_eq!(secret.len(), 32);
        assert!(secret.chars().all(|c| CHARSET_.contains(&(c as u8))));
    }

    #[test]
    fn test_recommended_rotation_interval() {
        let interval = recommended_rotation_interval(60, 5);
        assert_eq!(interval.as_secs(), 24);
    }

}