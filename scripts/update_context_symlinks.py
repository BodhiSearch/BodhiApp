#!/usr/bin/env python3
"""
Script to manage symlinks in ai-docs/context/ for CLAUDE.md and PACKAGE.md files.

This script automatically creates and maintains symlinks for:
- CLAUDE.md files in crates and their test_utils subdirectories
- PACKAGE.md files in crates and their test_utils subdirectories
- Special handling for bodhi/src and bodhi/src-tauri directories
- Files in devops, .github, xtask, and project root directories

The script is idempotent and safe to run multiple times.
"""

from pathlib import Path
from typing import List, Tuple


def normalize_crate_name(crate_path: str) -> str:
  """Convert crate path to normalized name for symlink.

  Examples:
      auth_middleware -> auth-middleware
      llama_server_proc -> llama-server-proc
      bodhi/src -> bodhi-src
      bodhi/src-tauri -> bodhi-src-tauri
  """
  # Handle special bodhi cases
  if crate_path.startswith("bodhi/"):
    return crate_path.replace("/", "-")

  # Extract crate name from path and convert underscores to hyphens
  crate_name = Path(crate_path).name
  return crate_name.replace("_", "-")


def find_documentation_files() -> List[Tuple[str, str, str]]:
  """Find all CLAUDE.md and PACKAGE.md files in the codebase.

  Returns:
      List of tuples (file_type, relative_path, crate_identifier)
      file_type: 'claude' or 'package'
      relative_path: path from repo root to the file
      crate_identifier: used for naming the symlink
  """
  repo_root = Path(__file__).parent.parent
  files = []

  # Find files in crates directory
  crates_dir = repo_root / "crates"
  if crates_dir.exists():
    for item in crates_dir.iterdir():
      if item.is_dir():
        crate_name = item.name

        # Main crate files
        claude_file = item / "CLAUDE.md"
        if claude_file.exists():
          files.append(("claude", str(claude_file.relative_to(repo_root)), crate_name))

        package_file = item / "PACKAGE.md"
        if package_file.exists():
          files.append(("package", str(package_file.relative_to(repo_root)), crate_name))

        # Test utils files
        test_utils_dir = item / "src" / "test_utils"
        if test_utils_dir.exists():
          test_claude = test_utils_dir / "CLAUDE.md"
          if test_claude.exists():
            files.append(("claude", str(test_claude.relative_to(repo_root)), f"{crate_name}/test_utils"))

          test_package = test_utils_dir / "PACKAGE.md"
          if test_package.exists():
            files.append(("package", str(test_package.relative_to(repo_root)), f"{crate_name}/test_utils"))

        # Special handling for bodhi subdirectories
        if crate_name == "bodhi":
          for subdir in ["src", "src-tauri"]:
            subdir_path = item / subdir
            if subdir_path.exists():
              sub_claude = subdir_path / "CLAUDE.md"
              if sub_claude.exists():
                files.append(("claude", str(sub_claude.relative_to(repo_root)), f"bodhi/{subdir}"))

              sub_package = subdir_path / "PACKAGE.md"
              if sub_package.exists():
                files.append(("package", str(sub_package.relative_to(repo_root)), f"bodhi/{subdir}"))

  # Find files in xtask directory
  xtask_dir = repo_root / "xtask"
  if xtask_dir.exists():
    xtask_claude = xtask_dir / "CLAUDE.md"
    if xtask_claude.exists():
      files.append(("claude", str(xtask_claude.relative_to(repo_root)), "xtask"))

    xtask_package = xtask_dir / "PACKAGE.md"
    if xtask_package.exists():
      files.append(("package", str(xtask_package.relative_to(repo_root)), "xtask"))

  # Find files in devops directory
  devops_dir = repo_root / "devops"
  if devops_dir.exists():
    devops_claude = devops_dir / "CLAUDE.md"
    if devops_claude.exists():
      files.append(("claude", str(devops_claude.relative_to(repo_root)), "devops"))

    devops_package = devops_dir / "PACKAGE.md"
    if devops_package.exists():
      files.append(("package", str(devops_package.relative_to(repo_root)), "devops"))

  # Find files in .github directory
  github_dir = repo_root / ".github"
  if github_dir.exists():
    github_claude = github_dir / "CLAUDE.md"
    if github_claude.exists():
      files.append(("claude", str(github_claude.relative_to(repo_root)), "github"))

    github_package = github_dir / "PACKAGE.md"
    if github_package.exists():
      files.append(("package", str(github_package.relative_to(repo_root)), "github"))

  # Find files in project root
  root_claude = repo_root / "CLAUDE.md"
  if root_claude.exists():
    files.append(("claude", "CLAUDE.md", "root"))

  root_package = repo_root / "PACKAGE.md"
  if root_package.exists():
    files.append(("package", "PACKAGE.md", "root"))

  return files


def generate_symlink_name(file_type: str, crate_identifier: str) -> str:
  """Generate the symlink name based on file type and crate identifier.

  Examples:
      ('claude', 'auth_middleware') -> 'auth-middleware.md'
      ('package', 'auth_middleware') -> 'auth-middleware-package.md'
      ('claude', 'objs/test_utils') -> 'objs-test-utils.md'
      ('package', 'objs/test_utils') -> 'objs-test-utils-package.md'
  """
  # Handle test_utils suffix by preserving full crate name
  if "/test_utils" in crate_identifier:
    parts = crate_identifier.split("/test_utils")
    crate_name = parts[0]
    normalized_name = normalize_crate_name(crate_name) + "-test-utils"
  else:
    normalized_name = normalize_crate_name(crate_identifier)

  if file_type == "claude":
    return f"{normalized_name}.md"
  elif file_type == "package":
    return f"{normalized_name}-package.md"
  else:
    raise ValueError(f"Unknown file type: {file_type}")


def is_symlink_correct(symlink_path: Path, target_path: str) -> bool:
  """Check if symlink exists and points to the correct target."""
  if not symlink_path.is_symlink():
    return False

  try:
    current_target = str(symlink_path.readlink())
    return current_target == target_path
  except (OSError, ValueError):
    return False


def create_or_update_symlink(symlink_path: Path, target_path: str, dry_run: bool = False) -> str:
  """Create or update a symlink to point to the target.

  Returns:
      Action description string
  """
  action = ""

  if symlink_path.exists() or symlink_path.is_symlink():
    if is_symlink_correct(symlink_path, target_path):
      return f"✓ {symlink_path.name} (already correct)"
    else:
      if not dry_run:
        symlink_path.unlink()
      action = f"↻ {symlink_path.name} (updated)"
  else:
    action = f"+ {symlink_path.name} (created)"

  if not dry_run:
    symlink_path.symlink_to(target_path)

  return action


def main():
  """Main function to update context symlinks."""
  import argparse

  parser = argparse.ArgumentParser(description="Update AI context symlinks")
  parser.add_argument("--dry-run", action="store_true", help="Show what would be done without making changes")
  parser.add_argument("--verbose", "-v", action="store_true", help="Show verbose output")

  args = parser.parse_args()

  repo_root = Path(__file__).parent.parent
  context_dir = repo_root / "ai-docs" / "context"

  # Ensure context directory exists
  if not context_dir.exists():
    if args.dry_run:
      print(f"Would create directory: {context_dir}")
    else:
      context_dir.mkdir(parents=True, exist_ok=True)
      print(f"Created directory: {context_dir}")

  # Find all documentation files
  doc_files = find_documentation_files()

  if args.verbose:
    print(f"Found {len(doc_files)} documentation files")

  actions = []

  for file_type, relative_path, crate_identifier in doc_files:
    symlink_name = generate_symlink_name(file_type, crate_identifier)
    symlink_path = context_dir / symlink_name

    # Calculate relative path from context directory to the file
    # ai-docs/context -> ../../relative_path
    target_path = f"../../{relative_path}"

    action = create_or_update_symlink(symlink_path, target_path, args.dry_run)
    actions.append(action)

    if args.verbose:
      print(f"{action} -> {target_path}")

  # Summary
  created = sum(1 for a in actions if a.startswith("+"))
  updated = sum(1 for a in actions if a.startswith("↻"))
  unchanged = sum(1 for a in actions if a.startswith("✓"))

  print(f"\nSymlink management {'(dry run)' if args.dry_run else 'completed'}:")
  print(f"  Created: {created}")
  print(f"  Updated: {updated}")
  print(f"  Unchanged: {unchanged}")
  print(f"  Total: {len(actions)}")

  if not args.verbose and (created > 0 or updated > 0):
    print("\nActions taken:")
    for action in actions:
      if not action.startswith("✓"):
        print(f"  {action}")


if __name__ == "__main__":
  main()
