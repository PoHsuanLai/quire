"""ComfyUI API graphs, one builder per (model, mode). Pure: values in, a dict out.

Klein graphs follow ComfyUI's bundled "Flux.2 Klein 4B Distilled" templates (text to image and
image edit), with the GGUF loader swapped in. Qwen graphs follow the bundled Qwen-Image and
Qwen-Image-Edit-2511 templates, GGUF loader swapped in, no speed LoRA.
"""

from __future__ import annotations

from dataclasses import dataclass

KLEIN_UNET = "flux-2-klein-4b-Q4_K_M.gguf"
KLEIN_CLIP = "qwen_3_4b_fp4_flux2.safetensors"
KLEIN_VAE = "flux2-vae.safetensors"

QWEN_UNET = "qwen-image-2512-Q3_K_M.gguf"
QWEN_EDIT_UNET = "qwen-image-edit-2511-Q3_K_M.gguf"
QWEN_CLIP = "qwen_2.5_vl_7b_nvfp4.safetensors"
QWEN_VAE = "qwen_image_vae.safetensors"


@dataclass(frozen=True)
class Job:
    """What one render needs; `reference` is a ComfyUI input filename (already uploaded)."""

    prompt: str
    negative: str
    seed: int
    size: int
    prefix: str
    reference: str | None = None


@dataclass(frozen=True)
class Sampling:
    steps: int
    cfg: float


KLEIN_SAMPLING = Sampling(steps=4, cfg=1.0)
QWEN_SAMPLING = Sampling(steps=20, cfg=2.5)
QWEN_EDIT_SAMPLING = Sampling(steps=20, cfg=4.0)


def _save(images: list, prefix: str) -> dict:
    return {"class_type": "SaveImage", "inputs": {"images": images, "filename_prefix": prefix}}


def klein(job: Job) -> dict:
    s = KLEIN_SAMPLING
    g: dict = {
        "unet": {"class_type": "UnetLoaderGGUF", "inputs": {"unet_name": KLEIN_UNET}},
        "clip": {
            "class_type": "CLIPLoader",
            "inputs": {"clip_name": KLEIN_CLIP, "type": "flux2", "device": "default"},
        },
        "vae": {"class_type": "VAELoader", "inputs": {"vae_name": KLEIN_VAE}},
        "pos": {"class_type": "CLIPTextEncode", "inputs": {"clip": ["clip", 0], "text": job.prompt}},
        "neg": {"class_type": "ConditioningZeroOut", "inputs": {"conditioning": ["pos", 0]}},
        "sampler_select": {"class_type": "KSamplerSelect", "inputs": {"sampler_name": "euler"}},
        "sigmas": {
            "class_type": "Flux2Scheduler",
            "inputs": {"steps": s.steps, "width": job.size, "height": job.size},
        },
        "noise": {"class_type": "RandomNoise", "inputs": {"noise_seed": job.seed}},
        "latent": {
            "class_type": "EmptyFlux2LatentImage",
            "inputs": {"width": job.size, "height": job.size, "batch_size": 1},
        },
        "sample": {
            "class_type": "SamplerCustomAdvanced",
            "inputs": {
                "noise": ["noise", 0],
                "guider": ["guider", 0],
                "sampler": ["sampler_select", 0],
                "sigmas": ["sigmas", 0],
                "latent_image": ["latent", 0],
            },
        },
        "decode": {"class_type": "VAEDecode", "inputs": {"samples": ["sample", 0], "vae": ["vae", 0]}},
        "save": _save(["decode", 0], job.prefix),
    }
    positive, negative = ["pos", 0], ["neg", 0]
    if job.reference is not None:
        g["ref_load"] = {"class_type": "LoadImage", "inputs": {"image": job.reference}}
        g["ref_scale"] = {
            "class_type": "ImageScaleToTotalPixels",
            "inputs": {"image": ["ref_load", 0], "upscale_method": "lanczos", "megapixels": 1.0,
                       "resolution_steps": 1},
        }
        g["ref_latent"] = {
            "class_type": "VAEEncode", "inputs": {"pixels": ["ref_scale", 0], "vae": ["vae", 0]},
        }
        g["pos_ref"] = {
            "class_type": "ReferenceLatent",
            "inputs": {"conditioning": ["pos", 0], "latent": ["ref_latent", 0]},
        }
        g["neg_ref"] = {
            "class_type": "ReferenceLatent",
            "inputs": {"conditioning": ["neg", 0], "latent": ["ref_latent", 0]},
        }
        positive, negative = ["pos_ref", 0], ["neg_ref", 0]
    g["guider"] = {
        "class_type": "CFGGuider",
        "inputs": {"model": ["unet", 0], "positive": positive, "negative": negative, "cfg": s.cfg},
    }
    return g


def _qwen_common(unet: str) -> dict:
    return {
        "unet": {"class_type": "UnetLoaderGGUF", "inputs": {"unet_name": unet}},
        "clip": {
            "class_type": "CLIPLoader",
            "inputs": {"clip_name": QWEN_CLIP, "type": "qwen_image", "device": "default"},
        },
        "vae": {"class_type": "VAELoader", "inputs": {"vae_name": QWEN_VAE}},
    }


def _ksampler(job: Job, s: Sampling, model: list, latent: list) -> dict:
    return {
        "class_type": "KSampler",
        "inputs": {
            "model": model, "positive": ["pos", 0], "negative": ["neg", 0],
            "latent_image": latent, "seed": job.seed, "steps": s.steps, "cfg": s.cfg,
            "sampler_name": "euler", "scheduler": "simple", "denoise": 1.0,
        },
    }


def qwen(job: Job) -> dict:
    g = _qwen_common(QWEN_UNET)
    g["shift"] = {"class_type": "ModelSamplingAuraFlow", "inputs": {"model": ["unet", 0], "shift": 3.1}}
    g["pos"] = {"class_type": "CLIPTextEncode", "inputs": {"clip": ["clip", 0], "text": job.prompt}}
    g["neg"] = {"class_type": "CLIPTextEncode", "inputs": {"clip": ["clip", 0], "text": job.negative}}
    g["latent"] = {
        "class_type": "EmptySD3LatentImage",
        "inputs": {"width": job.size, "height": job.size, "batch_size": 1},
    }
    g["sample"] = _ksampler(job, QWEN_SAMPLING, ["shift", 0], ["latent", 0])
    g["decode"] = {"class_type": "VAEDecode", "inputs": {"samples": ["sample", 0], "vae": ["vae", 0]}}
    g["save"] = _save(["decode", 0], job.prefix)
    return g


def qwen_edit(job: Job) -> dict:
    if job.reference is None:
        raise ValueError("qwen_edit needs a reference image")
    g = _qwen_common(QWEN_EDIT_UNET)
    g["shift"] = {"class_type": "ModelSamplingAuraFlow", "inputs": {"model": ["unet", 0], "shift": 3.1}}
    g["norm"] = {"class_type": "CFGNorm", "inputs": {"model": ["shift", 0], "strength": 1.0}}
    g["ref_load"] = {"class_type": "LoadImage", "inputs": {"image": job.reference}}
    g["ref_scale"] = {"class_type": "FluxKontextImageScale", "inputs": {"image": ["ref_load", 0]}}
    for key, text in (("pos_raw", job.prompt), ("neg_raw", "")):
        g[key] = {
            "class_type": "TextEncodeQwenImageEditPlus",
            "inputs": {"clip": ["clip", 0], "vae": ["vae", 0], "image1": ["ref_scale", 0], "prompt": text},
        }
    for key, raw in (("pos", "pos_raw"), ("neg", "neg_raw")):
        g[key] = {
            "class_type": "FluxKontextMultiReferenceLatentMethod",
            "inputs": {"conditioning": [raw, 0], "reference_latents_method": "index_timestep_zero"},
        }
    g["latent"] = {
        "class_type": "EmptySD3LatentImage",
        "inputs": {"width": job.size, "height": job.size, "batch_size": 1},
    }
    g["sample"] = _ksampler(job, QWEN_EDIT_SAMPLING, ["norm", 0], ["latent", 0])
    g["decode"] = {"class_type": "VAEDecode", "inputs": {"samples": ["sample", 0], "vae": ["vae", 0]}}
    g["save"] = _save(["decode", 0], job.prefix)
    return g


BUILDERS = {"klein": klein, "qwen": qwen, "qwen-edit": qwen_edit}
