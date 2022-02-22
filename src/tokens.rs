use jomini::TokenResolver;

pub struct TokenLookup;

impl TokenResolver for TokenLookup {
    fn resolve(&self, token: u16) -> Option<&str> {
        include!(concat!(env!("OUT_DIR"), "/gen_tokens.rs"))
    }
}
