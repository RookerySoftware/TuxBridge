#!/bin/sh
# Makes `docker` (and `docker compose`) in a distro use Pengi's Docker engine, as Docker
# Desktop's WSL integration does for its own. Run as root inside the distro:
#
#   docker-client.sh install FILES USER    FILES: a folder with docker, docker-compose (static
#                                          Linux builds) and the client's ca.pem, cert.pem,
#                                          key.pem; USER: the account that gets the key
#   docker-client.sh remove
#
# `docker` becomes a small script in /usr/local/bin that points the real command at the engine
# (127.0.0.1:2376, which all WSL systems share) with the client's certificate. A distro with a
# docker of its own is left alone.

set -eu
LIB=/usr/local/lib/easywsl
CERTS=/etc/easywsl/docker-client
WRAPPER=/usr/local/bin/docker
MARK="# EasyWSL's docker"

case ${1:-} in
install)
    FILES=${2:?folder}
    U=${3:?user}
    if command -v docker > /dev/null && ! grep -q "$MARK" "$(command -v docker)" 2> /dev/null; then
        echo "this system has a docker of its own ($(command -v docker)); it is left as it is" >&2
        exit 3
    fi
    mkdir -p "$LIB" "$CERTS" /usr/local/lib/docker/cli-plugins
    install -m 755 "$FILES/docker" "$LIB/docker"
    [ -f "$FILES/docker-compose" ] && install -m 755 "$FILES/docker-compose" /usr/local/lib/docker/cli-plugins/docker-compose
    install -m 644 "$FILES/ca.pem" "$FILES/cert.pem" "$CERTS/"
    install -m 600 "$FILES/key.pem" "$CERTS/key.pem"
    chown "$U" "$CERTS/key.pem"
    cat > "$WRAPPER" <<EOF
#!/bin/sh
$MARK: talks to Pengi's Docker engine (the Docker page in Pengi on Windows).
# Remove with: sudo /usr/local/lib/easywsl/docker-client.sh remove
[ -n "\${DOCKER_HOST:-}" ] || export DOCKER_HOST=tcp://127.0.0.1:2376 DOCKER_TLS_VERIFY=1 DOCKER_CERT_PATH=$CERTS
exec $LIB/docker "\$@"
EOF
    chmod 755 "$WRAPPER"
    install -m 755 "$0" "$LIB/docker-client.sh"
    echo "docker in this system now uses Pengi's Docker engine."
    ;;
remove)
    grep -q "$MARK" "$WRAPPER" 2> /dev/null && rm -f "$WRAPPER"
    rm -rf "$CERTS" "$LIB/docker" /usr/local/lib/docker/cli-plugins/docker-compose
    echo "docker is no longer set up here."
    ;;
*)
    echo "usage: docker-client.sh install FILES USER | remove" >&2
    exit 2
    ;;
esac
