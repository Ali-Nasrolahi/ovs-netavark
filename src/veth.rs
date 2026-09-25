use netavark::error::{NetavarkError, NetavarkResult};
use netavark::network::core_utils::{CoreUtils, open_netlink_sockets};
use netavark::network::netlink_route::{CreateLinkOptions, LinkID, parse_create_link_options};
use netlink_packet_route::link::{InfoData, InfoKind, InfoVeth, LinkAttribute, LinkMessage};
use std::os::unix::prelude::AsFd;

pub fn add_pair(ns: &str, peer_iface: &str) -> NetavarkResult<(String, String)> {
    let (mut hostns, mut netns) = open_netlink_sockets(ns)?;

    let mut peer_opts = CreateLinkOptions::new(peer_iface.to_string(), InfoKind::Veth);
    peer_opts.netns = Some(netns.file.as_fd());

    let mut peer = LinkMessage::default();
    parse_create_link_options(&mut peer, peer_opts);

    let mut host_veth_opts = CreateLinkOptions::new(String::new(), InfoKind::Veth);
    host_veth_opts.info_data = Some(InfoData::Veth(InfoVeth::Peer(peer)));

    hostns.netlink.create_link(host_veth_opts)?;

    let veth = netns
        .netlink
        .get_link(LinkID::Name(peer_iface.to_string()))?;

    let mut mac = String::new();
    let mut host_link_index = 0;

    for nla in veth.attributes.into_iter() {
        if let LinkAttribute::Address(ref addr) = nla {
            mac = CoreUtils::encode_address_to_hex(addr);
        }
        if let LinkAttribute::Link(link) = nla {
            host_link_index = link;
        }
    }

    if mac.is_empty() {
        return Err(NetavarkError::msg(
            "failed to get mac address from container veth",
        ));
    }

    if host_link_index == 0 {
        return Err(NetavarkError::msg(
            "failed to get host link index from container veth",
        ));
    }

    let host_veth_msg = hostns.netlink.get_link(LinkID::ID(host_link_index))?;
    let mut host_veth_name = String::new();

    for nla in host_veth_msg.attributes.into_iter() {
        if let LinkAttribute::IfName(name) = nla {
            host_veth_name = name;
            break;
        }
    }

    if host_veth_name.is_empty() {
        return Err(NetavarkError::msg(
            "failed to get auto-generated name for host veth",
        ));
    }

    hostns.netlink.set_up(LinkID::ID(host_link_index))?;
    netns.netlink.set_up(LinkID::ID(veth.header.index))?;

    Ok((host_veth_name, mac))
}

pub fn rm_pair(ns: &str, cont_iface: &str) -> NetavarkResult<Option<String>> {
    let (mut hostns, mut netns) = match open_netlink_sockets(ns) {
        Ok(sockets) => sockets,
        Err(_) => return Ok(None), // Namespace already gone, nothing to clean up
    };

    let cont_link = match netns.netlink.get_link(LinkID::Name(cont_iface.to_string())) {
        Ok(link) => link,
        Err(_) => return Ok(None), // Interface already gone
    };

    let mut host_link_index = 0;
    for nla in cont_link.attributes {
        if let LinkAttribute::Link(idx) = nla {
            host_link_index = idx;
            break;
        }
    }

    if host_link_index == 0 {
        return Err(NetavarkError::msg(format!(
            "interface '{}' does not have a host peer (not a veth?)",
            cont_iface
        )));
    }

    let host_link = hostns.netlink.get_link(LinkID::ID(host_link_index))?;
    let mut host_name = String::new();

    for nla in host_link.attributes {
        if let LinkAttribute::IfName(name) = nla {
            host_name = name;
            break;
        }
    }

    if host_name.is_empty() {
        return Err(NetavarkError::msg("failed to retrieve host veth name"));
    }

    hostns.netlink.del_link(LinkID::ID(host_link_index))?;

    Ok(Some(host_name))
}
