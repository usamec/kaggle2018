#include <stdio.h>
#include <cassert>
#include <cmath>
#include <vector>
#include <iostream>
#include <algorithm>
using namespace std;

#define FOR(i, n) for (int i=0; i<(int) n; i++)

const int N = 197769;
char is_prime[N];
double x[N];
double y[N];
double penalty = 0.1;

int check_prime(int i) {
  if (i < 2) return false;
  int j = 2;
  while (j * j <= i) {
    if (i % j == 0) return false;
    j++;
  }
  return true;
}

void gen_primes() {
  cout << "gen_primes()" << endl;
  FOR(i, N) {
      is_prime[i] = check_prime(i);
  }
}

void read_cities() {
  cout << "read_cities" << endl;
  FILE* f = fopen("cities.csv", "r");
  assert(f);
  FOR(i, N) {
    int tmp;
    fscanf(f, "%d %lf %lf", &tmp, &x[i], &y[i]);
  }
  fclose(f);
}

vector<int> read_path(const char* fname) {
  cout << "read_path" << endl;
  vector<int> result;
  FILE* f = fopen(fname, "r");
  assert(f);
  char tmp[100];
  fscanf(f, "%s", tmp);
  FOR(i, N+1) {
    int x;
    fscanf(f, "%d", &x);
    result.push_back(x);
  }
  fclose(f);
  return result;
}

void write_path(const char* fname, vector<int>& path) {
  FILE* f = fopen(fname, "w");
  assert(f);
  assert(*path.begin() == 0);
  assert(*(path.end() - 1) == 0);
  fprintf(f, "Path\n");
  for (int x: path) {
    fprintf(f, "%d\n", x);
  }
  fclose(f);
}

double distance(int i, int j) {
  double xx = (x[i] - x[j]) * (x[i] - x[j]);
  double yy = (y[i] - y[j]) * (y[i] - y[j]);
  return sqrt(xx + yy);
}

// shift = index of first vertex
double eval(const vector<int>& path, int shift) {
  double total = 0;
  FOR(i, path.size() - 1) {
    double d = distance(path[i], path[i+1]);
    if ((i + 1 + shift) % 10 == 0
        && !is_prime[path[i]]) {
      d *= 1 + penalty;
    }
    total += d;
  }
  return total;
}

