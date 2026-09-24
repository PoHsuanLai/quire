"""Sample `nvidia-smi` while a render runs, to report the peak VRAM a model adds."""

from __future__ import annotations

import subprocess
import threading
from dataclasses import dataclass, field


def used_mib() -> int:
    out = subprocess.run(
        ["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"],
        capture_output=True, text=True, check=True,
    ).stdout
    return int(out.strip().splitlines()[0])


@dataclass
class PeakSampler:
    """Context manager: records the peak `memory.used` while the block runs (0.5 s period)."""

    period_s: float = 0.5
    peak: int = 0
    _stop: threading.Event = field(default_factory=threading.Event)

    def _loop(self) -> None:
        while not self._stop.is_set():
            self.peak = max(self.peak, used_mib())
            self._stop.wait(self.period_s)

    def __enter__(self) -> "PeakSampler":
        self._thread = threading.Thread(target=self._loop, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *exc: object) -> None:
        self._stop.set()
        self._thread.join()
