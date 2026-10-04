# MTECH OS — targets
#   make skills | ask MSG="..." | watch | serve | app
#   make module | kernel | iso | iso-everything | install | smoke

AGENT := agent
PY    := python3

.PHONY: skills ask watch serve app module kernel iso iso-everything install smoke clean

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

install:
	sudo bash install.sh

smoke:
	$(PY) -m compileall -q $(AGENT)/mtech_agent gui/mtech_shell.py && echo "[OK] python"
	for f in install.sh kernel/build-kernel.sh kali/build-iso.sh; do bash -n $$f && echo "[OK] $$f"; done
	cd $(AGENT) && $(PY) -m mtech_agent probe >/dev/null && echo "[OK] probe"
	cd $(AGENT) && $(PY) -m mtech_agent skills | head -3

clean:
	rm -rf build/kali-work $(AGENT)/mtech_agent/__pycache__ gui/__pycache__
