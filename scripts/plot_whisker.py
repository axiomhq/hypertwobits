#!/usr/bin/env python

"""This program shows `hyperfine` benchmark results as a box and whisker plot.

Quoting from the matplotlib documentation:
    The box extends from the lower to upper quartile values of the data, with
    a line at the median. The whiskers extend from the box to show the range
    of the data. Flier points are those past the end of the whiskers.
"""

import argparse
import json
import matplotlib.pyplot as plt

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot Title")
parser.add_argument("--sort-by", choices=['median'], help="Sort method")
parser.add_argument(
    "--labels", help="Comma-separated list of entries for the plot legend"
)
parser.add_argument(
    "-o", "--output", help="Save image to the given filename."
)

args = parser.parse_args()

with open(args.file, encoding='utf-8') as f:
    results = json.load(f)["results"]

if args.labels:
    labels = args.labels.split(",")
else:
    labels = [b["algorithm"] for b in results]
counts = [b["counts"] for b in results]

if args.sort_by == 'median':
    medians = [b["median"] for b in results]
    indices = sorted(range(len(labels)), key=lambda k: medians[k])
    labels = [labels[i] for i in indices]
    counts = [counts[i] for i in indices]

plt.figure(figsize=(24, 24), constrained_layout=True)
boxplot = plt.boxplot(counts, vert=True, patch_artist=True)
cmap = plt.get_cmap("rainbow")
colors = [cmap(val / len(counts)) for val in range(len(counts))]

for patch, color in zip(boxplot["boxes"], colors):
    patch.set_facecolor(color)

if args.title:
    plt.title(args.title)
plt.legend(handles=boxplot["boxes"], labels=labels, loc="best", fontsize="medium")
plt.ylabel("Count [elements]")
# plt.ylim(0, None)
plt.xticks(list(range(1, len(labels)+1)), labels, rotation=45)
if args.output:
    plt.savefig(args.output)
else:
    plt.show()
