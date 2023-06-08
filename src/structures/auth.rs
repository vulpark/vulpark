// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RequestTokenError, Scope,
    StandardErrorResponse, StandardTokenResponse, TokenUrl,
};
use std::{borrow::Cow, fmt::Display};

const REDIRECT_URI: &str = "http://127.0.0.1:8000/auth";

#[derive(Eq, Hash, PartialEq)]
pub enum Service {
    Github,
}

impl Service {
    fn secret(&self) -> Option<ClientSecret> {
        Some(ClientSecret::new(
            std::env::var(format!("{self}_SECRET").to_uppercase()).ok()?,
        ))
    }

    fn client_id(&self) -> ClientId {
        match self {
            Self::Github => ClientId::new("01a54a05ac326eca7c4f".into()),
        }
    }

    fn auth_url(&self) -> AuthUrl {
        match self {
            Self::Github => {
                AuthUrl::new("https://github.com/login/oauth/authorize".into()).unwrap()
            }
        }
    }

    fn token_url(&self) -> Option<TokenUrl> {
        match self {
            Self::Github => {
                TokenUrl::new("https://github.com/login/oauth/access_token".into()).ok()
            }
        }
    }

    fn scope(&self) -> Scope {
        match self {
            Self::Github => Scope::new("user".to_string()),
        }
    }

    pub fn client(&self) -> BasicClient {
        BasicClient::new(
            self.client_id(),
            self.secret(),
            self.auth_url(),
            self.token_url(),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.into()).unwrap())
    }

    pub fn get_url(&self, client: &BasicClient, path: &str) -> (Url, CsrfToken, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(self.scope())
            .set_pkce_challenge(pkce_challenge)
            .set_redirect_uri(Cow::Owned(
                RedirectUrl::new(format!("{REDIRECT_URI}/{path}")).unwrap(),
            ))
            .url();
        (auth_url, csrf_token, pkce_verifier)
    }

    pub async fn get_token(
        &self,
        client: BasicClient,
        code: AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    > {
        client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Github => f.write_str("github"),
        }
    }
}
