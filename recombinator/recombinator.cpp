#include <cmath>
#include <vector>
#include <iostream>
#include <algorithm>

#include "utils.cpp"
using namespace std;

vector<int> path;


bool is_interesting_subtour(
  const vector<int>& index,
  const vector<int>& path1,
  const vector<int>& path2,
  int start2,
  int end2
) {
  int start1 = index[path2[start2]];
  int end1 = index[path2[end2]];
  assert(path1[start1] == path2[start2]);
  assert(path1[end1] == path2[end2]);

  assert(start2 < end2);
  if (start1 < end1) return false; // reversed?

  for (int pos2 = start2; pos2 <= end2; pos2++) {
    int pos1 = index[path2[pos2]];
    if (pos1 < start1 || pos1 > end1) return false; // not same set of vertices
  }
  for (int pos2 = start2; pos2 <= end2; pos2++) {
    int pos1 = index[path2[pos2]];
    if (pos2 - start2 != pos1 - start1) return true; // different order
  }
  return false; // same subtour, no point in trying out
}


vector<int> make_index(const vector<int>& path) {
  int N = path.size();
  assert(path[0] == 0);
  assert(path[N-1] == 0);
  vector<int> index(200000, -1);
  FOR(i, N-1) index[path[i]] = i;
  return index;
}

vector<int> map_index(const vector<int>& index, const vector<int>& path) {
  vector<int> result;
  for (auto& x: path) {
    assert(x >= 0);
    assert(x < (int) index.size());
    assert(index[x] >= 0);
    result.push_back(index[x]);
  }
  return result;
}

vector<int> trim(const vector<int>& skip, const vector<int>& path) {
  vector<int> result;
  for (auto& x: path) {
    if (!skip[x]) {
      result.push_back(x);
      //printf("      -> %d\n", x);
    } else {
      //printf("skipping %d\n", x);
    }
  }
  //printf("trim done\n");
  return result;
}

vector<int> untrim(const vector<int>& skip, const vector<int>& path) {
  vector<int> result;
  int NN = path.size();
  assert(path[0] == 0);
  assert(path[NN-1] == 0);
  result.push_back(0);
  for (int i = 0; i < NN - 1; i++) {
    int x = path[i];
    int y = path[i + 1];
    if (x < y) {
      int all = 1;
      for (int j = x + 1; all && (j < y); j++) all &= skip[j];
      if (all && x + 1 < y) {
        //printf("unskip %d %d\n", x, y);
        for (int j = x + 1; j < y; j++) result.push_back(j);
      }
    } else {
      int all = 1;
      for (int j = x - 1; all && (j > y); j--) all &= skip[j];
      if (all && x > y + 1) {
        //printf("unskip %d %d\n", x, y);
        for (int j = x - 1; j > y; j--) result.push_back(j);
      }
    }
    result.push_back(y);
    //printf("%d\n", y);
  }
  return result;
}

// Assumes p1, p2 are normalized to index
vector<int> make_skip(const vector<int>& p2) {
  int N = p2.size();
  vector<int> skip(N);

  for(int i = 1; i + 1 < N; i++) {
    if (p2[i-1] == p2[i] - 1 && p2[i + 1] == p2[i] + 1) skip[p2[i]] = 1;
    if (p2[i-1] == p2[i] + 1 && p2[i + 1] == p2[i] - 1) skip[p2[i]] = 1;
  }
  return skip;
}


bool is_interesting(const vector<int>& path, int start, int end) {
    int N = path.size();
    assert(0 < start);
    assert(start < end);
    assert(end < N - 1);
    int v_start = path[start];
    int v_end = path[end];
    if (v_end - v_start != end - start) {
      return false;
    }
    for (int i = start; i <= end; i++) {
      if (path[i] < v_start || path[i] > v_end) return false;
    }

    int all_same = 1;
    for (int i = start; i <= end; i++) {
      if (path[i] != v_start + i - start) all_same = 0;
    }

    return !all_same;
}

vector<int> reconstruct(
    const vector<int>& path1,
    const vector<int>& skip,
    const vector<int>& t1,
    const vector<int>& res) {
  auto step1 = map_index(t1, res);
  assert(step1.size() == t1.size());
  auto step2 = untrim(skip, step1);
  assert(step2.size() == path1.size());
  auto step3 = map_index(path1, step2);
  assert(step3.size() == path1.size());
  return step3;
}

// recombine from path1 into path2
vector<int> recombine(
  const vector<int>& path1,
  const vector<int>& path2
) {
  if (path1 == path2) {
    printf("paths are same\n");
    return path1;
  }
  const int N = path1.size();
  { // checks
    assert(path1.size() == path2.size());
    assert(path1[0] == 0);
    assert(path2[0] == 0);
    assert(path1[N-1] == 0);
    assert(path2[N-1] == 0);
  }

  auto index = make_index(path1);

  // p1 and p2 are normalized
  auto p1 = map_index(index, path1);
  auto p2 = map_index(index, path2);

  { // checks
    printf("checks 1\n");
    auto tmp1 = map_index(path1, p1);
    auto tmp2 = map_index(path1, p2);
    assert(tmp1 == path1);
    assert(tmp2 == path2);
    printf("done\n");
  }

  auto skip = make_skip(p2);

  // trim useless vertices from p1 and p2
  auto t1 = trim(skip, p1);
  auto t2 = trim(skip, p2);

  { // checks
    printf("checks 2\n");
    assert(t1.size() == t2.size());
    auto tmp1 = untrim(skip, t1);
    assert(tmp1.size() == p1.size());
    assert(tmp1 == p1);
    auto tmp2 = untrim(skip, t2);
    //printf("%d %d\n", tmp1.size(), tmp2.size());
    assert(tmp2 == p2);
    printf("done\n");
  }

  // renormalize
  auto idx = make_index(t1);
  auto pp2 = map_index(idx, t2);

  { // checks
    printf("checks 3\n");
    auto pp1 = map_index(idx, t1);
    auto tmp1 = map_index(t1, pp1);
    auto tmp2 = map_index(t1, pp2);
    assert(tmp1 == t1);
    assert(tmp2 == t2);
    printf("done\n");
  }

  auto result(pp2);

  int NN = pp2.size();
  printf("NN: %d\n", NN);

  for (int start2 = 1; start2 < NN; start2++) {
    for (int end2 = start2 + 2; end2 < NN - 1; end2++) {
        if (!is_interesting(pp2, start2, end2)) continue;
        //printf("is interesting: %d %d\n", start2, end2);
        //for (int i = start2; i <= end2; i++) printf(" %d\n", pp2[i]);
        //printf("\n");
        int has_conflict = 0;

        for (int i = start2; i <= end2; i++) if (result[i] != pp2[i]) has_conflict = 1;
        if (has_conflict) continue;
        //printf("no conflict, could continue\n");
        //printf("merge\n");
        // Copy segment from pp1
        auto tmp = result;
        for (int i = start2; i <= end2; i++) tmp[i] = pp2[start2] + i - start2;
    
        auto full_tmp = reconstruct(path1, skip, t1, tmp);
        auto full_res = reconstruct(path1, skip, t1, result);

        double best = eval(full_res, 0);
        double cur = eval(full_tmp, 0);
        if (cur < best) {
          printf("have improvement: %.2lf\n", cur - best);
          result = tmp;
        } else {
          printf("%.2lf %.2lf\n", cur, cur - best);
        }
    }
  }
  return reconstruct(path1, skip, t1, result);
}

int main(int argc, char* argv[]) {
  if (argc != 4) {
    assert(argc >= 1);
    printf("Usage: %s path1.csv path2.csv out.csv\n", argv[0]);
    return 1;
  }

  gen_primes();
  read_cities();
  auto path1 = read_path(argv[1]);
  auto path2 = read_path(argv[2]);
  printf("%.1lf\n", eval(path1, 0));
  printf("%.1lf\n", eval(path2, 0));
  auto recomb = recombine(path1, path2);
  write_path(argv[3], recomb);
}
