# OVS Netavark Plugin

## Overview

`ovs-netavark` is a lightweight Open vSwitch network plugin for rootful Podman through Netavark.

It connects Podman containers to an existing Open vSwitch bridge by creating a veth pair, moving one
end into the container's network namespace, and attaching the host-side interface to the configured
OVS bridge.

The plugin intentionally provides only L2 connectivity. IP configuration, DHCP, DNS, routing, NAT,
and other higher-level network services remain outside its scope.

## Purpose

Open vSwitch provides a software-switching datapath that can serve as a common networking layer for
different types of workloads. In environments where VMs, VDI sessions, and other instances are
already managed through OVS, administrators can centralize networking concerns such as IPAM, DHCP,
VLAN isolation, QoS, traffic accounting, and OpenFlow policies in the surrounding network
infrastructure.

VMs and VDI sessions can naturally participate in this model through TAP interfaces attached to OVS.
Containers, however, are commonly connected using the container runtime's own networking stack,
creating a separate networking model and making them harder to incorporate into the same
infrastructure.

`ovs-netavark` brings rootful Podman containers into that existing OVS-based model. Each container
is represented by a normal OVS port, allowing the same OVS datapath and externally managed network
infrastructure to be used for containers without implementing those networking features inside the
plugin.

The project exists to provide the missing Netavark integration needed to make this model possible
for Podman containers.

## Behaviour

When a Podman network using `ovs-netavark` is created, the plugin validates that the configured OVS
bridge already exists. The bridge itself remains externally managed.

When a container joins the network, the plugin creates a veth pair using Linux netlink, places one
end in the container's network namespace, and attaches the host-side interface to the configured OVS
bridge. The container therefore receives an ordinary L2 interface without the plugin assigning an IP
address or configuring routes, DNS, DHCP, or other network services.

When the container is removed, the plugin removes the endpoint and its OVS port. The OVS bridge
remains intact and available for other workloads.
