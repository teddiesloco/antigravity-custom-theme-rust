# Decision: Use GitHub-Hosted `macos-latest` / `ubuntu-latest` for Public Fork CI

- **Date:** 2026-11-24
- **Context:** `teddiesloco/antigravity-custom-theme-rust` is a public fork of `chinhzz108/antigravity-custom-theme-rust` (GitHub does not allow making public forks private). The `vps-cloud` self-hosted runner pool (`/srv/gh-runner/ctl.sh`) is designed for private `teddiesloco/*` repositories and runs Linux without a global Rust toolchain. Our primary build target is macOS (`#[cfg(target_os = "macos")]`).
- **Choice:** Use GitHub-hosted `macos-latest` on PRs (plus `ubuntu-latest` on pushes to `main`) with `concurrency: cancel-in-progress` and `Swatinem/rust-cache@v2`.
- **Rationale:** $0 free on public repos, avoids exposing a self-hosted VPS runner to a public repository, and actually compiles/tests macOS-specific code paths in CI.
- **Reversibility:** Replace `runs-on` in `.github/workflows/ci.yml` via `ai-sdlc ci --scaffold --force` if the repository is ever detached into a private repo.
