# MTAALAMU SMART — production targets
#   make build | test | health | run-ui | deep
#   make mobile-build | mobile-run | mobile-release | mobile-clean

.PHONY: build release test health deep sysprobe agentic run-ui clone-iot mobile-build mobile-run mobile-release mobile-clean clean

ENGINE := engine-rust
ENGINE_BIN := $(ENGINE)/target/release/mtaalamu
MOBILE := fundi-mobile
MOBILE_BIN := $(MOBILE)/target/release/fundi-mobile

build:
	cd $(ENGINE) && cargo build --release

release: build
	@mkdir -p dist
	cp -f $(ENGINE_BIN) dist/mtaalamu 2>/dev/null || cp -f $(ENGINE)/target/release/mtaalamu.exe dist/mtaalamu.exe
	@echo "Binary in dist/"

test:
	cd $(ENGINE) && cargo test --release

health: build
	@$(ENGINE_BIN) help >/dev/null && echo "[OK] CLI help"
	@$(ENGINE_BIN) deep --top 5 >/tmp/mtaalamu_deep.json && echo "[OK] deep probe -> /tmp/mtaalamu_deep.json"
	@$(ENGINE_BIN) sysprobe --top 5 >/tmp/mtaalamu_sys.json && echo "[OK] sysprobe"
	@test -f data/agents/agents_10.json && echo "[OK] agents_10.json" || echo "[WARN] agents data missing"
	@test -f web-r/app.R && echo "[OK] web-r/app.R" || echo "[WARN] UI missing"
	@echo "Health complete."

deep: build
	$(ENGINE_BIN) deep --top 12

sysprobe: build
	$(ENGINE_BIN) sysprobe --top 10

agentic: build
	$(ENGINE_BIN) agentic --msg "Production smoke: kompyuta inaenda polepole" --approve

run-ui:
	Rscript -e "shiny::runApp('web-r', port=3838, host='127.0.0.1')"

# --- IoT + HERMES (afya · usalama · nyumbani · kilimo) ---
clone-iot:
	bash scripts/clone_iot_repos.sh --list
	@echo ""
	@echo "Clone zote:    bash scripts/clone_iot_repos.sh"
	@echo "Vertical moja: bash scripts/clone_iot_repos.sh --only kilimo"
	@echo "+ agents OSS:  bash scripts/clone_iot_repos.sh --agents"

# --- FUNDI MOBILE 🩺 ---
mobile-build:
	cd $(MOBILE) && cargo build --release

mobile-release: mobile-build
	@mkdir -p dist
	cp -f $(MOBILE_BIN) dist/fundi-mobile 2>/dev/null || cp -f $(MOBILE)/target/release/fundi-mobile.exe dist/fundi-mobile.exe
	@echo "Binary in dist/"

mobile-run: mobile-build
	$(MOBILE_BIN) help

mobile-clean:
	cd $(MOBILE) && cargo clean

clean:
	cd $(ENGINE) && cargo clean
	cd $(MOBILE) && cargo clean
	rm -rf dist
