import json
import math
import re
import sys
from pathlib import Path

FILENAME_PATTERN = re.compile(r"^[CFH]P\d+N\d+L\d+\.json$")


def compute_stats(values: list[float]) -> dict:
    n = len(values)
    mean = sum(values) / n
    variance = sum((x - mean) ** 2 for x in values) / n
    std = math.sqrt(variance)

    return {
        "mean": mean,
        "std": std,
        "min": min(values),
        "max": max(values),
        "n": n,
    }


def extract_leaf_paths(obj, prefix=()):
    """Recursively yield (key_tuple, value) for all numeric leaf nodes."""
    if isinstance(obj, dict):
        for k, v in obj.items():
            yield from extract_leaf_paths(v, prefix + (k,))
    elif isinstance(obj, (int, float)):
        yield prefix, obj


def aggregate_participant_runs(runs: list[dict]) -> dict:
    """
    Given a list of 5 run-dicts for one participant, compute per-leaf stats
    across runs. Returns a nested dict mirroring the original structure but
    with stat dicts at the leaves.
    """
    # Collect all values per leaf path
    leaf_values: dict[tuple, list[float]] = {}
    for run in runs:
        for path, value in extract_leaf_paths(run):
            leaf_values.setdefault(path, []).append(value)

    # Build result dict with the same nested structure
    result = {}
    for path, values in leaf_values.items():
        node = result
        for key in path[:-1]:
            node = node.setdefault(key, {})
        node[path[-1]] = compute_stats(values)

    return result


def process_file(input_path: Path, output_path: Path) -> None:
    with input_path.open("r") as f:
        data = json.load(f)

    output = {}
    for pid, runs in data.items():
        if not isinstance(runs, list) or not runs:
            continue
        output[pid] = aggregate_participant_runs(runs)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w") as f:
        json.dump(output, f, indent=2)


def process_folder(input_folder: str, output_folder: str, prefix: str = "") -> None:
    input_dir = Path(input_folder)
    output_dir = Path(output_folder)

    if not input_dir.is_dir():
        print(f"Error: '{input_folder}' is not a directory.", file=sys.stderr)
        sys.exit(1)

    matched = 0
    for file in sorted(input_dir.iterdir()):
        if not file.is_file():
            continue
        if not FILENAME_PATTERN.match(file.name):
            continue
        if prefix and not file.name.startswith(prefix):
            continue

        output_path = output_dir / file.name
        process_file(file, output_path)
        print(f"Processed: {file.name} -> {output_path}")
        matched += 1

    if matched == 0:
        print(f"No matching files found in '{input_folder}' with prefix '{prefix}'.")
    else:
        print(f"\nDone. {matched} file(s) processed.")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Aggregate statistics from privacy-preserving protocol experiment JSONs."
    )
    parser.add_argument(
        "input_folder", help="Folder containing raw experiment JSON files."
    )
    parser.add_argument(
        "output_folder", help="Folder to write aggregated result JSON files."
    )
    parser.add_argument(
        "--prefix",
        default="",
        help="Optional filename prefix filter (e.g. 'FP' or 'CP2').",
    )

    args = parser.parse_args()
    process_folder(args.input_folder, args.output_folder, args.prefix)
