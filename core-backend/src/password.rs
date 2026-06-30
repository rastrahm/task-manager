//! Hash y verificación de contraseñas con Argon2id.
//!
//! Las contraseñas nunca se almacenan en texto plano; solo el string PHC
//! devuelto por [`hash_password`] se guarda en `users.password_hash`.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

/// Genera un hash Argon2id con salt aleatorio.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Comprueba una contraseña contra un hash PHC almacenado.
///
/// Devuelve `false` si el hash es inválido o la contraseña no coincide.
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("secret123").unwrap();
        assert!(verify_password("secret123", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn verify_rejects_invalid_hash_string() {
        assert!(!verify_password("x", "not-a-valid-phc-hash"));
    }

    #[test]
    fn same_password_produces_different_hashes() {
        let a = hash_password("same").unwrap();
        let b = hash_password("same").unwrap();
        assert_ne!(a, b);
        assert!(verify_password("same", &a));
        assert!(verify_password("same", &b));
    }
}
