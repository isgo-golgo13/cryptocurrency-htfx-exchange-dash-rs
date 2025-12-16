#!/bin/bash
# Firecracker Setup Script for BTC Exchange Dashboard
# 
# Prerequisites:
#   - Linux host with KVM support
#   - Firecracker binary installed
#   - Root/sudo access for networking
#
# Usage:
#   ./setup.sh build    # Build rootfs image
#   ./setup.sh network  # Setup network (requires sudo)
#   ./setup.sh run      # Launch microVM
#   ./setup.sh clean    # Cleanup

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK_DIR="/tmp/firecracker-btc-dash"
ROOTFS_SIZE_MB=256
VM_IP="172.16.0.2"
HOST_IP="172.16.0.1"
TAP_DEV="tap0"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check prerequisites
check_prereqs() {
    log_info "Checking prerequisites..."
    
    # Check for KVM
    if [ ! -e /dev/kvm ]; then
        log_error "KVM not available. Ensure you're on Linux with KVM enabled."
    fi
    
    # Check for Firecracker
    if ! command -v firecracker &> /dev/null; then
        log_error "Firecracker not found. Install from: https://github.com/firecracker-microvm/firecracker/releases"
    fi
    
    # Check for required tools
    for cmd in docker mkfs.ext4 curl; do
        if ! command -v $cmd &> /dev/null; then
            log_error "$cmd not found. Please install it."
        fi
    done
    
    log_info "Prerequisites OK"
}

# Build the rootfs image
build_rootfs() {
    log_info "Building rootfs image..."
    
    mkdir -p "$WORK_DIR"
    cd "$WORK_DIR"
    
    # Build static binary
    log_info "Building static binary (musl)..."
    cd "$PROJECT_ROOT"
    
    rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
    
    RUSTFLAGS='-C target-feature=+crt-static' \
        cargo build --release --target x86_64-unknown-linux-musl \
        -p dash-server
    
    # Build WASM frontend
    log_info "Building WASM frontend..."
    cd "$PROJECT_ROOT/crates/dash-app"
    trunk build --release
    
    cd "$WORK_DIR"
    
    # Create Alpine-based rootfs
    log_info "Creating Alpine rootfs..."
    mkdir -p rootfs
    
    # Export Alpine container
    CONTAINER_ID=$(docker create alpine:3.19)
    docker export "$CONTAINER_ID" | tar -C rootfs -xf -
    docker rm "$CONTAINER_ID"
    
    # Copy binary
    cp "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/dash-server" rootfs/usr/bin/
    chmod +x rootfs/usr/bin/dash-server
    
    # Copy WASM dist
    mkdir -p rootfs/var/www/dist
    cp -r "$PROJECT_ROOT/crates/dash-app/dist/"* rootfs/var/www/dist/
    
    # Create init script
    cat > rootfs/init << 'EOF'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev

# Configure network
ip addr add 172.16.0.2/24 dev eth0
ip link set eth0 up
ip route add default via 172.16.0.1

# Start server
cd /var/www
exec /usr/bin/dash-server
EOF
    chmod +x rootfs/init
    
    # Create ext4 image
    log_info "Creating ext4 image..."
    dd if=/dev/zero of=rootfs.ext4 bs=1M count=$ROOTFS_SIZE_MB
    mkfs.ext4 -F rootfs.ext4
    
    mkdir -p mnt
    sudo mount rootfs.ext4 mnt
    sudo cp -a rootfs/* mnt/
    sudo umount mnt
    
    # Copy to deploy directory
    cp rootfs.ext4 "$SCRIPT_DIR/"
    
    log_info "Rootfs built: $SCRIPT_DIR/rootfs.ext4"
}

# Download kernel (if not present)
download_kernel() {
    if [ -f "$SCRIPT_DIR/vmlinux" ]; then
        log_info "Kernel already present"
        return
    fi
    
    log_info "Downloading kernel..."
    KERNEL_URL="https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin"
    curl -fsSL -o "$SCRIPT_DIR/vmlinux" "$KERNEL_URL"
    log_info "Kernel downloaded"
}

# Setup network
setup_network() {
    log_info "Setting up network..."
    
    # Create TAP device
    sudo ip tuntap add $TAP_DEV mode tap
    sudo ip addr add $HOST_IP/24 dev $TAP_DEV
    sudo ip link set $TAP_DEV up
    
    # Enable IP forwarding
    sudo sysctl -w net.ipv4.ip_forward=1
    
    # NAT for outbound traffic
    sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
    sudo iptables -A FORWARD -i $TAP_DEV -j ACCEPT
    sudo iptables -A FORWARD -o $TAP_DEV -j ACCEPT
    
    log_info "Network configured. VM will be at $VM_IP:3001"
}

# Cleanup network
cleanup_network() {
    log_info "Cleaning up network..."
    
    sudo ip link del $TAP_DEV 2>/dev/null || true
    sudo iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE 2>/dev/null || true
    
    log_info "Network cleaned up"
}

# Run Firecracker
run_vm() {
    log_info "Starting Firecracker VM..."
    
    cd "$SCRIPT_DIR"
    
    # Check for required files
    [ -f vmlinux ] || log_error "Kernel not found. Run: $0 kernel"
    [ -f rootfs.ext4 ] || log_error "Rootfs not found. Run: $0 build"
    
    # Create socket
    API_SOCKET="/tmp/firecracker-btc.sock"
    rm -f "$API_SOCKET"
    
    # Launch Firecracker
    sudo firecracker \
        --api-sock "$API_SOCKET" \
        --config-file vm-config.json
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."
    
    cleanup_network
    rm -rf "$WORK_DIR"
    rm -f /tmp/firecracker-btc.sock
    rm -f /tmp/firecracker.log
    rm -f /tmp/firecracker-metrics.json
    
    log_info "Cleanup complete"
}

# Main
case "${1:-help}" in
    prereqs)
        check_prereqs
        ;;
    build)
        check_prereqs
        build_rootfs
        ;;
    kernel)
        download_kernel
        ;;
    network)
        setup_network
        ;;
    run)
        run_vm
        ;;
    clean)
        cleanup
        ;;
    all)
        check_prereqs
        download_kernel
        build_rootfs
        setup_network
        run_vm
        ;;
    *)
        echo "Usage: $0 {prereqs|build|kernel|network|run|clean|all}"
        echo ""
        echo "Commands:"
        echo "  prereqs  - Check prerequisites"
        echo "  build    - Build rootfs with server + WASM"
        echo "  kernel   - Download Linux kernel"
        echo "  network  - Setup TAP network (requires sudo)"
        echo "  run      - Launch Firecracker VM"
        echo "  clean    - Cleanup temporary files"
        echo "  all      - Run all steps"
        ;;
esac
