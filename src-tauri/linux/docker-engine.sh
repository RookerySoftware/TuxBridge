#!/bin/sh
# Pengi's Docker engine: the free Docker Engine in a small Alpine Linux system of its own
# (easywsl-docker), which nothing else runs in. Run as root inside it:
#
#   docker-engine.sh setup CERTS     installs Docker Engine and this script (as easywsl-docker),
#                                    and takes the engine's TLS files from the folder CERTS
#   easywsl-docker run               runs the engine until it is stopped (in the foreground: the
#                                    wsl.exe running it keeps the system running)
#   easywsl-docker stop              stops it
#
# The engine listens on its Unix socket and on 127.0.0.1:2376, which WSL forwards to Windows'
# localhost. That port takes only clients with a certificate from Pengi's own authority, kept
# in the Windows user's profile: other users and programs on the PC cannot use the engine.

set -eu
DIR=/etc/easywsl/docker

case ${1:-} in
setup)
    CERTS=${2:?folder with ca.pem, server-cert.pem and server-key.pem}
    apk update
    apk add docker ca-certificates tzdata
    mkdir -p "$DIR"
    cp "$CERTS/ca.pem" "$CERTS/server-cert.pem" "$CERTS/server-key.pem" "$DIR/"
    chmod 600 "$DIR/server-key.pem"
    # Containers' logs in Docker's compact format, kept to a few MB each.
    mkdir -p /etc/docker
    cat > /etc/docker/daemon.json <<'EOF'
{
  "log-driver": "local",
  "log-opts": { "max-size": "10m" }
}
EOF
    # Windows folders stay reachable (/mnt/c), for apps that share a folder with Windows. No
    # Windows programs on the PATH: this system only runs the engine.
    cat > /etc/wsl.conf <<'EOF'
[network]
hostname = easywsl-docker
[interop]
appendWindowsPath = false
EOF
    install -m 755 "$0" /usr/local/bin/easywsl-docker
    echo "Docker Engine $(dockerd --version | sed 's/^Docker version //') is installed."
    ;;
run)
    [ -f "$DIR/server-key.pem" ] || { echo "the engine is not set up" >&2; exit 1; }
    # A stopped engine's pid files would make it refuse to start.
    rm -f /var/run/docker.pid /var/run/docker/containerd/containerd.pid
    exec dockerd \
        --host unix:///var/run/docker.sock \
        --host tcp://127.0.0.1:2376 \
        --tlsverify \
        --tlscacert "$DIR/ca.pem" \
        --tlscert "$DIR/server-cert.pem" \
        --tlskey "$DIR/server-key.pem" \
        > /var/log/dockerd.log 2>&1
    ;;
stop)
    [ -f /var/run/docker.pid ] || exit 0
    pid=$(cat /var/run/docker.pid)
    kill -TERM "$pid" 2> /dev/null || exit 0
    # Docker stops the containers first, giving each 10 seconds.
    for _ in $(seq 60); do kill -0 "$pid" 2> /dev/null || exit 0; sleep 0.5; done
    echo "the engine did not stop in time" >&2
    exit 1
    ;;
*)
    echo "usage: docker-engine.sh setup CERTS | run | stop" >&2
    exit 2
    ;;
esac
