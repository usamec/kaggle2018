#!/usr/bin/python3
import argparse


def is_prime(x):
    if x < 2:
        return False
    y = 2
    while y * y <= x:
        if x % y == 0:
            return False
        y += 1
    return True


def get_penalty(i, edge):
    start, end = edge
    return i % 10 == 0 and not is_prime(start)


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
    l = lambda t: (t[1], get_penalty(t[0]+1, t[1]))
    map1 = dict(map(l, enumerate(edges1)))
    map2 = dict(map(l, enumerate(edges2)))

    added = []
    removed = []
    reverse = []
    common = []
    added_penalty = []
    removed_penalty = []

    for i, e in enumerate(edges1):
        if e in set2:
            if map1[e] == map2[e]:
                common.append(e)
            elif map1[e]:
                removed_penalty.append(e)
            else:
                added_penalty.append(e)
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

    return (added, reverse, removed, common, added_penalty, removed_penalty)


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
    added, reverse, removed, common, added_penalty, removed_penalty = diff_paths(cities, p1, p2)
    with open(args.out, "w") as fout:
        print_edges = edge_printer(fout, cities)
        print_edges(added, "0x3e8410")
        print_edges(removed, "0xc44129")
        print_edges(reverse, "0xec9332")
        print_edges(common, "0xcccccc")
        print_edges(added_penalty, "0xCE93D8")
        print_edges(removed_penalty, "0x90CAF9")

