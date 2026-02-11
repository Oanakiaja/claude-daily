#!/usr/bin/env bash
# Daily - Context Archive System for Claude Code
# Installation Script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/oanakiaja/claude-daily/main/scripts/install.sh | bash
#
# Options:
#   DAILY_VERSION    - Install specific version (default: latest)
#   DAILY_INSTALL_DIR - Installation directory (default: ~/.local/bin)
#   DAILY_PORT       - Dashboard service port (default: 9999)
#   DAILY_NO_SERVICE - Skip service installation if set to 1
#   GITHUB_TOKEN     - GitHub token to bypass rate limiting

set -euo pipefail

# Configuration
REPO="oanakiaja/claude-daily"
BINARY_NAME="daily"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
DEFAULT_PORT=9999
LAUNCHD_LABEL="com.daily.dashboard"
LAUNCHD_PLIST="${HOME}/Library/LaunchAgents/${LAUNCHD_LABEL}.plist"
SYSTEMD_SERVICE="daily-dashboard"
SYSTEMD_UNIT_DIR="${HOME}/.config/systemd/user"
SYSTEMD_UNIT="${SYSTEMD_UNIT_DIR}/${SYSTEMD_SERVICE}.service"
LOG_DIR="${HOME}/.claude/daily/logs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and Architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Detect init system
detect_init_system() {
    case "$(uname -s)" in
        Darwin*) echo "launchd" ;;
        Linux*)
            if command -v systemctl &>/dev/null && systemctl --user status &>/dev/null 2>&1; then
                echo "systemd"
            else
                echo "unknown"
            fi
            ;;
        *) echo "unknown" ;;
    esac
}

# Prompt user for service port
prompt_service_port() {
    local port="${DAILY_PORT:-}"
    if [[ -n "$port" ]]; then
        echo "$port"
        return
    fi

    echo -n "Dashboard service port [${DEFAULT_PORT}]: " >&2
    read -r port
    if [[ -z "$port" ]]; then
        port="${DEFAULT_PORT}"
    fi

    if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 || "$port" -gt 65535 ]]; then
        warn "Invalid port '${port}', using default ${DEFAULT_PORT}"
        port="${DEFAULT_PORT}"
    fi

    echo "$port"
}

# Uninstall existing service
uninstall_service() {
    local init_system
    init_system=$(detect_init_system)

    case "$init_system" in
        launchd)
            if [[ -f "$LAUNCHD_PLIST" ]]; then
                launchctl unload "$LAUNCHD_PLIST" 2>/dev/null || true
                rm -f "$LAUNCHD_PLIST"
                info "Removed existing launchd service"
            fi
            ;;
        systemd)
            if systemctl --user is-active "$SYSTEMD_SERVICE" &>/dev/null; then
                systemctl --user stop "$SYSTEMD_SERVICE" 2>/dev/null || true
            fi
            if [[ -f "$SYSTEMD_UNIT" ]]; then
                systemctl --user disable "$SYSTEMD_SERVICE" 2>/dev/null || true
                rm -f "$SYSTEMD_UNIT"
                systemctl --user daemon-reload
                info "Removed existing systemd service"
            fi
            ;;
    esac
}

# Install launchd service (macOS)
install_launchd() {
    local binary_path="$1"
    local port="$2"

    mkdir -p "$(dirname "$LAUNCHD_PLIST")"
    mkdir -p "$LOG_DIR"

    cat > "$LAUNCHD_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${binary_path}</string>
        <string>show</string>
        <string>--no-open</string>
        <string>--port</string>
        <string>${port}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${LOG_DIR}/dashboard.out.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/dashboard.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:$(dirname "${binary_path}")</string>
    </dict>
</dict>
</plist>
EOF

    launchctl load "$LAUNCHD_PLIST"
    success "Installed launchd service (${LAUNCHD_LABEL})"
}

# Install systemd service (Linux)
install_systemd() {
    local binary_path="$1"
    local port="$2"

    mkdir -p "$SYSTEMD_UNIT_DIR"
    mkdir -p "$LOG_DIR"

    cat > "$SYSTEMD_UNIT" <<EOF
[Unit]
Description=Daily Dashboard - Context Archive for Claude Code
After=network.target

[Service]
Type=simple
ExecStart=${binary_path} show --no-open --port ${port}
Restart=on-failure
RestartSec=5
Environment=PATH=/usr/local/bin:/usr/bin:/bin:$(dirname "${binary_path}")

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload
    systemctl --user enable "$SYSTEMD_SERVICE"
    systemctl --user start "$SYSTEMD_SERVICE"
    success "Installed systemd service (${SYSTEMD_SERVICE})"

    if command -v loginctl &>/dev/null; then
        loginctl enable-linger "$USER" 2>/dev/null || true
    fi
}

# Install as system service
install_service() {
    local install_dir="$1"
    local port="$2"
    local binary_path="${install_dir}/${BINARY_NAME}"
    local init_system

    init_system=$(detect_init_system)

    if [[ "${DAILY_NO_SERVICE:-}" == "1" ]]; then
        info "Skipping service installation (DAILY_NO_SERVICE=1)"
        return
    fi

    if [[ "$init_system" == "unknown" ]]; then
        warn "No supported init system detected (need launchd or systemd)"
        warn "Skipping service installation. You can run 'daily show' manually."
        return
    fi

    echo ""
    echo -n "Install Daily as a background service? [Y/n]: "
    read -r answer
    if [[ "$answer" =~ ^[Nn]$ ]]; then
        info "Skipping service installation"
        return
    fi

    uninstall_service

    case "$init_system" in
        launchd)  install_launchd "$binary_path" "$port" ;;
        systemd)  install_systemd "$binary_path" "$port" ;;
    esac

    echo ""
    echo "  Dashboard: http://127.0.0.1:${port}"
    echo ""
    show_service_hints "$init_system" "$port"
}

# Show service management hints
show_service_hints() {
    local init_system="$1"
    local port="$2"

    echo "Service management:"
    case "$init_system" in
        launchd)
            echo "  Stop:    launchctl unload ${LAUNCHD_PLIST}"
            echo "  Start:   launchctl load ${LAUNCHD_PLIST}"
            echo "  Logs:    tail -f ${LOG_DIR}/dashboard.out.log"
            ;;
        systemd)
            echo "  Status:  systemctl --user status ${SYSTEMD_SERVICE}"
            echo "  Stop:    systemctl --user stop ${SYSTEMD_SERVICE}"
            echo "  Start:   systemctl --user start ${SYSTEMD_SERVICE}"
            echo "  Logs:    journalctl --user -u ${SYSTEMD_SERVICE} -f"
            ;;
    esac
    echo ""
}

# Curl wrapper with optional GitHub token auth
curl_github() {
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        curl -H "Authorization: token ${GITHUB_TOKEN}" "$@"
    else
        curl "$@"
    fi
}

# Show rate limit hint
show_rate_limit_hint() {
    echo ""
    warn "GitHub API rate limit exceeded"
    echo ""
    echo "To bypass rate limiting, set GITHUB_TOKEN environment variable:"
    echo ""
    echo "  GITHUB_TOKEN=your_token curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash"
    echo ""
    echo "You can create a token at: https://github.com/settings/tokens"
    echo ""
}

# Get latest release version (outputs __RATE_LIMITED__ on rate limit)
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local tmp_file
    tmp_file=$(mktemp)
    local http_code
    http_code=$(curl_github -sL --connect-timeout 10 --max-time 30 -w "%{http_code}" -o "$tmp_file" "$api_url") || true

    if [[ "$http_code" == "403" || "$http_code" == "429" ]]; then
        rm -f "$tmp_file"
        echo "__RATE_LIMITED__"
        return
    fi

    if [[ "$http_code" != "200" ]]; then
        rm -f "$tmp_file"
        return
    fi

    local version
    version=$(grep '"tag_name"' "$tmp_file" 2>/dev/null | sed -E 's/.*"([^"]+)".*/\1/' || true)
    rm -f "$tmp_file"
    echo "$version"
}

# Download and install binary
install_daily() {
    local platform="$1"
    local version="$2"
    local install_dir="$3"

    local artifact_name="${BINARY_NAME}-${platform}"
    if [[ "$platform" == "windows"* ]]; then
        artifact_name="${artifact_name}.exe"
    fi

    local download_url="https://github.com/${REPO}/releases/download/${version}/${artifact_name}"

    info "Downloading Daily ${version} for ${platform}..."
    info "URL: ${download_url}"

    # Create temp directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf ${tmp_dir}" EXIT

    local tmp_file="${tmp_dir}/${BINARY_NAME}"

    # Download binary
    local http_code
    http_code=$(curl_github -sL --connect-timeout 10 --max-time 120 -w "%{http_code}" -o "$tmp_file" "$download_url") || true

    if [[ "$http_code" == "403" || "$http_code" == "429" ]]; then
        show_rate_limit_hint
        exit 1
    fi

    if [[ "$http_code" != "200" ]]; then
        error "Failed to download binary from ${download_url} (HTTP $http_code)"
    fi

    # Make executable
    chmod +x "$tmp_file"

    # Create install directory if not exists
    mkdir -p "$install_dir"

    # Move to install directory
    local target="${install_dir}/${BINARY_NAME}"
    mv "$tmp_file" "$target"

    success "Installed Daily to ${target}"
}

# Run post-install setup
post_install() {
    local install_dir="$1"
    local port="$2"

    echo ""
    success "Daily has been installed successfully!"
    echo ""

    # Check if directory is in PATH
    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        warn "${install_dir} is not in your PATH"
        echo ""
        echo "Add the following to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:${install_dir}\""
        echo ""

        # Temporarily add to PATH for this session
        export PATH="$PATH:${install_dir}"
        info "Temporarily added ${install_dir} to PATH for this session"
        echo ""
    fi

    # Run daily init (interactive by default)
    echo ""
    info "Running initial setup..."
    echo ""
    "${install_dir}/${BINARY_NAME}" init

    # Install as system service
    install_service "$install_dir" "$port"

    echo ""
    echo "For more information, visit:"
    echo "  https://github.com/${REPO}"
}

main() {
    echo ""
    echo "  ╔═══════════════════════════════════════════════════╗"
    echo "  ║          Daily Installation Script                ║"
    echo "  ║    Context Archive System for Claude Code         ║"
    echo "  ╚═══════════════════════════════════════════════════╝"
    echo ""

    # Get platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    # Detect init system
    local init_system
    init_system=$(detect_init_system)
    info "Init system: ${init_system}"

    # Get version
    local version="${DAILY_VERSION:-}"
    if [[ -z "$version" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version) || true
        if [[ "$version" == "__RATE_LIMITED__" ]]; then
            show_rate_limit_hint
            exit 1
        fi
        if [[ -z "$version" ]]; then
            error "Failed to get latest version. Please specify DAILY_VERSION."
        fi
    fi
    info "Version: ${version}"

    # Get install directory
    local install_dir="${DAILY_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
    info "Install directory: ${install_dir}"

    # Get service port
    local port
    port=$(prompt_service_port)
    info "Dashboard port: ${port}"

    # Install
    install_daily "$platform" "$version" "$install_dir"

    # Post-install instructions (includes PATH check and service setup)
    post_install "$install_dir" "$port"
}

main "$@"
