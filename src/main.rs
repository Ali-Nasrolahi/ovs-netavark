mod ovs;
mod veth;

use std::collections::HashMap;

use netavark::{
    error::NetavarkError,
    network::types,
    plugin::{API_VERSION, Info, Plugin, PluginExec},
};

fn main() {
    let info = Info::new("0.1.0-dev".to_owned(), API_VERSION.to_owned(), None);
    PluginExec::new(Exec, info).exec();
}

struct Exec;

impl Plugin for Exec {
    fn create(&self, net_: types::Network) -> Result<types::Network, Box<dyn std::error::Error>> {
        ovs::ensure_br_exists(&net_.name)?;
        let net = types::Network {
            created: net_.created,
            dns_enabled: false,
            driver: net_.driver,
            id: net_.id,
            internal: true,
            ipv6_enabled: false,
            name: net_.name,
            network_interface: None,
            options: net_.options,
            ipam_options: Some(std::collections::HashMap::from([(
                "driver".to_string(),
                "none".to_string(),
            )])),
            subnets: None,
            routes: None,
            network_dns_servers: None,
            labels: net_.labels,
        };

        Ok(net)
    }

    fn setup(
        &self,
        ns: String,
        opts: types::NetworkPluginExec,
    ) -> Result<types::StatusBlock, Box<dyn std::error::Error>> {
        let peer_veth = if opts.network_options.interface_name.is_empty() {
            "eth0".to_owned()
        } else {
            opts.network_options.interface_name.clone()
        };

        let (host_veth, cont_mac) = veth::add_pair(&ns, &peer_veth)
            .map_err(|e| NetavarkError::wrap("failed to setup veth pair", e))?;

        ovs::add_port(&opts.network.name, &host_veth)
            .map_err(|e| NetavarkError::wrap("failed to add port to OVS bridge", e))?;

        Ok(types::StatusBlock {
            dns_search_domains: None,
            dns_server_ips: None,
            interfaces: Some(HashMap::from([(
                peer_veth,
                types::NetInterface {
                    mac_address: cont_mac,
                    subnets: None,
                },
            )])),
        })
    }

    fn teardown(
        &self,
        ns: String,
        opts: types::NetworkPluginExec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peer_veth = if opts.network_options.interface_name.is_empty() {
            "eth0".to_owned()
        } else {
            opts.network_options.interface_name.clone()
        };

        let host_veth = veth::rm_pair(&ns, &peer_veth)
            .map_err(|e| NetavarkError::wrap("failed to remove veth pair", e))?;

        if let Some(host_veth_name) = host_veth {
            ovs::del_port(&opts.network.name, &host_veth_name)
                .map_err(|e| NetavarkError::wrap("failed to remove port from OVS bridge", e))?;
        }

        Ok(())
    }
}
