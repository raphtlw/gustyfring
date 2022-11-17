use natural::tokenize::tokenize;
use unicode_normalization::UnicodeNormalization;

pub fn normalize(text: &str) -> String {
    let toks = tokenize(&text);
    let mut text = toks.join(" ");
    text = text.nfc().collect::<String>();
    text.to_lowercase()
}
