"""The only module that talks to ComfyUI: submit a graph, wait for it, fetch the PNG."""

from __future__ import annotations

import json
import time
import urllib.parse
import urllib.request
import uuid
from dataclasses import dataclass
from pathlib import Path

DEFAULT_URL = "http://127.0.0.1:8188"


class ComfyError(RuntimeError):
    pass


@dataclass(frozen=True)
class Output:
    png: bytes
    seconds: float


def _get(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=30) as r:
        return json.load(r)


def upload(base: str, path: Path) -> str:
    """POST /upload/image; returns the name ComfyUI stored it under."""
    boundary = uuid.uuid4().hex
    body = b"".join([
        f"--{boundary}\r\n".encode(),
        f'Content-Disposition: form-data; name="image"; filename="{path.name}"\r\n'.encode(),
        b"Content-Type: image/png\r\n\r\n",
        path.read_bytes(),
        f"\r\n--{boundary}\r\n".encode(),
        b'Content-Disposition: form-data; name="overwrite"\r\n\r\ntrue',
        f"\r\n--{boundary}--\r\n".encode(),
    ])
    req = urllib.request.Request(
        f"{base}/upload/image", data=body,
        headers={"Content-Type": f"multipart/form-data; boundary={boundary}"},
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)["name"]


def submit(base: str, graph: dict) -> str:
    data = json.dumps({"prompt": graph, "client_id": "icongen"}).encode()
    req = urllib.request.Request(f"{base}/prompt", data=data, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return json.load(r)["prompt_id"]
    except urllib.error.HTTPError as e:
        raise ComfyError(f"/prompt rejected the graph: {e.read().decode()[:2000]}") from e


def wait(base: str, prompt_id: str, timeout_s: float = 1800.0) -> dict:
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        hist = _get(f"{base}/history/{prompt_id}")
        if prompt_id in hist:
            entry = hist[prompt_id]
            status = entry.get("status", {})
            if status.get("status_str") == "error":
                raise ComfyError(json.dumps(status.get("messages"))[:3000])
            if status.get("completed"):
                return entry
        time.sleep(1.0)
    raise ComfyError(f"prompt {prompt_id} did not finish in {timeout_s} s")


def fetch_first_image(base: str, entry: dict) -> bytes:
    for node in entry["outputs"].values():
        for img in node.get("images", []):
            q = urllib.parse.urlencode(
                {"filename": img["filename"], "subfolder": img["subfolder"], "type": img["type"]}
            )
            with urllib.request.urlopen(f"{base}/view?{q}", timeout=60) as r:
                return r.read()
    raise ComfyError("the finished prompt produced no image")


def run(base: str, graph: dict) -> Output:
    start = time.monotonic()
    entry = wait(base, submit(base, graph))
    return Output(png=fetch_first_image(base, entry), seconds=time.monotonic() - start)


def free(base: str) -> None:
    """Ask ComfyUI to unload models and free memory (between models: one model in VRAM at a time)."""
    req = urllib.request.Request(
        f"{base}/free", data=json.dumps({"unload_models": True, "free_memory": True}).encode(),
        headers={"Content-Type": "application/json"},
    )
    urllib.request.urlopen(req, timeout=60).close()
