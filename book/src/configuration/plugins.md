# Plugins

pzsh includes 9 built-in plugins inspired by oh-my-zsh, optimized for O(1) loading.

## Enabling Plugins

In your `~/.pzshrc`:

```toml
[plugins]
enabled = ["git", "docker", "kubectl", "npm", "python", "golang", "rust", "terraform", "aws"]
```

## Available Plugins

### git (14 aliases)

Common git shortcuts:

| Alias | Command |
|-------|---------|
| `g` | `git` |
| `gs` | `git status` |
| `ga` | `git add` |
| `gaa` | `git add --all` |
| `gc` | `git commit` |
| `gcm` | `git commit -m` |
| `gco` | `git checkout` |
| `gb` | `git branch` |
| `gd` | `git diff` |
| `gl` | `git pull` |
| `gp` | `git push` |
| `gf` | `git fetch` |
| `glog` | `git log --oneline --graph` |
| `gst` | `git stash` |

### docker (9 aliases)

Docker and Docker Compose shortcuts:

| Alias | Command |
|-------|---------|
| `d` | `docker` |
| `dc` | `docker compose` |
| `dcu` | `docker compose up` |
| `dcd` | `docker compose down` |
| `dps` | `docker ps` |
| `di` | `docker images` |
| `drm` | `docker rm` |
| `drmi` | `docker rmi` |
| `dex` | `docker exec -it` |

### kubectl (15 aliases)

Kubernetes CLI shortcuts:

| Alias | Command |
|-------|---------|
| `k` | `kubectl` |
| `kgp` | `kubectl get pods` |
| `kgs` | `kubectl get services` |
| `kgd` | `kubectl get deployments` |
| `kgn` | `kubectl get nodes` |
| `kga` | `kubectl get all` |
| `kd` | `kubectl describe` |
| `kdp` | `kubectl describe pod` |
| `kl` | `kubectl logs` |
| `klf` | `kubectl logs -f` |
| `kex` | `kubectl exec -it` |
| `ka` | `kubectl apply -f` |
| `kdel` | `kubectl delete` |
| `kctx` | `kubectl config current-context` |
| `kns` | `kubectl config set-context --current --namespace` |

### npm (21 aliases)

Node.js package manager shortcuts (npm, yarn, pnpm):

| Alias | Command |
|-------|---------|
| `ni` | `npm install` |
| `nid` | `npm install --save-dev` |
| `nig` | `npm install -g` |
| `nr` | `npm run` |
| `nrs` | `npm run start` |
| `nrb` | `npm run build` |
| `nrt` | `npm run test` |
| `nrd` | `npm run dev` |
| `nu` | `npm update` |
| `nci` | `npm ci` |
| `y` | `yarn` |
| `ya` | `yarn add` |
| `yad` | `yarn add --dev` |
| `yr` | `yarn run` |
| `ys` | `yarn start` |
| `yb` | `yarn build` |
| `yt` | `yarn test` |
| `pn` | `pnpm` |
| `pni` | `pnpm install` |
| `pna` | `pnpm add` |
| `pnr` | `pnpm run` |

### python (15 aliases)

Python development shortcuts:

| Alias | Command |
|-------|---------|
| `py` | `python3` |
| `py2` | `python2` |
| `pip` | `pip3` |
| `pir` | `pip install -r requirements.txt` |
| `pie` | `pip install -e .` |
| `piu` | `pip install --upgrade` |
| `pif` | `pip freeze` |
| `venv` | `python3 -m venv` |
| `va` | `source venv/bin/activate` |
| `vd` | `deactivate` |
| `pt` | `pytest` |
| `ptv` | `pytest -v` |
| `ptx` | `pytest -x` |
| `uvi` | `uv pip install` |
| `uvr` | `uv pip install -r requirements.txt` |

### golang (11 aliases)

Go development shortcuts:

| Alias | Command |
|-------|---------|
| `gob` | `go build` |
| `gor` | `go run` |
| `got` | `go test` |
| `gotv` | `go test -v` |
| `gof` | `go fmt ./...` |
| `gom` | `go mod` |
| `gomt` | `go mod tidy` |
| `gomi` | `go mod init` |
| `gog` | `go get` |
| `goi` | `go install` |
| `gov` | `go vet ./...` |

### rust (13 aliases)

Rust/Cargo development shortcuts:

| Alias | Command |
|-------|---------|
| `c` | `cargo` |
| `cb` | `cargo build` |
| `cbr` | `cargo build --release` |
| `cr` | `cargo run` |
| `crr` | `cargo run --release` |
| `ct` | `cargo test` |
| `cc` | `cargo check` |
| `ccl` | `cargo clippy` |
| `cf` | `cargo fmt` |
| `cu` | `cargo update` |
| `ca` | `cargo add` |
| `cdo` | `cargo doc --open` |
| `cw` | `cargo watch -x` |

### terraform (15 aliases)

Terraform and OpenTofu shortcuts:

| Alias | Command |
|-------|---------|
| `tf` | `terraform` |
| `tfi` | `terraform init` |
| `tfp` | `terraform plan` |
| `tfa` | `terraform apply` |
| `tfaa` | `terraform apply -auto-approve` |
| `tfd` | `terraform destroy` |
| `tff` | `terraform fmt` |
| `tfv` | `terraform validate` |
| `tfo` | `terraform output` |
| `tfs` | `terraform state` |
| `tfw` | `terraform workspace` |
| `tofu` | `tofu` |
| `tofui` | `tofu init` |
| `tofup` | `tofu plan` |
| `tofua` | `tofu apply` |

### aws (10 aliases)

AWS CLI shortcuts:

| Alias | Command |
|-------|---------|
| `awsw` | `aws sts get-caller-identity` |
| `awsl` | `aws configure list` |
| `awsp` | `aws configure list-profiles` |
| `s3ls` | `aws s3 ls` |
| `s3cp` | `aws s3 cp` |
| `s3sync` | `aws s3 sync` |
| `ec2ls` | `aws ec2 describe-instances` |
| `ecsls` | `aws ecs list-clusters` |
| `lamls` | `aws lambda list-functions` |
| `ssm` | `aws ssm start-session --target` |

## Performance

All plugins load in O(1) time with no external scripts:

```
Total plugin load time: ~17µs (all 9 plugins)
Individual plugin: ~1-5µs each
```

This is 1000x faster than oh-my-zsh plugin loading.

## Custom Plugins

You can create custom plugins by implementing the `Plugin` trait:

```rust
use pzsh::plugin::{Plugin, PluginInfo, PluginError};
use ahash::AHashMap;

#[derive(Clone, Debug)]
struct MyPlugin {
    enabled: bool,
}

impl Plugin for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("my-plugin")
            .with_description("My custom plugin")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: pzsh::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("myalias".to_string(), "my-command".to_string());
        aliases
    }
}
```
