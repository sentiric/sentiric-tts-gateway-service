.PHONY: help setup network up down logs test build clean

# Ağ Ayarları
NETWORK_NAME := sentiric-net
SUBNET := 10.88.0.0/16
GATEWAY := 10.88.0.1

help: ## Komutları listeler
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Ortam dosyasını hazırlar
	@if [ ! -f .env ]; then cp .env.example .env; echo "✅ .env oluşturuldu."; fi

network: ## Ortak Docker ağını oluşturur (Varsa atlar)
	@docker network inspect $(NETWORK_NAME) >/dev/null 2>&1 || \
	(echo "🌐 Creating network $(NETWORK_NAME)..." && \
	docker network create --driver bridge --subnet $(SUBNET) --gateway $(GATEWAY) $(NETWORK_NAME))

up: setup network ## Servisi başlatır (Önce ağ kontrolü yapar)
	docker compose up --build -d

down: ## Servisi durdurur
	docker compose down --remove-orphans

logs: ## Logları izler
	docker compose logs -f

test: ## Birim testleri çalıştırır
	cargo test

build: ## Release build alır
	cargo build --release

clean: ## Temizlik yapar
	cargo clean
	rm -rf target/