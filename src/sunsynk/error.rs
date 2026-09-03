use std::{error::Error, fmt};

#[derive(Debug)]
pub(crate) struct AuthenticationExpired;

impl fmt::Display for AuthenticationExpired {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SunSynk authentication expired")
    }
}

impl Error for AuthenticationExpired {}

#[derive(Debug)]
pub(crate) struct RefreshTokenRejected;

impl fmt::Display for RefreshTokenRejected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SunSynk refresh token rejected")
    }
}

impl Error for RefreshTokenRejected {}
