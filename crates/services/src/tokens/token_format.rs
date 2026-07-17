use base64::{engine::general_purpose, Engine};

/// Prefix for BodhiApp API tokens. `sk-` is the industry "secret key" convention
/// (OpenAI/Anthropic); `bodhiapp` is the issuer segment. The distinctive, keyword-
/// anchored prefix is what makes the format reliable for GitHub secret scanning.
pub const BODHIAPP_TOKEN_PREFIX: &str = "sk-bodhiapp_";

/// Length, in characters, of the base64url-encoded CRC32 checksum appended to the
/// random segment. CRC32 = 4 bytes → 6 base64url-no-pad chars.
pub const TOKEN_CHECKSUM_LEN: usize = 6;

/// Compute the format-integrity checksum for a token's random segment: CRC32(IEEE)
/// over the random string, big-endian, base64url-no-pad encoded (6 chars).
///
/// This is NOT a security control — it enables offline rejection of malformed tokens
/// (and high-confidence secret-scanner detection) before any DB lookup. Security is
/// the SHA-256 hash + DB comparison.
pub fn token_checksum(random: &str) -> String {
  let crc = crc32fast::hash(random.as_bytes());
  general_purpose::URL_SAFE_NO_PAD.encode(crc.to_be_bytes())
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  fn test_token_checksum_is_deterministic_and_fixed_len() {
    let random = "abcDEF0123456789_-abcDEF0123456789_-abcDE12";
    let sum = token_checksum(random);
    assert_eq!(TOKEN_CHECKSUM_LEN, sum.len());
    assert_eq!(sum, token_checksum(random));
  }

  #[rstest]
  fn test_token_checksum_changes_on_tamper() {
    let random = "abcDEF0123456789_-abcDEF0123456789_-abcDE12";
    let tampered = "Xbc DEF0123456789_-abcDEF0123456789_-abcDE12".replace(' ', "");
    assert_ne!(token_checksum(random), token_checksum(&tampered));
  }
}
