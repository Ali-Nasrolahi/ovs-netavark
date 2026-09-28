%global crate ovs-netavark
%global bin_name ovs-netavark
%global netavark_plugin_dir %{_libexecdir}/netavark

Name:           %{crate}
Version:        %(sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n1)
Release:        1%{?dist}
Summary:        Open vSwitch plugin for Netavark
License:        Apache-2.0

URL:            https://github.com/Ali-Nasrolahi/%{crate}
Source0:        %{url}/archive/v%{version}/%{crate}-%{version}.tar.gz
Source1:        %{url}/releases/download/v%{version}/vendor.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo >= 1.70
BuildRequires:  protobuf-compiler
BuildRequires:  gcc
BuildRequires:  make

Requires:   netavark >= 1.6
Requires:   /usr/bin/ovs-vsctl

ExclusiveArch:  %{rust_arches}

%description
Minimal Open vSwitch network plugin for Netavark, providing L2 container connectivity.

%prep
%autosetup -n %{crate}-%{version}

%setup -q -T -D -a 1

mkdir -p .cargo
cat > .cargo/config.toml <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
export CARGO_PROFILE_RELEASE_LTO="true"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1"
export CARGO_PROFILE_RELEASE_DEBUG="2"

cargo build --release --offline --locked %{?_smp_mflags}

%install
install -d -m 0755 %{buildroot}%{netavark_plugin_dir}
install -m 0755 target/release/%{bin_name} \
    %{buildroot}%{netavark_plugin_dir}/%{bin_name}

%files
%doc README.md
%license LICENSE
%dir %{netavark_plugin_dir}
%{netavark_plugin_dir}/%{bin_name}

%changelog
* Mon Sep 28 2026 Ali Nasrollahi <A.Nasrolahi01@gmail.com> - 0.1.0-1
- Initial package

```