#!/bin/bash
set -e
echo "FUNDI DEPLOY — site install"
sudo apt-get update
sudo apt-get install -y docker.io docker-compose-v2 wakeonlan arp-scan || true
cd "$(dirname "$0")/.."
mkdir -p images data tftp/fundi
sudo docker compose up -d --build
echo "API: http://$(hostname -I | awk '{print $1}'):8080/health"
