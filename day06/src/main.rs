use std::io;
use std::io::Lines;
use io::prelude::*;
use std::collections::{HashSet, HashMap, VecDeque};
use std::hash::Hash;

use regex::Regex;

fn parse_line(pat: &Regex, line: &str) -> Option<(String, String)> {
    pat.captures(line).and_then(|cap| {
        cap.get(1).and_then(|m1| {
            cap.get(2).map(|m2| (String::from(m1.as_str()), String::from(m2.as_str())))
        })
    })
}

fn parse_puzzle(lines: Lines<std::io::StdinLock<'_>>) -> HashMap<String, HashSet<String>> {
    let pat = Regex::new(r"(\w+)\)*(\w+)").unwrap();
    let mut puzzle = HashMap::new();
    for line in lines.flat_map(|x| x.ok()) {
        for (c, p) in parse_line(&pat, &line) {
            puzzle.entry(c).or_insert(HashSet::new()).insert(p);
        }
    };
    puzzle
}

fn transitive_count<T: Hash + Eq>(covers: &HashMap<T, HashSet<T>>, root: &T) -> usize {
    let mut lt: HashMap<&T, usize> = HashMap::new();
    lt.insert(root, 0);
    let mut q: VecDeque<&T> = VecDeque::new();
    q.push_back(root);
    while let Some(c) = q.pop_front() {
        let c_down = lt[c];
        for ps in covers.get(c) {
            for p in ps {
                lt.insert(p, c_down + 1);
                q.push_back(p)
            }
        }
    }

    lt.iter().fold(0, |acc, (_,v)| acc + v)
}

fn symmetric<'a, T: Hash + Eq + Clone>(digraph: &'a HashMap<T, HashSet<T>>) -> HashMap<&'a T, HashSet<&'a T>> {
    let mut symgraph: HashMap<&'a T, HashSet<&'a T>> = HashMap::new();
    digraph.iter().for_each(|(k, vs)| {
        let vs_sym_k = symgraph.entry(k).or_insert(HashSet::new());
        for t in vs {
            vs_sym_k.insert(t);
        }
        for v in vs {
            symgraph.entry(v).or_insert(HashSet::new()).insert(k);
        }
    });

    symgraph
}

fn parent_planet<'a, T: Hash + Eq>(digraph: &'a HashMap<T, HashSet<T>>, t: &T) -> Option<&'a T> {
    for (parent, children) in digraph {
        if children.contains(t) {
            return Some(&parent)
        }
    }
    return None
}

fn dist<T: Hash + Eq>(symgraph: &HashMap<&T, HashSet<&T>>, src: &T, dest: &T) -> usize {
    let mut visited: HashSet<&T> = HashSet::new();
    let mut q: VecDeque<(&T, usize)> = VecDeque::new();
    q.push_back((&src, 0));
    while let Some((t, tdist)) = q.pop_front() {
        if t == dest {
            return tdist
        } else if !visited.contains(t) {
            visited.insert(t);
            for neighbors in symgraph.get(t) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        q.push_back((neighbor, tdist + 1))
                    }
                }
            }
        }
    }
    
    return usize::max_value()
}

fn main() {
    let root = String::from("COM");
    let santa = String::from("SAN");
    let you = String::from("YOU");
    
    let stdin = io::stdin();
    let covers = parse_puzzle(stdin.lock().lines());

    let t = transitive_count(&covers, &root);
    println!("Transitive count: {}", &t);
    let santas_parent = parent_planet(&covers, &santa).unwrap();
    let your_parent = parent_planet(&covers, &you).unwrap();
    println!("Santa is orbiting {}; you are orbiting {}", &santas_parent, &your_parent);

    let covers_sym = symmetric(&covers);
    let d = dist(&covers_sym, your_parent, santas_parent);
    println!("It will take {} hops to get to Santa's planet.", d);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_puzzle() -> HashMap<String, HashSet<String>> {
        let pat = Regex::new(r"(\w+)\)*(\w+)").unwrap();
        let mut covers: HashMap<String, HashSet<String>> = HashMap::new();
        for line in &[
            "COM)B",
            "B)C",
            "C)D",
            "D)E",
            "E)F",
            "B)G",
            "G)H",
            "D)I",
            "E)J",
            "J)K",
            "K)L"
        ] {
            for (c,p) in parse_line(&pat, line) {
                covers.entry(c).or_insert(HashSet::new()).insert(p);
            }
        }
        covers
    }

    #[test]
    fn transitive_count_test() {
        let covers = get_test_puzzle();

        assert_eq!(transitive_count(&covers, &String::from("COM")), 42);
        assert_eq!(transitive_count(&covers, &String::from("F")), 0);
    }

    #[test]
    fn dist_test() {
        let covers = get_test_puzzle();
        let covers_sym = symmetric(&covers);
        let src = String::from("K");
        let dest = String::from("I");
        assert_eq!(dist(&covers_sym, &src, &dest), 4);
        let src = String::from("H");
        let dest = String::from("L");
        assert_eq!(dist(&covers_sym, &src, &dest), 8);
    }
}