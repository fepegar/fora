# Example: Submitting a Job

This guide walks through setting up a Python project with [uv](https://docs.astral.sh/uv/) and submitting it to Azure ML using `fora submit`.

## 1. Create a new project

Use `uv` to scaffold a new Python project:

```bash
uv init my-training-job
cd my-training-job
```

This creates the following structure:

```
my-training-job/
├── .python-version
├── pyproject.toml
├── README.md
└── main.py
```

## 2. Add dependencies

Add the packages your training script needs:

```bash
uv add torch torchvision
```

This updates `pyproject.toml` and creates a `uv.lock` lockfile.

## 3. Write your training script

Edit `main.py` (or create a new script):

```python
import torch
import torch.nn as nn

def main():
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Using device: {device}")

    model = nn.Linear(10, 1).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)

    # Your training loop here
    for epoch in range(10):
        x = torch.randn(32, 10, device=device)
        y = torch.randn(32, 1, device=device)
        loss = nn.functional.mse_loss(model(x), y)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(f"Epoch {epoch}: loss={loss.item():.4f}")

if __name__ == "__main__":
    main()
```

## 4. Create a `.amlignore` file

Fora respects `.amlignore` (and `.gitignore`) when uploading code. Create one to exclude unnecessary files:

```
.venv/
__pycache__/
*.pyc
.git/
```

## 5. Create a Dockerfile (optional)

Fora auto-generates a Dockerfile for uv-based environments, but you can also provide your own. Here's the template Fora uses:

```dockerfile
FROM mcr.microsoft.com/azureml/openmpi5.0-cuda12.6-ubuntu24.04

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

RUN groupadd --system --gid 999 nonroot \
 && useradd --system --gid 999 --uid 999 --create-home nonroot

WORKDIR /env
ENV UV_PROJECT_ENVIRONMENT=/env/.venv
ENV UV_LINK_MODE=copy

RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=uv.lock,target=uv.lock \
    --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    --mount=type=bind,source=.python-version,target=.python-version \
    uv sync --locked --no-install-project

WORKDIR /workdir
CMD ["bash"]
```

The key feature is that the environment is deterministically named by hashing `pyproject.toml`, `uv.lock`, `.python-version`, and the config. If the hash matches an existing environment, it is reused — no rebuild needed.

## 6. Submit the job

```bash
fora submit \
  -s 00000000-0000-0000-0000-000000000000 \
  -r my-resource-group \
  -w my-workspace \
  -e training-experiment \
  -c gpu-cluster \
  -- main.py
```

### What happens

1. **Code upload** — Fora walks your project directory (respecting `.amlignore`), and uploads all files concurrently to Azure Blob Storage.
2. **Environment creation** — Fora hashes your dependency files to create a deterministic environment name. If the environment already exists, it's reused. Otherwise, a new Docker image is built.
3. **Job submission** — A `CommandJob` is created on Azure ML. The default command is `uv run python main.py`.

Steps 1 and 2 run concurrently for faster submission.

## 7. Monitor the job

Once submitted, you can monitor the job using the Fora TUI:

```bash
fora
```

Navigate to the **Recent Jobs** tab to see your job's status, or use the **Experiments** tab to find it by experiment name.

## Tips

- **Extra dependency groups**: Use `--uv-group` to include optional dependency groups defined in your `pyproject.toml`:
  ```bash
  fora submit ... --uv-group dev -- main.py
  ```

- **Environment variables**: Pass secrets or config via `--set`:
  ```bash
  fora submit ... --set WANDB_API_KEY=xxx -- main.py
  ```

- **Custom base image**: Override the default base Docker image:
  ```bash
  fora submit ... --base-docker-image python:3.13-slim -- main.py
  ```

- **Data inputs**: Mount datasets from Azure ML data assets or datastores:
  ```bash
  fora submit ... --mount data=my-dataset:latest -- main.py
  ```
