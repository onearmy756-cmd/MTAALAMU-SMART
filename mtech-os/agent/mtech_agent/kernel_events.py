"""MTECH OS — matukio ya kernel (kernel events).

Chanzo cha kwanza: /dev/mtech (kernel module ya MTECH — kprobes za
exec/exit zikiandika kwenye ring buffer). Fallback: uchunguzi wa /proc
(processes mpya) ili mfumo ufanye kazi hata bila module.
"""
import os
import threading
import time
from pathlib import Path


class EventFeed(threading.Thread):
    """Thread inayosoma matukio ya kernel na kuyahifadhi kwa GUI/agent."""

    def __init__(self, maxlen: int = 500):
        super().__init__(daemon=True)
        self.events: list = []
        self.maxlen = maxlen
        self._lock = threading.Lock()
        self._stop = threading.Event()
        self.source = "proc-fallback"
        self._dev = Path("/dev/mtech")
        self._seen_pids: dict = {}

    # ---------------------------------------------------------- public
    def run(self) -> None:
        from .platform import IS_LINUX
        if IS_LINUX and self._dev.exists():
            self.source = "dev/mtech"
            self._read_dev()
        elif IS_LINUX:
            self.source = "proc-fallback"
            self._poll_proc()
        else:
            # Windows: Sysmon Event Log; macOS: Unified Log (halisi)
            self._read_platform()
            if self.source in ("sysmon", "unified-log"):
                return
            self.source = "platform-fallback"
            self._poll_proc()

    def stop(self) -> None:
        self._stop.set()

    def snapshot(self, limit: int = 30) -> list:
        with self._lock:
            return list(self.events[-limit:])

    # ---------------------------------------------------------- sources
    def _push(self, ev: dict) -> None:
        with self._lock:
            self.events.append(ev)
            if len(self.events) > self.maxlen:
                self.events = self.events[-self.maxlen:]

    def _read_platform(self) -> None:
        """Windows (Sysmon Event Log) na macOS (Unified Log) — HALISI."""
        from .platform import tail_events
        stop = threading.Event()
        self.source = tail_events(lambda ev: self._push(ev), stop)
        while not self._stop.is_set():
            time.sleep(0.5)
        stop.set()

    def _read_dev(self) -> None:
        try:
            fd = os.open(str(self._dev), os.O_RDONLY | os.O_NONBLOCK)
        except OSError:
            self.source = "proc-fallback"
            return self._poll_proc()
        while not self._stop.is_set():
            try:
                chunk = os.read(fd, 4096).decode(errors="replace")
            except BlockingIOError:
                time.sleep(0.3)
                continue
            except OSError:
                break
            for line in chunk.splitlines():
                parts = line.strip().split("|")
                if len(parts) >= 4:
                    self._push({"ts": parts[0], "type": parts[1], "pid": parts[2], "detail": "|".join(parts[3:])})
            time.sleep(0.05)
        os.close(fd)

    def _poll_proc(self) -> None:
        while not self._stop.is_set():
            now = time.time()
            live = set()
            for d in Path("/proc").glob("[0-9]*"):
                pid = d.name
                live.add(pid)
                if pid not in self._seen_pids:
                    try:
                        comm = (d / "comm").read_text().strip()
                    except OSError:
                        continue
                    self._seen_pids[pid] = comm
                    self._push({"ts": str(int(now)), "type": "exec", "pid": pid, "detail": comm})
            gone = set(self._seen_pids) - live
            for pid in list(gone)[:50]:
                self._push({"ts": str(int(now)), "type": "exit", "pid": pid, "detail": self._seen_pids.pop(pid)})
            if len(self._seen_pids) > 5000:
                self._seen_pids = {}
            time.sleep(1.0)


FEED: EventFeed = None


def start_feed() -> EventFeed:
    """Rudisha feed iliyopo (thread isianzishwe mara mbili)."""
    global FEED
    if FEED is not None:
        return FEED  # tayari ipo — hata kama imefeli, usiianzishe tena
    FEED = EventFeed()
    FEED.start()
    return FEED
