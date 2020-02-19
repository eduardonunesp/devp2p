use bigint::{H256, U256};
use block::Block;
use devp2p::{
    dpt::DPTNode, rlpx::RLPxNode, DevP2PConfig, ETHMessage, ETHReceiveMessage, ETHSendMessage,
    ETHStream,
};
use futures::{future, Future, Sink, Stream};
use hexutil::*;
use rand::os::OsRng;
use secp256k1::{key::SecretKey, SECP256K1};
use sha3::{Digest, Keccak256};
use std::{str::FromStr, time::Duration};
use tokio_core::reactor::{Core, Timeout};
use url::Url;

const GENESIS_HASH: &str = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
const GENESIS_DIFFICULTY: usize = 17_179_869_184;

const ETC_DAO_BLOCK: &str = "f903cff9020fa0a218e2c611f21232d857e3c8cecdcdf1f65f25a4477f98f6f47e4063807f2308a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479461c808d82a3ac53231750dadc13c777b59310bd9a0614d7d358b03cbdaf0343529673be20ad45809d02487f023e047efdce9da8affa0d33068a7f21bff5018a00ca08a3566a06be4196dfe9e39f96e431565a619d455a07bda9aa65977800376129148cbfe89d35a016dd51c95d6e6dc1e76307d315468b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008638c3bf2616aa831d4c008347e7c08301482084578f7aa78fe4b883e5bda9e7a59ee4bb99e9b1bca0c52daa7054babe515b17ee98540c0889cf5e1595c5dd77496997ca84a68c8da18805276a600980199df901b9f86c018504a817c8008252089453d284357ec70ce289d6d64134dfac8e511c8a3d888b6cfa3afc058000801ba08d94a55c7ac7adbfa2285ef7f4b0c955ae1a02647452cd4ead03ee6f449675c6a067149821b74208176d78fc4dffbe37c8b64eecfd47532406b9727c4ae8eb7c9af86d018504a817c8008252089453d284357ec70ce289d6d64134dfac8e511c8a3d890116db7272d6d94000801ca06d31e3d59bfea97a34103d8ce767a8fe7a79b8e2f30af1e918df53f9e78e69aba0098e5b80e1cc436421aa54eb17e96b08fe80d28a2fbd46451b56f2bca7a321e7f86c018504a817c8008252089453d284357ec70ce289d6d64134dfac8e511c8a3d8814da2c24e0d37014801ba0fdbbc462a8a60ac3d8b13ee236b45af9b7991cf4f0f556d3af46aa5aeca242aba05de5dc03fdcb6cf6d14609dbe6f5ba4300b8ff917c7d190325d9ea2144a7a2fbf86c018504a817c8008252089453d284357ec70ce289d6d64134dfac8e511c8a3d880e301365046d5000801ba0bafb9f71cef873b9e0395b9ed89aac4f2a752e2a4b88ba3c9b6c1fea254eae73a01cef688f6718932f7705d9c1f0dd5a8aad9ddb196b826775f6e5703fdb997706c0";

const BOOTSTRAP_NODES: &[&str] = &[
    "enode://5d6d7cd20d6da4bb83a1d28cadb5d409b64edf314c0335df658c1a54e32c7c4a7ab7823d57c39b6a757556e68ff1df17c748b698544a55cb488b52479a92b60f@104.42.217.25:30303",
    "enode://68f46370191198b71a1595dd453c489bbfe28036a9951fc0397fabd1b77462930b3c5a5359b20e99677855939be47b39fc8edcf1e9ff2522a922b86d233bf2df@144.217.153.76:30303",
    "enode://ffed6382e05ee42854d862f08e4e39b8452c50a5a5d399072c40f9a0b2d4ad34b0eb5312455ad8bcf0dcb4ce969dc89a9a9fd00183eaf8abf46bbcc59dc6e9d5@51.195.3.238:30303",
];

pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.input(data);
    let out = hasher.result();
    H256::from(out.as_ref())
}

fn main() {
    let _ = env_logger::init();

    let addr = "0.0.0.0:30303".parse().unwrap();
    let public_addr = "127.0.0.1".parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = ETHStream::new(
        addr,
        public_addr,
        &handle,
        SecretKey::new(&SECP256K1, &mut OsRng::new().unwrap()),
        "etclient Rust/0.1.0".to_string(),
        1,
        H256::from_str(GENESIS_HASH).unwrap(),
        H256::from_str(GENESIS_HASH).unwrap(),
        U256::from(GENESIS_DIFFICULTY),
        BOOTSTRAP_NODES
            .iter()
            .map(|v| DPTNode::from_url(&Url::parse(v).unwrap()).unwrap())
            .collect(),
        DevP2PConfig {
            ping_interval: Duration::new(600, 0),
            ping_timeout_interval: Duration::new(700, 0),
            optimal_peers_len: 25,
            optimal_peers_interval: Duration::new(5, 0),
            reconnect_dividend: 5,
            listen: false,
        },
    )
    .unwrap();

    let mut best_number;
    let mut best_hash: H256 = H256::from_str(GENESIS_HASH).unwrap();
    let got_bodies_for_current = true;

    let dur = Duration::new(10, 0);
    let req_max_headers = 2048;

    let (mut client_sender, client_receiver) = client.split();
    let mut client_future = client_receiver.into_future();
    let mut timeout = Box::new(Timeout::new(dur, &handle).unwrap())
        as Box<dyn Future<Item = _, Error = _> + Send + 'static>;

    let mut active_peers = 0;

    while let Ok(ret) = core.run(client_future.select2(timeout)) {
        let (val, new_client_receiver) = match ret {
            future::Either::A(((val, new_client), t)) => {
                timeout = t;
                (val, new_client)
            }
            future::Either::B((_, fu)) => {
                client_future = fu;

                println!("request downloading header ...");
                client_sender = core
                    .run(client_sender.send(ETHSendMessage {
                        node: RLPxNode::Any,
                        data: ETHMessage::GetBlockHeadersByHash {
                            hash: best_hash,
                            max_headers: req_max_headers,
                            skip: 0,
                            reverse: false,
                        },
                    }))
                    .unwrap();

                timeout = Box::new(Timeout::new(dur, &handle).unwrap());

                continue;
            }
        };

        if val.is_none() {
            break;
        }
        let val = val.unwrap();

        match val {
            ETHReceiveMessage::Normal { node, data, .. } => match data {
                ETHMessage::Status { .. } => (),

                ETHMessage::Transactions(_) => {
                    println!("received new transactions");
                }

                ETHMessage::GetBlockHeadersByNumber { number, .. } => {
                    if number == U256::from(1_920_000) {
                        println!("requested DAO header");
                        let block_raw = read_hex(ETC_DAO_BLOCK).unwrap();
                        let block: Block = rlp::decode(&block_raw);
                        client_sender = core
                            .run(client_sender.send(ETHSendMessage {
                                node: RLPxNode::Peer(node),
                                data: ETHMessage::BlockHeaders(vec![block.header]),
                            }))
                            .unwrap();
                    } else {
                        println!("requested header {}", number);
                        client_sender = core
                            .run(client_sender.send(ETHSendMessage {
                                node: RLPxNode::Peer(node),
                                data: ETHMessage::BlockHeaders(Vec::new()),
                            }))
                            .unwrap();
                    }
                }

                ETHMessage::GetBlockHeadersByHash { hash, .. } => {
                    println!("requested header {}", hash);
                    client_sender = core
                        .run(client_sender.send(ETHSendMessage {
                            node: RLPxNode::Peer(node),
                            data: ETHMessage::BlockHeaders(Vec::new()),
                        }))
                        .unwrap();
                }

                ETHMessage::GetBlockBodies(hash) => {
                    println!("requested body {:?}", hash);
                    client_sender = core
                        .run(client_sender.send(ETHSendMessage {
                            node: RLPxNode::Peer(node),
                            data: ETHMessage::BlockBodies(Vec::new()),
                        }))
                        .unwrap();
                }

                ETHMessage::BlockHeaders(headers) => {
                    println!("received block headers of len {}", headers.len());
                    if got_bodies_for_current {
                        for header in headers {
                            if header.parent_hash == best_hash {
                                best_hash = keccak256(&rlp::encode(&header).to_vec());
                                best_number = header.number;
                                println!("updated best number: {}", best_number);
                                println!("updated best hash: 0x{:x}", best_hash);
                            }
                        }
                    }
                    client_sender = core
                        .run(client_sender.send(ETHSendMessage {
                            node: RLPxNode::Any,
                            data: ETHMessage::GetBlockHeadersByHash {
                                hash: best_hash,
                                max_headers: req_max_headers,
                                skip: 0,
                                reverse: false,
                            },
                        }))
                        .unwrap();
                    timeout = Box::new(Timeout::new(dur, &handle).unwrap());
                }

                ETHMessage::BlockBodies(bodies) => {
                    println!("received block bodies of len {}", bodies.len());
                }

                msg => {
                    println!("received {:?}", msg);
                }
            },
            ETHReceiveMessage::Connected { .. } => {
                active_peers += 1;
            }
            ETHReceiveMessage::Disconnected { .. } => {
                active_peers -= 1;
            }
        }

        println!("current active peers: {}", active_peers);

        client_future = new_client_receiver.into_future();
    }
}
