mod queries_activity;
mod queries_dashboard;
mod queries_file_settings;
mod queries_settings;
pub(crate) mod queries_tuning;
mod utils;

pub(crate) use queries_activity::{ActivityKey, QUERIES_ACTIVITY};
pub(crate) use queries_dashboard::{DashboardKey, QUERIES_DASHBOARD};
pub(crate) use queries_file_settings::{FileSettingsKey, QUERIES_FILE_SETTINGS};
pub(crate) use queries_settings::{QUERIES_SETTINGS, SettingsKey};
pub(crate) use queries_tuning::{
    ColumnConstraint, DatabaseColumnDefinition, QUERIES_TUNING, TuningKey,
};
pub(crate) use utils::get_str;

use std::sync::Arc;

use postgres::{Client, Config, Error, NoTls};
use rustls::ClientConfig;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error as TlsError, SignatureScheme};
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::args::ArgsStruct;

#[derive(Debug)]
struct AcceptAllVerifier;

impl ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dre: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dre: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

pub(crate) fn db_connect(args: &ArgsStruct) -> Result<Client, Error> {
    let app_name = format!("Ellie {}", crate::VERSION);

    let mut config = match &args.url {
        Some(url) => url.parse::<Config>().unwrap_or_default(),
        None => {
            let mut c = Config::new();
            c.host(&args.host)
                .port(args.port as u16)
                .user(&args.user)
                .password(&args.password)
                .dbname(&args.database);
            c
        }
    };
    config.application_name(&app_name);

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    if let Ok(tls_builder) =
        ClientConfig::builder_with_provider(provider).with_safe_default_protocol_versions()
    {
        let tls_config = tls_builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAllVerifier))
            .with_no_client_auth();

        if let Ok(client) = config.connect(MakeRustlsConnect::new(tls_config)) {
            return Ok(client);
        }
    }

    config.connect(NoTls)
}

pub(crate) fn db_query(client: &mut Client, query: &str) -> Result<Vec<postgres::Row>, Error> {
    client.query(query, &[])
}
