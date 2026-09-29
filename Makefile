# MTAALAMU SMART — production targets
#   make build | test | health | run-ui | deep

.PHONY: build release test health deep sysprobe agentic run-ui clean

ENGINE := engine-rust
BIN := $(ENGINE)/target/release/mtaalamu

build:
	cd $(ENGINE) && cargo build --release

release: build
	@mkdir -p dist
	cp -f $(BIN) dist/mtaalamu 2>/dev/null || cp -f $(ENGINE)/target/release/mtaalamu.exe dist/mtaalamu.exe
	@echo "Binary in dist/"

test:
	cd $(ENGINE) && cargo test --release

health: build
	@$(BIN) help >/dev/null && echo "[OK] CLI help"
	@$(BIN) deep --top 5 >/tmp/mtaalamu_deep.json && echo "[OK] deep probe -> /tmp/mtaalamu_deep.json"
	@$(BIN) sysprobe --top 5 >/tmp/mtaalamu_sys.json && echo "[OK] sysprobe"
	@test -f data/agents/agents_10.json && echo "[OK] agents_10.json" || echo "[WARN] agents data missing"
	@test -f web-r/app.R && echo "[OK] web-r/app.R" || echo "[WARN] UI missing"
	@echo "Health complete."

deep: build
	$(BIN) deep --top 12

sysprobe: build
	$(BIN) sysprobe --top 10

agentic: build
	$(BIN) agentic --msg "Production smoke: kompyuta inaenda polepole" --approve

run-ui:
	Rscript -e "shiny::runApp('web-r', port=3838, host='127.0.0.1')"

clean:
	cd $(ENGINE) && cargo clean
	rm -rf dist
