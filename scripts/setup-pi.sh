#!/bin/bash
# Rustassistant Raspberry Pi Setup Script
# Run as: sudo bash setup-pi.sh

set -e

echo "🔧 Rustassistant Pi Setup"
echo "========================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
DATA_DIR="/var/lib/rustassistant"
BIN_DIR="/usr/local/bin"
CONFIG_DIR="/etc/rustassistant"
LOG_DIR="/var/log/rustassistant"
USER="${SUDO_USER:-pi}"

echo -e "\n${YELLOW}Step 1: Creating directories${NC}"
mkdir -p "$DATA_DIR"
mkdir -p "$DATA_DIR/backups"
mkdir -p "$DATA_DIR/cache"
mkdir -p "$CONFIG_DIR"
mkdir -p "$LOG_DIR"

# Set permissions - owned by the user, not root
chown -R "$USER:$USER" "$DATA_DIR"
chown -R "$USER:$USER" "$CONFIG_DIR"
chown -R "$USER:$USER" "$LOG_DIR"
chmod 755 "$DATA_DIR"
chmod 755 "$CONFIG_DIR"
chmod 755 "$LOG_DIR"

echo -e "${GREEN}✓ Directories created${NC}"

echo -e "\n${YELLOW}Step 2: Installing rclone${NC}"
if command -v rclone &> /dev/null; then
    echo -e "${GREEN}✓ rclone already installed${NC}"
else
    curl https://rclone.org/install.sh | bash
    echo -e "${GREEN}✓ rclone installed${NC}"
fi

echo -e "\n${YELLOW}Step 3: Creating environment file${NC}"
if [ ! -f "$CONFIG_DIR/rustassistant.env" ]; then
    cat > "$CONFIG_DIR/rustassistant.env" << 'EOF'
# Rustassistant Environment Configuration

# Database
RUSTASSISTANT_DB_PATH=/var/lib/rustassistant/rustassistant.db
RUSTASSISTANT_ENV=production
RUSTASSISTANT_AUTO_MIGRATE=true

# Server
RUSTASSISTANT_PORT=3000

# LLM (Grok)
XAI_API_KEY=your_api_key_here
XAI_MODEL=grok-4.1
XAI_MAX_TOKENS=4096

# GitHub
GITHUB_USERNAME=nuniesmith

# Backup
BACKUP_REMOTE_NAME=gdrive
BACKUP_REMOTE_PATH=rustassistant-backups
BACKUP_RETENTION_COUNT=30
EOF
    chown "$USER:$USER" "$CONFIG_DIR/rustassistant.env"
    chmod 600 "$CONFIG_DIR/rustassistant.env"
    echo -e "${GREEN}✓ Environment file created at $CONFIG_DIR/rustassistant.env${NC}"
    echo -e "${YELLOW}⚠ Don't forget to add your XAI_API_KEY!${NC}"
else
    echo -e "${GREEN}✓ Environment file already exists${NC}"
fi

echo -e "\n${YELLOW}Step 4: Creating systemd service${NC}"
cat > /etc/systemd/system/rustassistant.service << EOF
[Unit]
Description=Rustassistant Developer Workflow Manager
After=network.target

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$DATA_DIR
EnvironmentFile=$CONFIG_DIR/rustassistant.env
ExecStart=$BIN_DIR/rustassistant-server
Restart=on-failure
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=true
ReadWritePaths=$DATA_DIR $LOG_DIR

# Logging
StandardOutput=append:$LOG_DIR/rustassistant.log
StandardError=append:$LOG_DIR/rustassistant-error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo -e "${GREEN}✓ Systemd service created${NC}"

echo -e "\n${YELLOW}Step 5: Setting up backup cron job${NC}"
# Create backup script
cat > "$BIN_DIR/rustassistant-backup" << 'EOF'
#!/bin/bash
# Rustassistant automated backup script

source /etc/rustassistant/rustassistant.env

LOG_FILE="/var/log/rustassistant/backup.log"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

echo "[$TIMESTAMP] Starting backup..." >> "$LOG_FILE"

# Run backup
/usr/local/bin/rustassistant backup create >> "$LOG_FILE" 2>&1

if [ $? -eq 0 ]; then
    echo "[$TIMESTAMP] Backup completed successfully" >> "$LOG_FILE"
else
    echo "[$TIMESTAMP] Backup failed!" >> "$LOG_FILE"
fi
EOF
chmod +x "$BIN_DIR/rustassistant-backup"
chown "$USER:$USER" "$BIN_DIR/rustassistant-backup"

# Add cron job (as the user, not root)
CRON_JOB="0 2 * * * $BIN_DIR/rustassistant-backup"
(sudo -u "$USER" crontab -l 2>/dev/null | grep -v rustassistant-backup; echo "$CRON_JOB") | sudo -u "$USER" crontab -
echo -e "${GREEN}✓ Daily backup cron job added (2 AM)${NC}"

echo -e "\n${YELLOW}Step 6: Summary${NC}"
echo "========================="
echo ""
echo "Data directory:    $DATA_DIR"
echo "Config directory:  $CONFIG_DIR"
echo "Log directory:     $LOG_DIR"
echo "Service user:      $USER"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo "1. Add your API key:"
echo "   sudo nano $CONFIG_DIR/rustassistant.env"
echo ""
echo "2. Configure Google Drive backup:"
echo "   rclone config"
echo "   (Create a remote named 'gdrive' - see: rustassistant backup setup)"
echo ""
echo "3. Build and install the binary:"
echo "   cargo build --release"
echo "   sudo cp target/release/rustassistant-server $BIN_DIR/"
echo "   sudo cp target/release/rustassistant $BIN_DIR/"
echo ""
echo "4. Start the service:"
echo "   sudo systemctl enable rustassistant"
echo "   sudo systemctl start rustassistant"
echo ""
echo "5. Check status:"
echo "   sudo systemctl status rustassistant"
echo "   tail -f $LOG_DIR/rustassistant.log"
echo ""
echo "6. Test backup:"
echo "   rustassistant backup check"
echo "   rustassistant backup create"
echo ""
echo -e "${GREEN}Setup complete!${NC}"
