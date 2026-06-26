import json
import os
import re
from collections import defaultdict

import matplotlib.pyplot as plt

# ============================================
# Configuration
# ============================================

INPUT_DIR = "./results"
OUTPUT_DIR = "./plots"

# Chain (C), Fully-connected (F) or Hybrid (H)
TARGET_PREFIX = "F"

FILENAME_PATTERN = re.compile(r"([CFH])P(\d+)N(\d+)L(\d+)\.json$")


# ============================================
# Helpers
# ============================================


def extract_parameters(filename):
    """
    Extract prefix, P, N, L from filename.
    Example: CP4N8L16.json
    """
    match = FILENAME_PATTERN.match(filename)
    if not match:
        return None

    prefix = match.group(1)
    P = int(match.group(2))
    N = int(match.group(3))
    L = int(match.group(4))

    return prefix, P, N, L


def aggregate_metric(data, metric):
    means = []
    stds = []

    for participant_data in data.values():
        if metric == "time":
            metric_data = participant_data.get("time", {})

            if "total" in metric_data:
                means.append(
                    metric_data["total"]["mean"] * 1e-9
                )  # Convert from ns to s
                stds.append(metric_data["total"]["std"] * 1e-9)  # Convert from ns to s
            else:
                m = 0
                s = 0

                for func, entry in metric_data.items():
                    if func == "total":
                        continue

                    m += entry["mean"] * 1e-9  # Convert from ns to s
                    s += entry["std"] * 1e-9  # Convert from ns to s

                means.append(m)
                stds.append(s)

        elif metric == "messages":
            means.append(participant_data["messages"]["mean"])
            stds.append(participant_data["messages"]["std"])

        else:
            m = (
                sum(
                    entry["mean"] for entry in participant_data.get(metric, {}).values()
                )
                / 1024  # Convert from bytes to KB
            )
            s = (
                sum(entry["std"] for entry in participant_data.get(metric, {}).values())
                / 1024  # Convert from bytes to KB
            )

            means.append(m)
            stds.append(s)

    return (sum(means) / len(means)), (sum(stds) / len(stds))


def aggregate_function_breakdown(data, metric):
    values = defaultdict(list)

    for participant_data in data.values():
        # ignore total for time
        for func, entry in participant_data.get(metric, {}).items():
            if metric == "time" and func == "total":
                continue

            if metric == "time":
                values[func].append(entry["mean"] * 1e-9)  # Convert from ns to s
            else:
                values[func].append(entry["mean"] / 1024)  # Convert from bytes to KB

    result = {}

    for func, values in values.items():
        result[func] = sum(values) / len(values)

    return {}


# ============================================
# Load experiment results
# ============================================

results = {
    "time": defaultdict(lambda: defaultdict(list)),
    "space": defaultdict(lambda: defaultdict(list)),
    "communication": defaultdict(lambda: defaultdict(list)),
    "messages": defaultdict(lambda: defaultdict(list)),
}

breakdown = {
    "time": defaultdict(lambda: defaultdict(list)),
    "space": defaultdict(lambda: defaultdict(list)),
    "communication": defaultdict(lambda: defaultdict(list)),
}


for filename in os.listdir(INPUT_DIR):
    params = extract_parameters(filename)
    if params is None:
        continue

    prefix, P, N, L = params

    if prefix != TARGET_PREFIX:
        continue

    filepath = os.path.join(INPUT_DIR, filename)

    with open(filepath, "r") as f:
        data = json.load(f)

    for metric in ["time", "space", "communication", "messages"]:
        mean, std = aggregate_metric(data, metric)
        results[metric][P][L].append((N, mean, std))

        if metric != "messages":
            breakdown_data = aggregate_function_breakdown(data, metric)
            breakdown[metric][P][L].append((N, breakdown_data))


# ============================================
# Plotting
# ============================================

os.makedirs(OUTPUT_DIR, exist_ok=True)


labels = {
    "time": "Time (s)",
    "space": "Space (KB)",
    "communication": "Communication (KB)",
    "messages": "# of messages",
}

file_labels = {
    "time": "T",
    "space": "S",
    "communication": "C",
    "messages": "M",
}


def create_plot(metric, P, data):
    for L in sorted(data.keys()):
        points = sorted(data[L], key=lambda x: x[0])

        x = [n for n, _, _ in points]
        y = [m for _, m, _ in points]
        std = [s for _, _, s in points]

        plt.plot(x, y, marker="x", label=rf"$\ell$ = {L}")

        # Show standard deviation as shaded area
        plt.fill_between(
            x,
            [m - s for m, s in zip(y, std)],
            [m + s for m, s in zip(y, std)],
            alpha=0.2,
        )

    plt.yscale("log")
    plt.xticks(fontweight="bold")
    plt.yticks(fontweight="bold")

    plt.xlabel("# of internal nodes")
    plt.ylabel(labels[metric])
    plt.legend(loc="lower right", prop={"weight": "bold"})
    plt.grid(True)

    output_path = os.path.join(
        OUTPUT_DIR, f"{TARGET_PREFIX}P{P}", f"{file_labels[metric]}.png"
    )

    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    plt.savefig(output_path)
    plt.close()


def create_stacked_plot(metric, P, L, data):
    # Sort by number of nodes
    data = sorted(data, key=lambda x: x[0])

    x = [entry[0] for entry in data]

    # Collect all function names
    functions = set()
    for _, breakdown in data:
        functions.update([k for k in breakdown.keys()])
    functions = sorted(functions)

    # Collect y-values for each function
    y = []
    for func in functions:
        values = []

        for _, breakdown in data:
            values.append(breakdown.get(func, 0))

        y.append(values)

    plt.stackplot(x, *y, labels=functions)

    plt.xticks(fontweight="bold")
    plt.yticks(fontweight="bold")
    plt.margins(x=0)

    plt.xlabel("# of internal nodes")
    plt.ylabel(labels[metric])
    plt.legend(loc="upper left", prop={"weight": "bold"})

    output_path = os.path.join(
        OUTPUT_DIR,
        f"{TARGET_PREFIX}P{P}",
        f"L{L}-{metric}-S.png",
    )
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    plt.savefig(output_path, bbox_inches="tight")
    plt.close()


for metric in ["time", "space", "communication", "messages"]:
    for P, L in results[metric].items():
        create_plot(metric, P, L)

    if metric != "messages":
        for P in breakdown[metric]:
            # Create one plot per L-value
            for L in breakdown[metric][P]:
                create_stacked_plot(
                    metric,
                    P,
                    L,
                    breakdown[metric][P][L],
                )


print("Done.")
