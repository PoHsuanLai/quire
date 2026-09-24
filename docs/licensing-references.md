# Licensing references: image-model weights and tools

design/08-ICONS.md section 5 names this file as the record for model weight licences. This
first version covers the app-icon bake-off (2026-09-24, branch `icon-bakeoff`). Glyph sets
(Lucide ISC, Tabler MIT) get their rows when the glyphs land.

Licences were read from each Hugging Face repo's model card metadata through the HF API
(`/api/models/<repo>` → `cardData.license`, `gated`) on 2026-09-24. No repo is gated; no
token was used.

## Model weights used in the bake-off

| Weights (file on disk under `~/comfy/ComfyUI/models/`) | Source repo | Upstream model | Licence | Commercial use of outputs | sha256 |
| --- | --- | --- | --- | --- | --- |
| `unet/flux-2-klein-4b-Q4_K_M.gguf` (2.60 GB) | `unsloth/FLUX.2-klein-4B-GGUF` | `black-forest-labs/FLUX.2-klein-4B` (2026-01-14) | Apache-2.0 | yes (Apache-2.0 places no restriction on outputs; the 4B card says "open weights available for commercial use") | `0b25d143c8469b342bc5af3bce92b783bf6b0636d285f7b2f75e38af63af9a15` |
| `text_encoders/qwen_3_4b_fp4_flux2.safetensors` (3.85 GB) | `Comfy-Org/vae-text-encorder-for-flux-klein-4b` | Qwen3-4B, fp4 | Apache-2.0 | yes | `3eab03a77adb0ee5304a4e677d5c10ac22f9049c1d7c894adca4f8bb39206ca8` |
| `vae/flux2-vae.safetensors` (336 MB) | same | FLUX.2 VAE | Apache-2.0 | yes | `868fe7b343cc8f3a19dbcfcafbc3d5f888802be3f89bd81b65b3621a066ce8f3` |
| `unet/qwen-image-2512-Q3_K_M.gguf` (9.93 GB) | `unsloth/Qwen-Image-2512-GGUF` | `Qwen/Qwen-Image-2512` (2025-12-30) | Apache-2.0 | yes | QWEN_T2I_SHA |
| `unet/qwen-image-edit-2511-Q3_K_M.gguf` (9.92 GB) | `unsloth/Qwen-Image-Edit-2511-GGUF` | `Qwen/Qwen-Image-Edit-2511` (2025-12-17) | Apache-2.0 | yes | QWEN_EDIT_SHA |
| `text_encoders/qwen_2.5_vl_7b_nvfp4.safetensors` (6.11 GB) | `Comfy-Org/Qwen-Image_ComfyUI` | Qwen2.5-VL-7B, NVFP4 | Apache-2.0 | yes | `c7bb24d331f5df991bab2ce76a15195b12652e86a69a79823183c6a047407463` |
| `vae/qwen_image_vae.safetensors` (254 MB) | same | Qwen-Image VAE | Apache-2.0 | yes | `a70580f0213e67967ee9c95f05bb400e8fb08307e017a924bf3441223e023d1f` |

Correction to design/08-ICONS.md 3.2 and 5: the doc expects a separate "FLUX.2 Klein licence".
The 4B Klein release is plain Apache-2.0. `black-forest-labs/FLUX.2-klein-9B` and
`FLUX.2-dev` are `license: other`, `license_name: flux-non-commercial-license`, gated (HF API,
same date); they are not used, and nothing 9B may enter this pipeline. Design/08 can drop "verify before shipping"
for the 4B row once the user agrees.

Excluded on licence: `Qwen/Qwen-Image-2.1` (2026-09-21) is `license: other`, a non-commercial
research licence (see `~/comfy/README.md`); it is not downloaded.

Not used: `HiDream-ai/HiDream-O1-Image` (MIT), design/08's third candidate.

## Tools

| Tool | Licence | How it is used |
| --- | --- | --- |
| ComfyUI (`~/comfy/ComfyUI`, rev `b5cc8830279eae909a59de030af1e50761c36751`) | GPL-3.0 | local tool only, driven over HTTP by `tools/icongen`; never shipped, linked or copied from |
| ComfyUI-GGUF (city96, rev `6ea2651e7df66d7585f6ffee804b20e92fb38b8a`) | Apache-2.0 | custom node inside ComfyUI, tool only |
| `tools/icongen` | this workspace (MIT OR Apache-2.0) | stdlib-only Python client of ComfyUI's HTTP API; the graphs it builds were written from ComfyUI's bundled workflow templates' node lists (node names and parameter values are facts about the API, not copied code) |
| `tools/icons` | this workspace | pure-Rust post-process, `image` 0.25 and `clap` 4 from the pinned block |

## Outputs

Bake-off renders are not assets: they stay under `~/comfy/out/bakeoff/` and are not committed;
only the contact sheets under `tools/progress/shots/icons/` are. Shipped icons will carry "our
own licence" per design/08 section 5, with the recipe (model, weights sha256, workflow sha256,
seed, prompt) committed next to each.
