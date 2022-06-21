use jomini::binary::TokenResolver;

pub struct EnvTokens;

impl TokenResolver for EnvTokens {
    fn resolve(&self, token: u16) -> Option<&str> {
        include!(concat!(env!("OUT_DIR"), "/gen_tokens.rs"))
    }
}
