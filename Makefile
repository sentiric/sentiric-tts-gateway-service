.PHONY: help setup up down logs test build clean

help: ## Komutları listeler
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Ortam dosyasını hazırlar
	@if [ ! -f .env ]; then cp .env.example .env; echo "✅ .env oluşturuldu."; fi

up: setup ## Geliştirme ortamını başlatır
	docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build -d

down: ## Ortamı durdurur
	docker compose -f docker-compose.yml -f docker-compose.dev.yml down --remove-orphans

logs: ## Logları izler
	docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f tts-gateway-service

test: ## Birim testleri çalıştırır
	cargo test

build: ## Release build alır
	cargo build --release

clean: ## Temizlik yapar
	cargo clean
	rm -rf target/