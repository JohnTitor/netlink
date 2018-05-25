extern crate netlink;

use netlink::socket::sys::Socket;
use netlink::socket::{Protocol, SocketAddr};

use netlink::packet::constants::flags::{NLM_F_DUMP, NLM_F_REQUEST};
use netlink::packet::rtnl::{
    LinkFlags, LinkLayerType, NetlinkMessage, RtnlLinkHeader, RtnlLinkMessage, RtnlMessage,
};
use netlink::packet::NetlinkFlags;

fn main() {
    let mut socket = Socket::new(Protocol::Route).unwrap();
    let _port_number = socket.bind_auto().unwrap().port_number();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();

    let mut packet: NetlinkMessage = RtnlMessage::GetLink(RtnlLinkMessage {
        header: RtnlLinkHeader {
            address_family: 0, // AF_UNSPEC
            link_layer_type: LinkLayerType::Ether,
            flags: LinkFlags::new(),
        },
        nlas: vec![],
    }).into();
    packet.set_flags(NetlinkFlags::from(NLM_F_DUMP | NLM_F_REQUEST));
    packet.set_sequence_number(1);
    packet.finalize();
    let mut buf = vec![0; packet.length() as usize];
    packet.to_bytes(&mut buf[..]).unwrap();
    println!(">>> {:?}", packet);
    socket.send(&buf[..], 0).unwrap();

    let mut receive_buffer = vec![0; 4096];
    let mut offset = 0;

    // we set the NLM_F_DUMP flag so we expect a multipart rx_packet in response.
    loop {
        let size = socket.recv(&mut receive_buffer[..], 0).unwrap();

        loop {
            let rx_packet = NetlinkMessage::from_bytes(&receive_buffer[offset..]).unwrap();
            println!("<<< {:?}", rx_packet);

            if *rx_packet.message() == RtnlMessage::Done {
                println!("Done!");
                return;
            }

            offset += rx_packet.length() as usize;
            // the header.length() check avoids infinite loops
            if offset == size || rx_packet.length() == 0 {
                offset = 0;
                break;
            }
        }
    }
}
