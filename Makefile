PLUGIN_NAME	:= ovs-netavark
RELEASE_BIN	:= $(CURDIR)/target/release/$(PLUGIN_NAME)
PLUGIN_DIR	:= /usr/local/libexec/netavark
PLUGIN_LINK	:= $(PLUGIN_DIR)/$(PLUGIN_NAME)

TEST_BRIDGE		:= ovs-netv-test
TEST_NETWORK	:= ovs-netv-test
TEST_CONTAINER	:= ovs-netv-test-container

all:
	cargo build

test:
	@echo "==> Building release binary"
	@cargo build --release

	@echo "==> Installing temporary plugin symlink"
	@sudo mkdir -p "$(PLUGIN_DIR)"
	@sudo ln -sfn "$(RELEASE_BIN)" "$(PLUGIN_LINK)"

	@echo "==> Creating test OVS bridge"
	@sudo ovs-vsctl --may-exist add-br "$(TEST_BRIDGE)"

	@echo "==> Removing stale test container/network"
	@sudo podman rm -f "$(TEST_CONTAINER)" >/dev/null 2>&1 || true
	@sudo podman network rm "$(TEST_NETWORK)" >/dev/null 2>&1 || true

	@echo "==> Creating Podman network"
	@sudo podman network create \
		--driver "$(PLUGIN_NAME)" \
		--opt bridge="$(TEST_BRIDGE)" \
		"$(TEST_NETWORK)"

	@echo "==> Inspecting network"
	@sudo podman network inspect "$(TEST_NETWORK)"

	@echo "==> Running Alpine container"
	@sudo podman run \
		--name $(TEST_CONTAINER) \
		--network $(TEST_NETWORK) \
		--rm \
		-d \
		alpine:latest \
		sleep 30

	@sudo podman exec $(TEST_CONTAINER) ip addr

	@echo "==> Inspecting OVS bridge"
	@sudo ovs-vsctl show
	@sudo ovs-vsctl list-ports $(TEST_BRIDGE)

	@echo "==> Cleaning up"
	@sudo podman kill "$(TEST_CONTAINER)" >/dev/null 2>&1 || true
	@sudo podman network rm "$(TEST_NETWORK)" >/dev/null 2>&1 || true
	@sudo ovs-vsctl --if-exists del-br "$(TEST_BRIDGE)"
	@sudo rm -f "$(PLUGIN_LINK)"

	@echo "==> Test complete"

clean:
	sudo podman rm -f "$(TEST_CONTAINER)"
	sudo podman network rm -f "$(TEST_NETWORK)"
	sudo ovs-vsctl --if-exists del-br "$(TEST_BRIDGE)"
	sudo rm -f "$(PLUGIN_LINK)"

.PHONY: all test clean