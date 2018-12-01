#!/usr/bin/python3
import argparse
import colorsys


def read_cities(fname):
    cities = []
    with open(fname) as f:
        for line in f:
            cities.append(list(map(float, line.strip().split()[1:])))
    return cities


def read_solution(fname):
    path = []
    with open(fname) as f:
        f.readline()
        for line in f:
            path.append(int(line.strip()))
    return path


def to_edges(path):
    return list(zip(path[:-1], path[1:]))


def edge_printer(fout, cities):
    def print_edge(edge, color):
        start, end = edge
        print(
            cities[start][0],
            cities[start][1],
            cities[end][0] - cities[start][0],
            cities[end][1] - cities[start][1],
            color,
            file=fout,
        )
    return print_edge


def get_color(progress):
    r, g, b = colorsys.hls_to_rgb(progress, 0.5, 0.5)
    r = int(r * 255)
    g = int(g * 255)
    b = int(b * 255)
    return "0x%x%x%x" % (r, g, b)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("file")
    parser.add_argument("out")
    args = parser.parse_args()

    cities = read_cities("cities.tsv")
    path = read_solution(args.file)
    with open(args.out, "w") as fout:
        print_edge = edge_printer(fout, cities)
        edges = to_edges(path)
        for i, edge in enumerate(edges):
            color = get_color(i / 197770.0)
            print_edge(edge, color)
