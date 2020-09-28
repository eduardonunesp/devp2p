use crate::types::*;
use async_trait::async_trait;
use std::{collections::HashMap, net::SocketAddr};

#[cfg(feature = "discv5")]
mod discv5;

#[cfg(feature = "dnsdisc")]
mod dnsdisc;

#[cfg(feature = "dnsdisc")]
pub use self::dnsdisc::DnsDiscovery;

#[async_trait]
pub trait Discovery: Send + Sync + 'static {
    async fn get_new_peer(&mut self) -> StdResult<(SocketAddr, PeerId)>;
}

#[async_trait]
impl<S: Send + Sync + 'static> Discovery for HashMap<SocketAddr, PeerId, S> {
    async fn get_new_peer(&mut self) -> StdResult<(SocketAddr, PeerId)> {
        Ok(self
            .iter()
            .next()
            .map(|(&k, &v)| (k, v))
            .ok_or_else(|| "No peers in set")?)
    }
}
