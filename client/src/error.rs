#[cfg(feature = "auth")]
use authc::AuthClientError;
use common_net::msg::server::BanInfo;
pub use network::{InitProtocolError, NetworkConnectError, NetworkError};
use network::{ParticipantError, StreamError};
#[cfg(feature = "quic")]
use rustls::Error as RustlsError;
use specs::error::Error as SpecsError;

#[derive(Debug)]
pub enum Error {
    Kicked(String),
    NetworkErr(NetworkError),
    ParticipantErr(ParticipantError),
    StreamErr(StreamError),
    ServerTimeout,
    ServerShutdown,
    TooManyPlayers,
    NotOnWhitelist,
    AuthErr(String),
    #[cfg(feature = "auth")]
    AuthClientError(AuthClientError),
    AuthServerUrlInvalid(String),
    AuthServerNotTrusted,
    HostnameLookupFailed(std::io::Error),
    Banned(BanInfo),
    /// Persisted character data is invalid or missing
    InvalidCharacter,
    //TODO: InvalidAlias,
    Other(String),
    SpecsErr(SpecsError),
    #[cfg(feature = "quic")]
    RustlsErr(RustlsError),
}

impl From<SpecsError> for Error {
    fn from(err: SpecsError) -> Self { Self::SpecsErr(err) }
}

#[cfg(feature = "quic")]
impl From<RustlsError> for Error {
    fn from(err: RustlsError) -> Self { Self::RustlsErr(err) }
}

impl From<NetworkError> for Error {
    fn from(err: NetworkError) -> Self { Self::NetworkErr(err) }
}

impl From<ParticipantError> for Error {
    fn from(err: ParticipantError) -> Self { Self::ParticipantErr(err) }
}

impl From<StreamError> for Error {
    fn from(err: StreamError) -> Self { Self::StreamErr(err) }
}

#[cfg(feature = "auth")]
impl From<AuthClientError> for Error {
    fn from(err: AuthClientError) -> Self { Self::AuthClientError(err) }
}
