"""icongen: render bake-off objects through a running ComfyUI (see tools/icongen/README.md)."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from . import brief, client, gpu, graphs

EDIT_MODEL = {"klein": "klein", "qwen": "qwen-edit"}


def _record(out: Path, row: dict) -> None:
    with (out / "runs.jsonl").open("a") as f:
        f.write(json.dumps(row) + "\n")


def _render(args: argparse.Namespace, model: str, job: graphs.Job, name: str) -> dict:
    graph = graphs.BUILDERS[model](job)
    base_mib = gpu.used_mib()
    with gpu.PeakSampler() as peak:
        result = client.run(args.url, graph)
    path = args.out / f"{name}.png"
    path.write_bytes(result.png)
    row = {
        "name": name, "model": model, "seed": job.seed, "size": job.size,
        "seconds": round(result.seconds, 2), "vram_before_mib": base_mib, "vram_peak_mib": peak.peak,
        "prompt": job.prompt, "reference": job.reference,
        "workflow_sha256": hashlib.sha256(json.dumps(graph, sort_keys=True).encode()).hexdigest(),
    }
    _record(args.out, row)
    print(f"{name}: {row['seconds']} s, peak {peak.peak} MiB (before {base_mib})", flush=True)
    return row


def cmd_bakeoff(args: argparse.Namespace) -> None:
    args.out.mkdir(parents=True, exist_ok=True)
    subjects = [brief.subject(s) for s in args.subjects] if args.subjects else brief.SUBJECTS
    for s in subjects:
        for seed in args.seeds:
            if (args.out / f"{s.slug}-s{seed}.png").exists():
                continue  # resume: a finished render is kept
            job = graphs.Job(brief.prompt(s, args.style), brief.negative(args.style), seed, args.size,
                             f"bakeoff-{args.model}-{args.style}-{s.slug}")
            _render(args, args.model, job, f"{s.slug}-s{seed}")


def cmd_vary(args: argparse.Namespace) -> None:
    args.out.mkdir(parents=True, exist_ok=True)
    ref = client.upload(args.url, args.hero)
    model = EDIT_MODEL[args.model]
    for s in brief.SUBJECTS:
        if s.slug == args.hero_subject or (args.out / f"{s.slug}-ref.png").exists():
            continue  # the hero itself, or a finished render (resume)
        job = graphs.Job(brief.variation_prompt(s), brief.NEGATIVE, args.seed, args.size,
                         f"vary-{args.model}-{s.slug}", reference=ref)
        _render(args, model, job, f"{s.slug}-ref")


def cmd_dump(args: argparse.Namespace) -> None:
    args.out.mkdir(parents=True, exist_ok=True)
    mail = brief.subject("mail")
    t2i = graphs.Job(mail.prompt(), brief.NEGATIVE, brief.SEEDS[0], 1024, "example")
    ref = graphs.Job(brief.variation_prompt(brief.subject("files")), brief.NEGATIVE, brief.SEEDS[0],
                     1024, "example", reference="hero.png")
    for name, builder, job in (("klein-t2i", graphs.klein, t2i), ("klein-ref", graphs.klein, ref),
                               ("qwen-t2i", graphs.qwen, t2i), ("qwen-edit-ref", graphs.qwen_edit, ref)):
        (args.out / f"{name}.json").write_text(json.dumps({"prompt": builder(job)}, indent=2) + "\n")


def cmd_free(args: argparse.Namespace) -> None:
    client.free(args.url)


def main() -> None:
    p = argparse.ArgumentParser(prog="icongen")
    p.add_argument("--url", default=client.DEFAULT_URL)
    sub = p.add_subparsers(required=True)

    b = sub.add_parser("bakeoff", help="every subject x seed through one model")
    b.add_argument("--model", choices=["klein", "qwen"], required=True)
    b.add_argument("--out", type=Path, required=True)
    b.add_argument("--size", type=int, default=1024)
    b.add_argument("--seeds", type=int, nargs="+", default=list(brief.SEEDS))
    b.add_argument("--subjects", nargs="*")
    b.add_argument("--style", choices=list(brief.STYLES), default="3d",
                   help="3d = round one's brief; flat = round two; flat-trigger = flat plus a style word")
    b.set_defaults(func=cmd_bakeoff)

    v = sub.add_parser("vary", help="reference-conditioned pass: the hero as image 1, every other subject")
    v.add_argument("--model", choices=["klein", "qwen"], required=True)
    v.add_argument("--hero", type=Path, required=True, help="hero object PNG (raw render)")
    v.add_argument("--hero-subject", required=True)
    v.add_argument("--out", type=Path, required=True)
    v.add_argument("--seed", type=int, default=brief.SEEDS[0])
    v.add_argument("--size", type=int, default=1024)
    v.set_defaults(func=cmd_vary)

    d = sub.add_parser("dump-workflows", help="write the API graphs used, for the record")
    d.add_argument("--out", type=Path, required=True)
    d.set_defaults(func=cmd_dump)

    f = sub.add_parser("free", help="unload models from ComfyUI")
    f.set_defaults(func=cmd_free)

    args = p.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
