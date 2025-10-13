#!/usr/bin/env python3
"""
Docker Build Timing Analysis Script

Parses GitHub Actions Docker build logs and extracts timing information
for multi-platform builds (linux/amd64 and linux/arm64).
"""

import re
from datetime import datetime
from collections import defaultdict
from typing import Dict, List, Tuple, Optional


class BuildStep:
    def __init__(self, step_id: str, platform: str, stage: str, step_num: str, description: str):
        self.step_id = step_id
        self.platform = platform
        self.stage = stage
        self.step_num = step_num
        self.description = description
        self.start_time: Optional[datetime] = None
        self.end_time: Optional[datetime] = None
        self.duration: Optional[float] = None

    def __repr__(self):
        return f"BuildStep({self.step_id}, {self.platform}, {self.stage} {self.step_num}, {self.duration}s)"


def parse_timestamp(line: str) -> Optional[datetime]:
    """Extract timestamp from log line."""
    match = re.match(r'^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)', line)
    if match:
        # Parse ISO format timestamp
        ts_str = match.group(1)
        return datetime.fromisoformat(ts_str.replace('Z', '+00:00'))
    return None


def parse_build_step(line: str) -> Optional[Tuple[str, str, str, str, str]]:
    """Parse a Docker build step line.

    Returns: (step_id, platform, stage, step_num, description)
    """
    # Pattern: #<id> [linux/<arch> <stage> <step>/<total>] <description>
    pattern = r'#(\d+) \[linux/(amd64|arm64) ([\w-]+)\s+(\d+/\d+)\] (.+)'
    match = re.search(pattern, line)
    if match:
        return match.groups()
    return None


def parse_done_line(line: str) -> Optional[Tuple[str, float]]:
    """Parse a DONE line with step ID and duration.

    Returns: (step_id, duration_seconds)
    """
    # Pattern: #<id> DONE <time>s
    pattern = r'#(\d+) DONE (\d+\.\d+)s'
    match = re.search(pattern, line)
    if match:
        return match.group(1), float(match.group(2))
    return None


def analyze_log_file(log_path: str) -> Dict[str, List[BuildStep]]:
    """Analyze the log file and extract build steps."""
    steps_by_platform = defaultdict(list)
    steps_by_id = {}
    start_times = {}

    with open(log_path, 'r') as f:
        for line in f:
            timestamp = parse_timestamp(line)

            # Check for build step start
            step_info = parse_build_step(line)
            if step_info:
                step_id, platform, stage, step_num, description = step_info

                # Create or update step
                if step_id not in steps_by_id:
                    step = BuildStep(step_id, platform, stage, step_num, description)
                    steps_by_id[step_id] = step
                    steps_by_platform[platform].append(step)

                # Record start time if we have a timestamp
                if timestamp and step_id not in start_times:
                    start_times[step_id] = timestamp
                    steps_by_id[step_id].start_time = timestamp

            # Check for DONE line
            done_info = parse_done_line(line)
            if done_info:
                step_id, duration = done_info
                if step_id in steps_by_id:
                    steps_by_id[step_id].duration = duration
                    if timestamp:
                        steps_by_id[step_id].end_time = timestamp

    return dict(steps_by_platform)


def format_duration(seconds: float) -> str:
    """Format duration in human-readable format."""
    if seconds < 60:
        return f"{seconds:.1f}s"
    elif seconds < 3600:
        mins = int(seconds // 60)
        secs = seconds % 60
        return f"{mins}m {secs:.1f}s"
    else:
        hours = int(seconds // 3600)
        mins = int((seconds % 3600) // 60)
        secs = seconds % 60
        return f"{hours}h {mins}m {secs:.1f}s"


def calculate_cumulative_time(steps: List[BuildStep]) -> List[Tuple[BuildStep, float]]:
    """Calculate cumulative time for each step based on end times."""
    # Sort steps by end time
    sorted_steps = sorted([s for s in steps if s.end_time], key=lambda x: x.end_time)

    if not sorted_steps:
        return []

    start_time = min(s.start_time for s in steps if s.start_time)
    result = []

    for step in sorted_steps:
        if step.end_time and start_time:
            cumulative = (step.end_time - start_time).total_seconds()
            result.append((step, cumulative))

    return result


def print_platform_report(platform: str, steps: List[BuildStep]):
    """Print detailed report for a single platform."""
    print(f"\n{'='*120}")
    print(f"Platform: {platform.upper()}")
    print(f"{'='*120}")

    # Calculate cumulative times
    steps_with_cumulative = calculate_cumulative_time(steps)

    if not steps_with_cumulative:
        print("No timing data available")
        return

    # Print header
    print(f"{'Step':<6} {'Stage':<15} {'Step #':<8} {'Duration':<15} {'Cumulative':<15} {'Description':<50}")
    print(f"{'-'*6} {'-'*15} {'-'*8} {'-'*15} {'-'*15} {'-'*50}")

    # Print each step
    for step, cumulative in steps_with_cumulative:
        duration_str = format_duration(step.duration) if step.duration else "N/A"
        cumulative_str = format_duration(cumulative)
        desc = step.description[:47] + "..." if len(step.description) > 50 else step.description

        print(f"#{step.step_id:<5} {step.stage:<15} {step.step_num:<8} {duration_str:<15} {cumulative_str:<15} {desc:<50}")

    # Print summary
    total_time = steps_with_cumulative[-1][1]
    total_step_time = sum(s.duration for s in steps if s.duration)

    print(f"\n{'Summary':-^120}")
    print(f"Total wall-clock time: {format_duration(total_time)}")
    print(f"Total step time: {format_duration(total_step_time)}")
    print(f"Number of steps: {len(steps_with_cumulative)}")


def print_slowest_steps(steps_by_platform: Dict[str, List[BuildStep]], top_n: int = 10):
    """Print the slowest build steps across all platforms."""
    all_steps = []
    for platform, steps in steps_by_platform.items():
        for step in steps:
            if step.duration:
                all_steps.append((platform, step))

    # Sort by duration
    all_steps.sort(key=lambda x: x[1].duration, reverse=True)

    print(f"\n{'='*120}")
    print(f"TOP {top_n} SLOWEST STEPS (All Platforms)")
    print(f"{'='*120}")
    print(f"{'Platform':<12} {'Step':<6} {'Stage':<15} {'Duration':<15} {'Description':<50}")
    print(f"{'-'*12} {'-'*6} {'-'*15} {'-'*15} {'-'*50}")

    for platform, step in all_steps[:top_n]:
        duration_str = format_duration(step.duration)
        desc = step.description[:47] + "..." if len(step.description) > 50 else step.description
        print(f"{platform:<12} #{step.step_id:<5} {step.stage:<15} {duration_str:<15} {desc:<50}")


def print_optimization_recommendations(steps_by_platform: Dict[str, List[BuildStep]]):
    """Analyze and print optimization recommendations."""
    print(f"\n{'='*120}")
    print("OPTIMIZATION RECOMMENDATIONS")
    print(f"{'='*120}")

    recommendations = []

    # Find longest steps per platform
    for platform, steps in steps_by_platform.items():
        sorted_steps = sorted([s for s in steps if s.duration], key=lambda x: x.duration, reverse=True)

        for step in sorted_steps[:5]:
            if step.duration > 60:  # Steps taking more than 1 minute
                if "cargo build" in step.description:
                    recommendations.append(
                        f"• [{platform}] Cargo build taking {format_duration(step.duration)} - "
                        f"Consider using sccache or cargo-chef for better caching"
                    )
                elif "npm" in step.description:
                    recommendations.append(
                        f"• [{platform}] NPM build taking {format_duration(step.duration)} - "
                        f"Consider using npm ci with better layer caching"
                    )
                elif "apt-get" in step.description:
                    recommendations.append(
                        f"• [{platform}] Package installation taking {format_duration(step.duration)} - "
                        f"Consider using a pre-built base image with dependencies"
                    )

    # Check for parallel execution opportunities
    amd64_steps = steps_by_platform.get('amd64', [])
    arm64_steps = steps_by_platform.get('arm64', [])

    if amd64_steps and arm64_steps:
        amd64_total = sum(s.duration for s in amd64_steps if s.duration)
        arm64_total = sum(s.duration for s in arm64_steps if s.duration)

        print(f"\nPlatform-specific observations:")
        print(f"  • AMD64 total step time: {format_duration(amd64_total)}")
        print(f"  • ARM64 total step time: {format_duration(arm64_total)}")

        if abs(amd64_total - arm64_total) > 300:  # 5 minutes difference
            slower_platform = "ARM64" if arm64_total > amd64_total else "AMD64"
            recommendations.append(
                f"• Significant time difference between platforms - {slower_platform} is much slower. "
                f"Consider optimizing platform-specific builds or using native builders."
            )

    if recommendations:
        print("\nKey recommendations:")
        for rec in recommendations[:10]:  # Show top 10 recommendations
            print(rec)
    else:
        print("\nNo specific recommendations - build appears reasonably optimized.")


def main():
    log_path = "git-log.log"

    print("Analyzing Docker build logs...")
    print(f"Log file: {log_path}")

    steps_by_platform = analyze_log_file(log_path)

    if not steps_by_platform:
        print("No build steps found in log file")
        return

    # Print reports for each platform
    for platform in sorted(steps_by_platform.keys()):
        print_platform_report(platform, steps_by_platform[platform])

    # Print slowest steps across all platforms
    print_slowest_steps(steps_by_platform, top_n=15)

    # Print optimization recommendations
    print_optimization_recommendations(steps_by_platform)

    print(f"\n{'='*120}")


if __name__ == "__main__":
    main()
