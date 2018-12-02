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


def read_cities(fname):
    cities = []
    with open(fname) as f:
        for line in f:
            cities.append(list(map(float, line.strip().split()[1:])))
    return cities


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("out")
    args = parser.parse_args()

    cities = read_cities("cities.tsv")
    with open(args.out, "w") as fout:
        for i, (x, y) in enumerate(cities):
            if is_prime(i):
                print(x, y, "0x3e8800", file=fout)
