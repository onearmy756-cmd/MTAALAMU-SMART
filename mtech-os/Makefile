# MTECH OS — targets
#   make skills | ask MSG="..." | watch | serve | app
#   make module | kernel | iso | iso-everything | install | smoke

AGENT := agent
PY    := python3

.PHONY: skills ask watch serve app module kernel iso iso-everything iso-arm64 docker-build docker-run install smoke clean

skills:
	cd $(AGENT) && $(PY) -m mtech_agent skills

ask:
	cd $(AGENT) && $(PY) -m mtech_agent ask "$(MSG)"

watch:
	cd $(AGENT) && $(PY) -m mtech_agent watch

serve:
	cd $(AGENT) && $(PY) -m mtech_agent serve

app:
	$(PY) gui/mtech_shell.py

shell: app

module:
	bash kernel/build-kernel.sh --module-only

kernel:
	bash kernel/build-kernel.sh --install

iso:
	sudo bash kali/build-iso.sh

iso-everything:
	sudo bash kali/build-iso.sh --everything

iso-arm64:
	sudo bash kali/build-iso.sh --arch arm64

docker-build:
	docker build -t mtech-os -f kali/Dockerfile.mtech ..

docker-run:
	bash kali/docker-run.sh

install:
	sudo bash install.sh

smoke:
	$(PY) -m compileall -q $(AGENT)/mtech_agent gui/mtech_shell.py desktop/system-apps-window.py desktop/kali-tools-window.py && echo "[OK] python"
	for f in install.sh kernel/build-kernel.sh kali/build-iso.sh kali/docker-run.sh; do bash -n $$f && echo "[OK] $$f"; done
	$(PY) -m compileall -q ../hermes-agent/mtaalamu && echo "[OK] mtaalamu"
	cd $(AGENT) && $(PY) -m mtech_agent probe >/dev/null && echo "[OK] probe"
	cd $(AGENT) && $(PY) -m mtech_agent skills | head -3

clean:
	rm -rf build/kali-work $(AGENT)/mtech_agent/__pycache__ gui/__pycache__
