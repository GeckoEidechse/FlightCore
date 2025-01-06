# %%
""""Ensure that version numbers between `tauri.conf.json`, `cargo.toml`, and GitHub release are the same"""
import json
import toml
import sys

with open("src-tauri/tauri.conf.json", "rt") as f:
    tauri_conf_json = json.load(f)

with open("src-tauri/Cargo.toml", "rt") as f:
    Cargo_toml = toml.load(f)

tauri_conf_json_version = tauri_conf_json["version"]
Cargo_toml_version = Cargo_toml["package"]["version"]

# Ensure same
assert(tauri_conf_json_version == Cargo_toml_version)

# Check release tag additionally if release
if "--release" in sys.argv:
    print("TODO")
    release_tag = sys.argv[2]
    print(release_tag)
    assert(release_tag == f"v{tauri_conf_json_version}")
