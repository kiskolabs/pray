pub(crate) struct PublishIdentity<'a> {
    pub signer: &'a str,
    pub signer_fingerprint: Option<&'a str>,
    pub published_at: u64,
    pub signing_key: Option<&'a ed25519_dalek::SigningKey>,
    pub resign: bool,
}

pub(crate) struct PublishDestLabel<'a> {
    pub dest_name: &'a str,
    pub target: &'a str,
}
