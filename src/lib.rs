//! `TDN` - Trusted Distributed Network.
//!
//! Blockchain infrastructure framework for security and trusted distributed interactive.
//!
//! TDN is underlying network (including p2p, rpc, and other special transports)
//! and application framework built on Groups and Layers, we built this framework
//! because we feel that the blockchain is very limited. If you want a more open
//! and free distributed application development technology, and Pluggable,
//! lightweight application framework, TDN can satisfy you.

#![recursion_limit = "1024"]

// check features conflict
#[cfg(any(
    all(
        feature = "std",
        any(feature = "single", feature = "multiple", feature = "full")
    ),
    all(
        feature = "single",
        any(feature = "std", feature = "multiple", feature = "full")
    ),
    all(
        feature = "multiple",
        any(feature = "std", feature = "single", feature = "full")
    ),
    all(
        feature = "full",
        any(feature = "std", feature = "single", feature = "multiple")
    ),
))]
panic!("feature conflict, only one feature at one time.");

#[macro_use]
extern crate log;

mod config;
mod group;
mod rpc;

#[cfg(any(feature = "std", feature = "full"))]
mod layer;

// public mod
pub mod error;

// re-export smol
pub use smol;
// re-export tdn_types
pub use tdn_types as types;

// public struct
pub mod prelude {
    pub use super::config::Config;

    pub use tdn_types::group::{GroupId, GROUP_LENGTH};
    pub use tdn_types::message::{NetworkType, RecvType, SendType, StateRequest, StateResponse};

    pub use tdn_types::message::{ReceiveMessage, SendMessage};
    pub use tdn_types::primitive::{Broadcast, HandleResult, PeerAddr};

    use chamomile::prelude::{
        start as chamomile_start, ReceiveMessage as ChamomileReceiveMessage,
        SendMessage as ChamomileSendMessage,
    };
    use smol::{
        channel::{self, Receiver, Sender},
        future,
        io::Result,
        lock::RwLock,
    };
    use std::sync::Arc;
    use tdn_types::message::RpcSendMessage;

    use super::group::*;
    use super::rpc::start as rpc_start;

    #[cfg(any(feature = "std", feature = "full"))]
    use super::layer::*;

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_send_channel() -> (Sender<SendMessage>, Receiver<SendMessage>) {
        channel::unbounded()
    }

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_receive_channel() -> (Sender<ReceiveMessage>, Receiver<ReceiveMessage>) {
        channel::unbounded()
    }

    /// start a service, use config.toml file.
    /// send a Sender<Message>, and return the peer_id, and service Sender<Message>.
    pub async fn start() -> Result<(PeerAddr, Sender<SendMessage>, Receiver<ReceiveMessage>)> {
        let (send_send, send_recv) = new_send_channel();
        let (recv_send, recv_recv) = new_receive_channel();

        let config = Config::load().await;

        let peer_addr = start_main(recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    /// start a service with config.
    pub async fn start_with_config(
        config: Config,
    ) -> Result<(PeerAddr, Sender<SendMessage>, Receiver<ReceiveMessage>)> {
        let (send_send, send_recv) = new_send_channel();
        let (recv_send, recv_recv) = new_receive_channel();

        let peer_addr = start_main(recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    async fn start_main(
        out_send: Sender<ReceiveMessage>,
        self_recv: Receiver<SendMessage>,
        config: Config,
    ) -> Result<PeerAddr> {
        let (group_ids, p2p_config, rpc_config) = config.split();

        // start chamomile network & inner rpc.
        let ((peer_id, p2p_send, p2p_recv), rpc_sender) = future::try_zip(
            chamomile_start(p2p_config),
            rpc_start(rpc_config, out_send.clone()),
        )
        .await?;

        debug!("chamomile & jsonrpc service started");
        let my_groups = Arc::new(RwLock::new(group_ids));
        let my_groups_1 = my_groups.clone();

        // handle outside msg.
        smol::spawn(async move {
            while let Ok(message) = self_recv.recv().await {
                match message {
                    #[cfg(any(feature = "single", feature = "std"))]
                    SendMessage::Group(msg) => {
                        let groups_lock = my_groups_1.read().await;
                        if groups_lock.len() == 0 {
                            drop(groups_lock);
                            continue;
                        }
                        let default_group_id = groups_lock[0].clone();
                        drop(groups_lock);

                        group_handle_send(default_group_id, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    #[cfg(any(feature = "multiple", feature = "full"))]
                    SendMessage::Group(group_id, msg) => {
                        if !my_groups_1.read().await.contains(&group_id) {
                            continue;
                        }

                        group_handle_send(&group_id, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    SendMessage::Rpc(uid, param, is_ws) => {
                        rpc_sender
                            .send(RpcSendMessage(uid, param, is_ws))
                            .await
                            .map_err(|e| error!("Rpc channel: {:?}", e))
                            .expect("Rpc channel closed");
                    }
                    #[cfg(feature = "std")]
                    SendMessage::Layer(tgid, msg) => {
                        let groups_lock = my_groups_1.read().await;
                        if groups_lock.len() == 0 {
                            drop(groups_lock);
                            continue;
                        }
                        let default_group_id = groups_lock[0].clone();
                        drop(groups_lock);

                        layer_handle_send(default_group_id, tgid, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    #[cfg(feature = "full")]
                    SendMessage::Layer(fgid, tgid, msg) => {
                        if !my_groups_1.read().await.contains(&tgid) {
                            continue;
                        }

                        layer_handle_send(&fgid, tgid, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    #[cfg(any(feature = "multiple", feature = "full"))]
                    SendMessage::AddGroup(gid) => {
                        let group_lock = my_groups_1.write().await;
                        if !group_lock.contains(&gid) {
                            group_lock.push(gid);
                        }
                        drop(group_lock);
                    }
                    #[cfg(any(feature = "multiple", feature = "full"))]
                    SendMessage::DelGroup(gid) => {
                        let group_lock = my_groups_1.write().await;
                        let mut need_remove: Vec<usize> = vec![];
                        for (k, i) in group_lock.iter().enumerate() {
                            if i == &gid {
                                need_remove.push(k);
                            }
                        }
                        for i in need_remove.iter().rev() {
                            group_lock.remove(*i);
                        }
                        drop(group_lock);
                    }
                    SendMessage::Network(nmsg) => match nmsg {
                        NetworkType::Broadcast(broadcast, data) => {
                            // broadcast use default_group_id.
                            let mut bytes = vec![];
                            let groups_lock = my_groups_1.read().await;
                            if groups_lock.len() == 0 {
                                drop(groups_lock);
                                continue;
                            }
                            bytes.extend(&groups_lock[0].0);
                            drop(groups_lock);
                            bytes.extend(data);
                            p2p_send
                                .send(ChamomileSendMessage::Broadcast(broadcast, bytes))
                                .await
                                .map_err(|e| error!("Chamomile channel: {:?}", e))
                                .expect("Chamomile channel closed");
                        }
                        NetworkType::Connect(addr) => {
                            p2p_send
                                .send(ChamomileSendMessage::Connect(addr))
                                .await
                                .map_err(|e| error!("Chamomile channel: {:?}", e))
                                .expect("Chamomile channel closed");
                        }
                        NetworkType::DisConnect(addr) => {
                            p2p_send
                                .send(ChamomileSendMessage::DisConnect(addr))
                                .await
                                .map_err(|e| error!("Chamomile channel: {:?}", e))
                                .expect("Chamomile channel closed");
                        }
                        NetworkType::NetworkState(req, sender) => {
                            p2p_send
                                .send(ChamomileSendMessage::NetworkState(req, sender))
                                .await
                                .map_err(|e| error!("Chamomile channel: {:?}", e))
                                .expect("Chamomile channel closed");
                        }
                    },
                }
            }
        })
        .detach();

        // handle chamomile send msg.
        smol::spawn(async move {
            // if group's inner message, from_group in our groups.
            // if layer's message,       from_group not in our groups.

            while let Ok(message) = p2p_recv.recv().await {
                match message {
                    ChamomileReceiveMessage::StableConnect(peer_addr, mut data) => {
                        let mut gid_bytes = [0u8; GROUP_LENGTH];
                        if data.len() < GROUP_LENGTH {
                            continue;
                        }
                        gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
                        let gid = GroupId(gid_bytes);

                        let group_lock = my_groups.read().await;
                        if group_lock.len() == 0 {
                            continue;
                        }

                        if group_lock.contains(&gid) {
                            drop(group_lock);
                            let _ =
                                group_handle_recv_stable_connect(&gid, &out_send, peer_addr, data)
                                    .await;
                        } else {
                            drop(group_lock);
                            // layer handle it.
                            let _ =
                                layer_handle_recv_connect(gid, &out_send, peer_addr, data).await;
                        }
                    }
                    ChamomileReceiveMessage::StableResult(peer_addr, is_ok, mut data) => {
                        let mut gid_bytes = [0u8; GROUP_LENGTH];
                        if data.len() < GROUP_LENGTH {
                            continue;
                        }
                        gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
                        let gid = GroupId(gid_bytes);

                        let group_lock = my_groups.read().await;
                        if group_lock.len() == 0 {
                            continue;
                        }

                        if group_lock.contains(&gid) {
                            drop(group_lock);
                            let _ = group_handle_recv_stable_result(
                                &gid, &out_send, peer_addr, is_ok, data,
                            )
                            .await;
                        } else {
                            drop(group_lock);
                            // layer handle it.
                            let _ =
                                layer_handle_recv_result(gid, &out_send, peer_addr, is_ok, data)
                                    .await;
                        }
                    }
                    ChamomileReceiveMessage::StableLeave(peer_addr) => {
                        let group_lock = my_groups.read().await;
                        for gid in group_lock.iter() {
                            let _ =
                                group_handle_recv_stable_leave(&gid, &out_send, peer_addr).await;
                            let _ = layer_handle_recv_leave(*gid, &out_send, peer_addr).await;
                        }
                        drop(group_lock);
                    }
                    ChamomileReceiveMessage::Data(peer_addr, mut data) => {
                        let mut gid_bytes = [0u8; GROUP_LENGTH];
                        if data.len() < GROUP_LENGTH {
                            continue;
                        }
                        gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
                        let gid = GroupId(gid_bytes);

                        let group_lock = my_groups.read().await;
                        if group_lock.len() == 0 {
                            continue;
                        }

                        if group_lock.contains(&gid) {
                            drop(group_lock);
                            let _ = group_handle_recv_data(&gid, &out_send, peer_addr, data).await;
                        } else {
                            drop(group_lock);
                            // layer handle it.
                            let _ = layer_handle_recv_data(gid, &out_send, peer_addr, data).await;
                        }
                    }
                    ChamomileReceiveMessage::Stream(id, stream, mut data) => {
                        let mut gid_bytes = [0u8; GROUP_LENGTH];
                        if data.len() < GROUP_LENGTH {
                            continue;
                        }
                        gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
                        let gid = GroupId(gid_bytes);

                        let group_lock = my_groups.read().await;
                        if group_lock.len() == 0 {
                            continue;
                        }

                        if group_lock.contains(&gid) {
                            drop(group_lock);
                            let _ =
                                group_handle_recv_stream(&gid, &out_send, id, stream, data).await;
                        } else {
                            drop(group_lock);
                            // layer handle it.
                            let _ =
                                layer_handle_recv_stream(gid, &out_send, id, stream, data).await;
                        }
                    }
                    ChamomileReceiveMessage::Delivery(t, tid, is_ok, mut data) => {
                        let mut gid_bytes = [0u8; GROUP_LENGTH];
                        if data.len() < GROUP_LENGTH {
                            continue;
                        }
                        gid_bytes.copy_from_slice(data.drain(..GROUP_LENGTH).as_slice());
                        let gid = GroupId(gid_bytes);

                        let group_lock = my_groups.read().await;
                        if group_lock.len() == 0 {
                            continue;
                        }

                        if group_lock.contains(&gid) {
                            drop(group_lock);
                            let _ =
                                group_handle_recv_delivery(&gid, &out_send, t.into(), tid, is_ok)
                                    .await;
                        } else {
                            drop(group_lock);
                            // layer handle it.
                            let _ = layer_handle_recv_delivery(
                                gid,
                                &out_send,
                                t.into(),
                                tid,
                                is_ok,
                                data,
                            )
                            .await;
                        }
                    }
                }
            }
        })
        .detach();

        Ok(peer_id)
    }
}
