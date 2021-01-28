use chamomile::prelude::SendMessage;
use smol::channel::{SendError, Sender};
use tdn_types::{
    group::GroupId,
    message::{LayerReceiveMessage, LayerSendMessage, ReceiveMessage},
    primitive::{DeliveryType, PeerAddr, Result, StreamType},
};

#[inline]
pub(crate) async fn layer_handle_send(
    _fgid: &GroupId,
    _tgid: GroupId,
    p2p_send: &Sender<SendMessage>,
    msg: LayerSendMessage,
) -> std::result::Result<(), SendError<SendMessage>> {
    // TODO fgid, tgid serialize data to msg data.
    match msg {
        LayerSendMessage::Connect(tid, peer_addr, _domain, addr, data) => {
            //
            p2p_send
                .send(SendMessage::StableConnect(tid, peer_addr, addr, data))
                .await
        }
        LayerSendMessage::Disconnect(peer_addr) => {
            p2p_send
                .send(SendMessage::StableDisconnect(peer_addr))
                .await
        }
        LayerSendMessage::Result(tid, peer_addr, is_ok, is_force, result) => {
            //
            p2p_send
                .send(SendMessage::StableResult(
                    tid, peer_addr, is_ok, is_force, result,
                ))
                .await
        }
        LayerSendMessage::Event(tid, peer_addr, data) => {
            //
            p2p_send.send(SendMessage::Data(tid, peer_addr, data)).await
        }
        LayerSendMessage::Stream(id, stream) => {
            //
            p2p_send.send(SendMessage::Stream(id, stream)).await
        }
    }
}

#[inline]
pub(crate) async fn layer_handle_recv_connect(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Connect(peer_addr, data);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_result(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    is_ok: bool,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Result(peer_addr, is_ok, data);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_leave(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Leave(peer_addr);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_data(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    peer_addr: PeerAddr,
    data: Vec<u8>,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Event(peer_addr, data);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_stream(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    uid: u32,
    stream_type: StreamType,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Stream(uid, stream_type);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}

#[inline]
pub(crate) async fn layer_handle_recv_delivery(
    fgid: GroupId,
    out_send: &Sender<ReceiveMessage>,
    delivery_type: DeliveryType,
    tid: u64,
    is_sended: bool,
) -> Result<()> {
    let gmsg = LayerReceiveMessage::Delivery(delivery_type, tid, is_sended);

    let _tgid: GroupId = Default::default();

    #[cfg(any(feature = "single", feature = "std"))]
    let msg = ReceiveMessage::Layer(fgid, gmsg);
    #[cfg(any(feature = "multiple", feature = "full"))]
    let msg = ReceiveMessage::Layer(fgid, _tgid, gmsg);

    out_send
        .send(msg)
        .await
        .map_err(|e| error!("Outside channel: {:?}", e))
        .expect("Outside channel closed");

    Ok(())
}
