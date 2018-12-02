#!/usr/bin/python3
import argparse


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


def rev(edge):
    x, y = edge
    return (y, x)


def diff_paths(cities, path1, path2):
    edges1 = to_edges(path1)
    edges2 = to_edges(path2)
    set1 = set(edges1)
    set2 = set(edges2)

    added = []
    removed = []
    reverse = []
    common = []

    for e in edges1:
        if e in set2:
            common.append(e)
        elif rev(e) in set2:
            reverse.append(e)
        else:
            removed.append(e)

    for e in edges2:
        if e in set1:
            pass
        elif rev(e) in set1:
            pass
        else:
            added.append(e)

    return (added, reverse, removed, common)


def edge_printer(fout, cities):
    def print_edges(edges, color):
        for start, end in edges:
            print(
                cities[start][0],
                cities[start][1],
                cities[end][0] - cities[start][0],
                cities[end][1] - cities[start][1],
                color,
                file=fout,
            )
    return print_edges


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("file1")
    parser.add_argument("file2")
    parser.add_argument("out")
    args = parser.parse_args()

    cities = read_cities("cities.tsv")
    p1 = read_solution(args.file1)
    p2 = read_solution(args.file2)
    diff_paths(cities, p1, p2)
    added, reverse, removed, common = diff_paths(cities, p1, p2)
    with open(args.out, "w") as fout:
        print_edges = edge_printer(fout, cities)
        print_edges(added, "0x3e8410")
        print_edges(removed, "0xc44129")
        print_edges(reverse, "0xec9332")
        print_edges(common, "0xcccccc")
