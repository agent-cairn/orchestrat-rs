# CI Workflow (Manual Addition Required)

This file contains the CI workflow that should be added as `.github/workflows/ci.yml` after merging the scaffold PR.

The GitHub OAuth token for agent-cairn lacks the `workflow` scope, which is required to push workflow files via the API. This workflow must be added manually.

---

## File: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, feature/**]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
      - name: Run cargo check
        run: cargo check --all-features --all-targets
      - name: Run cargo clippy
        run: cargo clippy --all-features --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    needs: check
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
      - name: Run cargo test
        run: cargo test --all-features

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check
```

---

## How to Add Manually

1. After merging the scaffold PR, clone the repo locally:
   ```bash
   git clone git@github.com:agent-cairn/orchestrat-rs.git
   cd orchestrat-rs
   ```

2. Create the workflow file:
   ```bash
   mkdir -p .github/workflows
   cat > .github/workflows/ci.yml << 'EOF'
   # (paste the YAML content above)
   EOF
   ```

3. Commit and push:
   ```bash
   git add .github/workflows/ci.yml
   git commit -m "Add CI workflow: check, test, fmt"
   git push origin HEAD
   ```

4. Verify CI is running: Check https://github.com/agent-cairn/orchestrat-rs/actions

---

## Why Manual Addition?

The agent-cairn GitHub OAuth token lacks the `workflow` scope, which is required to:
- Create or update `.github/workflows/*` files via the GitHub API
- Push workflow files via git using OAuth credentials

This is a security feature to prevent automated workflows from being modified without explicit approval.

After the token scope is elevated (if desired), future workflow updates can be automated. For now, this manual step ensures CI is in place.
