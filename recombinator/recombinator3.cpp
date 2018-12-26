#include <cmath>
#include <vector>
#include <iostream>
#include <algorithm>
#include <set>

#include "utils2.cpp"
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

vector<int> offset(const vector<int>& skip, const vector<int>& path) {
  vector<int> result;
  for (int i = 0; i < path.size(); i++) {
    auto x = path[i];
    if (!skip[x]) {
      result.push_back(i);
      //printf("      -> %d\n", x);
    } else {
      //printf("skipping %d\n", x);
    }
  }
  //printf("trim done\n");
  return result;
}

vector<int> untrim_part(const vector<int>& skip, const vector<int>& path) {
  vector<int> result;
  int NN = path.size();
/*  assert(path[0] == 0);
  assert(path[NN-1] == 0);*/
  result.push_back(path[0]);
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

vector<int> reconstruct_part(
    const vector<int>& path1,
    const vector<int>& skip,
    const vector<int>& t1,
    const vector<int>& res) {
  auto step1 = map_index(t1, res);
  //assert(step1.size() == t1.size());
  auto step2 = untrim_part(skip, step1);
//  assert(step2.size() == path1.size());
  auto step3 = map_index(path1, step2);
//  assert(step3.size() == path1.size());
  return step3;
}

vector<int> offset_part(
    const vector<int>& path1,
    const vector<int>& skip,
    const vector<int>& t1,
    const vector<int>& res) {
  auto step1 = map_index(t1, res);
  //assert(step1.size() == t1.size());
  auto step2 = untrim_part(skip, step1);
//  assert(step2.size() == path1.size());
  return offset(skip, step2); 
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

  auto o1 = offset(skip, p1);
  auto o2 = offset(skip, p2);

  { // checks
    printf("checks 2\n");
    assert(t1.size() == t2.size());
    auto tmp1 = untrim(skip, t1);
    assert(tmp1.size() == p1.size());
    assert(tmp1 == p1);
    auto tmp2 = untrim(skip, t2);
    //printf("%d %d\n", tmp1.size(), tmp2.size());
    assert(tmp2 == p2);

/*    auto tmp3 = untrim_part(skip, vector<int>(t2.begin() + 10, t2.begin()+30));
    printf("%d\n", tmp3.size());
    assert(tmp3 == vector<int>(p2.begin() + o2[10], p2.begin() + o2[10] + tmp3.size()));*/

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

  vector<vector<int>> dyn(NN, result);
  vector<pair<int, int>> interesting;

  for (int end2 = 2; end2 < NN - 1; end2++) {
    for (int start2 = 1; start2 <= end2 - 2; start2++) {
        if (!is_interesting(pp2, start2, end2)) continue;
	interesting.push_back(make_pair(start2, end2));
    }
  }
  printf("interesting size %d\n", interesting.size());

  set<pair<int, int>> inter_set(interesting.begin(), interesting.end());

/*  for (int end2 = 2; end2 < NN - 1; end2++) {
    dyn[end2] = dyn[end2-1];
    for (int start2 = 1; start2 <= end2 - 2; start2++) {
        if (!is_interesting(pp2, start2, end2)) continue;*/
  int last_end = 0;
  for (int i = 0; i < interesting.size(); i++) {
        bool bad = false;
        for (int j = i+1; j < interesting.size() && interesting[j].second == interesting[i].second;
		j++) {
	    if (inter_set.count(make_pair(interesting[i].first, interesting[j].first))) {
                bad = true;
		break;
	    }
	}
	if (bad) {
	    continue;
	}
        int end2 = interesting[i].second;
	int start2 = interesting[i].first;
//	printf("%d %d\n", start2, end2);
	if (end2 != last_end) {
	    for (int ee = last_end + 1; ee <= end2; ee++) 
	        dyn[ee] = dyn[ee-1];
	}
	last_end = end2;
        //printf("is interesting: %d %d\n", start2, end2);
        //for (int i = start2; i <= end2; i++) printf(" %d\n", pp2[i]);
        //printf("\n");
/*        int has_conflict = 0;

        for (int i = start2; i <= end2; i++) if (result[i] != pp2[i]) has_conflict = 1;*/
        //if (has_conflict) printf("skip due to conflict\n");
//        if (has_conflict) continue;
        //printf("no conflict, could continue\n");
        //printf("merge\n");
        // Copy segment from pp1
        //auto tmp = dyn[start2-1];
	vector<int> tmp;
	vector<int> orig(pp2.begin() + start2, pp2.begin() + end2 + 1);
        for (int i = start2; i <= end2; i++) tmp.push_back(pp2[start2] + i - start2);
	assert(tmp.size() == orig.size());

	auto full_orig = reconstruct_part(path1, skip, t1, orig);
   	auto full_tmp = reconstruct_part(path1, skip, t1, tmp);
	auto offset_tmp = offset_part(path1, skip, t1, tmp);
	assert(full_orig.size() == full_tmp.size());
	auto best = eval(full_orig, o2[start2]);
	auto cur = eval(full_tmp, o2[start2]);
 
/*        auto full_tmp = reconstruct(path1, skip, t1, tmp);
        auto full_res = reconstruct(path1, skip, t1, dyn[end2]);

        double best = eval(full_res, 0);
        double cur = eval(full_tmp, 0);*/

		

        if (cur < best) {
          printf("have pot improvement: %.2lf\n", cur - best);
	  auto tmp = dyn[start2-1];
          for (int i = start2; i <= end2; i++) tmp[i] = pp2[start2] + i - start2;
          auto full_res = reconstruct(path1, skip, t1, dyn[end2]);
          auto full_tmp = reconstruct(path1, skip, t1, tmp);
          double best2 = eval(full_res, 0);
          double cur2 = eval(full_tmp, 0);
          if (cur2 < best2) {
            printf("have real improvement: %.2lf\n", cur - best);
            dyn[end2] = tmp;
          }
	  best = cur;
        }
	auto inter = interesting[i];
	for (auto inter2: interesting) {
	    if (inter2.first > inter.first + 1 && inter2.second < inter.second - 1) {
		if (inter_set.count(make_pair(inter2.second, inter.second))) {
		    continue;
		}
//		printf("%d %d %d %d\n", inter.first, inter.second, inter2.first, inter2.second);

/*		double cur22; 
		{

		    vector<int> tmp;
		    vector<int> orig(pp2.begin() + start2, pp2.begin() + end2 + 1);
		    auto orig2 = vector<int>(pp2.begin() + inter2.first, pp2.begin() + inter2.second);
		    for (int i = start2; i <= end2; i++) tmp.push_back(pp2[start2] + i - start2);
		    auto orig_pos = orig2[0] - pp2[start2];
		    assert(tmp[orig_pos] == orig2[0]);
		    for (int i = 0; i < orig2.size(); i++) {
			tmp[i+orig_pos] = orig2[i];
		    }
		    assert(tmp.size() == orig.size());

		    auto full_orig = reconstruct_part(path1, skip, t1, orig);
		    auto full_tmp = reconstruct_part(path1, skip, t1, tmp);
		    assert(full_orig.size() == full_tmp.size());
		    //auto best = eval(full_orig, o2[start2]);
		    cur22 = eval(full_tmp, o2[start2]);
		}*/

		auto orig2 = vector<int>(pp2.begin() + inter2.first, pp2.begin() + inter2.second + 1);
                auto orig_pos = orig2[0] - pp2[start2] + start2;
		auto full_orig = reconstruct_part(path1, skip, t1, orig2);
		auto tmp = vector<int>();
		for (int i = 0; i < orig2.size(); i++) {
		    tmp.push_back(orig2[0]+i);
		}
		auto full_tmp = reconstruct_part(path1, skip, t1, tmp);
		auto offseta = o2[start2] + offset_tmp[orig2[0] - pp2[start2]];
/*		for (int offset = 0; offset < 10; offset++) {*/
			auto bestx = eval(full_tmp, offseta);
			auto curx = eval(full_orig, offseta);
 
	
/*			printf("%lf %lf %d %d %d %d\n", cur22, cur - bestx + curx,
				offseta % 10, offseta % 10, o2[start2] % 10,
				offset_tmp[orig2[0]-pp2[start2]] % 10);
			printf("%d %d %d %d\n", orig2[0], tmp[0], orig2.back(), tmp.back());*/
			
//		}
		if ((curx < bestx) && (cur - bestx + curx < best)) {
/*		if (cur22 < best) {*/
		  printf("have pot improvement2: %.2lf\n", cur - best);
		  auto tmp = dyn[start2-1];
		  for (int i = start2; i <= end2; i++) tmp[i] = pp2[start2] + i - start2;
   		  auto orig_pos = orig2[0] + start2 - pp2[start2];
		  for (int i = 0; i < orig2.size(); i++) {
		    tmp[i+orig_pos] = orig2[i];
		  }
		  auto full_res = reconstruct(path1, skip, t1, dyn[end2]);
		  auto full_tmp = reconstruct(path1, skip, t1, tmp);
		  double best2 = eval(full_res, 0);
		  double cur2 = eval(full_tmp, 0);
		  if (cur2 < best2) {
		    printf("have real improvement2: %.2lf\n", cur - best);
		    dyn[end2] = tmp;
		  }
		  best = cur;
		} else {
		  //printf("%.2lf %.2lf\n", cur, cur - best);
		}

	    }
	}
//    }
  }
  return reconstruct(path1, skip, t1, dyn[last_end]);
}

int main(int argc, char* argv[]) {
  if (argc != 4 && argc != 5 && argc != 6 && argc != 7) {
    assert(argc >= 1);
    printf("Usage: %s path1.csv path2.csv out.csv\n", argv[0]);
    return 1;
  }
  if (argc >= 5) {
    penalty = atof(argv[4]);
  }    
  if (argc >= 6) {
    length_slope = atof(argv[5]);
  }    
  if (argc >= 7) {
    max_bonus = atof(argv[6]);
  }    

  gen_primes();
  read_cities();
  auto path1 = read_path(argv[1]);
  auto path2 = read_path(argv[2]);
  printf("start1 %.3lf\n", eval(path1, 0));
  printf("start2 %.3lf\n", eval(path2, 0));
  auto recomba = recombine(path1, path2);
  auto recombb = recombine(path2, path1);
  auto evala = eval(recomba, 0);
  auto evalb = eval(recombb, 0);
  printf("output %.3lf %.3lf\n", evala, evalb);
  if (evala < evalb) {
      write_path(argv[3],recomba);
  } else {
      write_path(argv[3],recombb);
  }
}
