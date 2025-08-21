#!/usr/bin/env python3
"""
Script to get the latest Docker image version from GitHub Container Registry (GHCR).

This script replaces the complex shell function get_ghcr_docker_version from the Makefile
with a more maintainable and readable Python implementation.
"""

import argparse
import json
import re
import subprocess
import sys
from typing import List, Optional


def run_gh_api_command(repo_owner: str, package_name: str) -> Optional[dict]:
  """Run gh api command to get package versions from GHCR."""
  try:
    cmd = ["gh", "api", f"/orgs/{repo_owner}/packages/container/{package_name}/versions"]
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)
  except subprocess.CalledProcessError:
    return None
  except json.JSONDecodeError:
    return None


def extract_versions_from_tags(tags: List[str], variant: str) -> List[str]:
  """Extract version numbers from container tags based on variant (production or development)."""
  versions = []

  for tag in tags:
    if variant == "production":
      # Match production tags: X.Y.Z or X.Y.Z-{cpu|cuda|rocm|vulkan}
      if re.match(r"^\d+\.\d+\.\d+$", tag) or re.match(r"^\d+\.\d+\.\d+-(cpu|cuda|rocm|vulkan)$", tag):
        # Extract version part (remove variant suffix if present)
        version = tag.split("-")[0]
        versions.append(version)
    else:  # development
      # Match development tags: X.Y.Z-development or X.Y.Z-{cpu|cuda|rocm|vulkan}-development
      if re.match(r"^\d+\.\d+\.\d+-development$", tag) or re.match(
        r"^\d+\.\d+\.\d+-(cpu|cuda|rocm|vulkan)-development$", tag
      ):
        # Extract version part (remove -development and variant suffix)
        if tag.endswith("-development"):
          version = tag.replace("-development", "").split("-")[0]
        else:
          # Handle X.Y.Z-cpu-development format
          parts = tag.split("-")
          if len(parts) >= 3 and parts[-1] == "development":
            version = parts[0]
          else:
            continue
        versions.append(version)

  return versions


def sort_versions(versions: List[str]) -> List[str]:
  """Sort versions in semantic version order."""

  def version_key(v: str) -> tuple:
    return tuple(int(x) for x in v.split("."))

  return sorted(set(versions), key=version_key)


def get_ghcr_docker_version(variant: str, debug: bool = False) -> str:
  """
  Get the latest Docker image version from GHCR for the specified variant.

  Args:
      variant: Either "production" or "development"
      debug: If True, print debug information

  Returns:
      Latest version string or "0.0.0" if no versions found
  """
  repo_owner = "BodhiSearch"
  package_name = "bodhiapp"

  if debug:
    print(f"DEBUG: Fetching versions for {repo_owner}/{package_name} (variant: {variant})", file=sys.stderr)

  # Get package versions from GHCR
  response = run_gh_api_command(repo_owner, package_name)
  if not response:
    if debug:
      print("DEBUG: No response from GHCR API", file=sys.stderr)
    return "0.0.0"

  if debug:
    print(f"DEBUG: Found {len(response)} version entries from GHCR", file=sys.stderr)

  # Extract all tags from all versions
  all_tags = []
  for i, version in enumerate(response):
    if debug:
      print(f"DEBUG: Version {i + 1}: {json.dumps(version.get('metadata', {}), indent=2)}", file=sys.stderr)

    if "metadata" in version and "container" in version["metadata"] and "tags" in version["metadata"]["container"]:
      tags = version["metadata"]["container"]["tags"]
      if tags:  # tags can be None
        all_tags.extend(tags)
        if debug:
          print(f"DEBUG: Version {i + 1} tags: {tags}", file=sys.stderr)
      elif debug:
        print(f"DEBUG: Version {i + 1} has no tags", file=sys.stderr)
    elif debug:
      print(f"DEBUG: Version {i + 1} missing container metadata", file=sys.stderr)

  if debug:
    print(f"DEBUG: All tags found: {sorted(all_tags)}", file=sys.stderr)

  if not all_tags:
    if debug:
      print("DEBUG: No tags found", file=sys.stderr)
    return "0.0.0"

  # Extract versions based on variant
  versions = extract_versions_from_tags(all_tags, variant)

  if debug:
    print(f"DEBUG: Filtered versions for {variant}: {versions}", file=sys.stderr)

  if not versions:
    if debug:
      print(f"DEBUG: No versions found for variant {variant}", file=sys.stderr)
    return "0.0.0"

  # Sort and return the latest version
  sorted_versions = sort_versions(versions)
  if debug:
    print(f"DEBUG: Sorted versions: {sorted_versions}", file=sys.stderr)

  latest = sorted_versions[-1] if sorted_versions else "0.0.0"
  if debug:
    print(f"DEBUG: Latest version: {latest}", file=sys.stderr)

  return latest


def main():
  """Main function to handle command line arguments."""
  parser = argparse.ArgumentParser(
    description="Get latest Docker image version from GHCR",
    formatter_class=argparse.RawDescriptionHelpFormatter,
    epilog="""
Examples:
  get_ghcr_version.py production
  get_ghcr_version.py development --debug
        """,
  )
  parser.add_argument(
    "variant", choices=["production", "development"], help="Variant to check (production or development)"
  )
  parser.add_argument("--debug", "-d", action="store_true", help="Print debug information to stderr")

  args = parser.parse_args()

  version = get_ghcr_docker_version(args.variant, args.debug)
  print(version)


if __name__ == "__main__":
  main()
