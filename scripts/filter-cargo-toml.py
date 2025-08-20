#!/usr/bin/env python3
"""
Filter Cargo.toml for dependency-only builds in Docker.

This script creates a minimal workspace containing only:
- xtask (required by workspace)
- ci_optims (our dependency compilation crate)

It removes all workspace dependencies that reference local crates (path = "crates/*")
to avoid missing dependency errors during the dependency pre-compilation stage.
"""

import sys
import os


def filter_cargo_toml(input_file, output_file):
  """Filter Cargo.toml to create a minimal workspace for dependency compilation."""

  with open(input_file, "r") as f:
    content = f.read()

  lines = content.split("\n")
  filtered_lines = []
  in_members_section = False
  in_workspace_deps_section = False
  skip_line = False

  for line in lines:
    # Track which section we're in
    if line.strip().startswith("[workspace]"):
      in_members_section = False
      in_workspace_deps_section = False
    elif line.strip() == "members = [":
      in_members_section = True
    elif line.strip().startswith("[workspace.dependencies]"):
      in_workspace_deps_section = True
      in_members_section = False
    elif line.strip().startswith("[") and not line.strip().startswith("[workspace"):
      in_members_section = False
      in_workspace_deps_section = False

    # Filter members section - keep only ci_optims (xtask has local dependencies)
    if in_members_section:
      if '"crates/ci_optims"' in line:
        filtered_lines.append(line)
      elif line.strip() in ["]", "members = ["]:
        filtered_lines.append(line)
      elif line.strip().startswith('"'):
        # Skip all other members (including xtask which has local deps)
        continue
      else:
        filtered_lines.append(line)

    # Filter workspace dependencies - remove local path dependencies
    elif in_workspace_deps_section:
      if 'path = "crates/' in line:
        # Skip this line and the next line if it's a continuation
        skip_line = True
        continue
      elif skip_line and line.strip() == "":
        # Skip empty line after removed dependency
        skip_line = False
        continue
      else:
        skip_line = False
        filtered_lines.append(line)

    # Keep all other lines
    else:
      filtered_lines.append(line)

  # Write filtered content
  with open(output_file, "w") as f:
    f.write("\n".join(filtered_lines))

  print(f"✅ Filtered Cargo.toml written to {output_file}")
  print("   - Kept members: ci_optims only")
  print("   - Removed local workspace dependencies")
  print("   - Removed xtask (has local dependencies)")


if __name__ == "__main__":
  if len(sys.argv) != 3:
    print("Usage: python3 filter-cargo-toml.py <input_file> <output_file>")
    sys.exit(1)

  input_file = sys.argv[1]
  output_file = sys.argv[2]

  if not os.path.exists(input_file):
    print(f"Error: Input file {input_file} not found")
    sys.exit(1)

  filter_cargo_toml(input_file, output_file)
