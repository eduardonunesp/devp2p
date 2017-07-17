extern crate dpt;
extern crate rand;
extern crate secp256k1;
extern crate etcommon_crypto;
extern crate etcommon_bigint;
extern crate etcommon_rlp as rlp;

#[macro_use]
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;

use etcommon_bigint::H512;
use etcommon_crypto::SECP256K1;
use tokio_core::reactor::Core;
use secp256k1::key::{PublicKey, SecretKey};
use rand::os::OsRng;
use futures::future;
use futures::{Stream, Sink, Future};
use std::str::FromStr;
use dpt::{DPTMessage, DPTNode, DPTStream};

const REMOTE_ID: &str = "d02c7c6d49c668f750cf6c007b4a9cc96be08c335d3e027afa110f86c48192725aa2e8a60c581044c7c489fee45a3d0acbbfe4d10eb1717bc6b3374364bf895d";

fn main() {
    let addr = "127.0.0.1:50505".parse().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let mut client = DPTStream::new(
        &addr, &handle,
        SecretKey::new(&SECP256K1, &mut OsRng::new().unwrap()),
        vec![DPTNode {
            address: "127.0.0.1".parse().unwrap(),
            tcp_port: 30303,
            udp_port: 30303,
            id: H512::from_str(REMOTE_ID).unwrap(),
        }], 0).unwrap();

    core.run(client.send(DPTMessage::RequestNewPeer)
             .and_then(|client| {
                 client.into_future().map_err(|(e, _)| e)
             })
             .and_then(|(val, client)| {
                 println!("new peer: {:?}", val);
                 future::ok(client)
             })).unwrap();
}
