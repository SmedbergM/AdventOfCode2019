use regex;
use itertools::Itertools;
use std::collections::{BTreeSet, BTreeMap};
use std::fmt::{Display, Formatter, Error};

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
struct Compound {
    name: String
}

impl Compound {
    fn from_str(s: &str) -> Compound {
        Compound { name: String::from(s) }
    }
}

impl Display for Compound {
    fn fmt(&self, writer: &mut Formatter) -> Result<(), Error> {
        write!(writer, "{}", self.name)
    }
}

struct CompoundStore<'a> {
    store: BTreeMap<&'a Compound, usize>
}

impl<'a> CompoundStore<'a> {
    fn new() -> CompoundStore<'a> {
        CompoundStore { store: BTreeMap::new() }
    }

    fn get(&self, k: &Compound) -> Option<usize> {
        self.store.get(k).map(|u| *u)
    }

    fn increment(&mut self, k: &'a Compound, v: usize) {
        *self.store.entry(k).or_insert(0) += v;
    }

    fn decrement(&mut self, k: &'a Compound, v: usize) {
        match self.store.get(k) {
            None => (),
            Some(&u) if u <= v => {
                self.store.remove(k);
            },
            Some(_) => {
                self.store.entry(k).and_modify(|u| *u -= v);
            }
        }
    }

    fn by_height<F>(&self, height: F) -> Vec<(&Compound, usize)> where F: Fn(&Compound) -> usize {
        let mut v: Vec<(&Compound, usize)> = self.store.clone().into_iter().collect();
        v.sort_unstable_by_key(|(c, _)| height(c));
        v
    }
}

impl<'a> IntoIterator for CompoundStore<'a> {
    type IntoIter = std::collections::btree_map::IntoIter<&'a Compound, usize>;
    type Item = (&'a Compound, usize);

    fn into_iter(self) -> Self::IntoIter { self.store.into_iter() }
}

impl Display for CompoundStore<'_> {
    fn fmt(&self, writer: &mut Formatter) -> Result<(), std::fmt::Error> {
        let contents = if self.store.len() > 0 {
            let mut c = self.store.iter().map(|(compound, n)| format!("{}:{}", compound, n)).join(", ");
            c.push(' ');
            c
        } else {
            String::new()
        };
        write!(writer, "{{ {}}}", contents)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
struct Reagent {
    compound: Compound,
    n: usize
}

impl Display for Reagent {
    fn fmt(&self, writer: &mut Formatter) -> Result<(), Error> {
        write!(writer, "{}:{}", self.compound, self.n)
    }
}

impl Reagent {
    fn from_str(s: &str) -> Option<Reagent> {
        let pat = regex::Regex::new(r"(\d+)\s+(\w+)").unwrap();
        pat.captures(s).and_then(|caps| {
            caps.get(1).and_then(|m1| caps.get(2).map(|m2| (m1, m2))).and_then(|(m1, m2)|{
                usize::from_str_radix(m1.as_str(), 10).ok().map(|n| {
                    let name = String::from(m2.as_str());
                    Reagent {
                        compound: Compound { name },
                        n
                    }
                })
            })
        })
    }

    fn apply(name: &str, n: usize) -> Reagent { // used in tests stoopid warn_unused
        Reagent { compound: Compound { name: String::from(name)}, n }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Reaction {
    reagents: BTreeSet<Reagent>,
    product: Reagent
}

impl Display for Reaction {
    fn fmt(&self, writer: &mut Formatter) -> Result<(), Error> {
        write!(writer, "{} => {}", self.reagents.iter().join(","), self.product)
    }
}

impl Reaction {
    fn from_str(line: &str) -> Option<Reaction> {
        let opt_reagents_product = {
            let mut rp = line.split(" => ");
            rp.next().and_then(|r| rp.next().map(|p| (r, p)))
        };
        opt_reagents_product.and_then(|(rs, p)| {
            let reagents: BTreeSet<Reagent> = rs.split(", ").flat_map(|r| Reagent::from_str(r)).collect();
            Reagent::from_str(p).map (|product| Reaction { reagents, product })
        })
    }

    fn produce<'a, 'b>(&'a self,
                   product_desired: usize,
                   available: &'b mut CompoundStore<'a>) -> Result<usize, CompoundStore> { 
        // returns the number of ore used in this reaction if successful
        // If successful the products will be added to `available`;
        // If insufficient reagents are available, the Err value will be specify how many are needed.
        // This method is safe -- all reagents are checked before any mutations are made.

        let reaction_count = match product_desired % self.product.n {
            0 => product_desired / self.product.n,
            _ => 1 + product_desired / self.product.n
        };

        enum ReactionResult<'a> {
            Ok(usize, CompoundStore<'a>),
            InsufficientResources(CompoundStore<'a>)
        }
        
        impl<'a> ReactionResult<'a> {
            fn consume_ore(self, ore: usize) -> ReactionResult<'a> {
                match self {
                    ReactionResult::Ok(prev_ore, prev_consumed) => ReactionResult::Ok(prev_ore + ore, prev_consumed),
                    ins => ins
                }
            }
        
            fn consume(self, compound: &'a Compound, n: usize) -> ReactionResult {
                match self {
                    ReactionResult::InsufficientResources(_) => self,
                    ReactionResult::Ok(prev_ore, mut prev_consumed) => {
                        prev_consumed.increment(compound, n);
                        ReactionResult::Ok(prev_ore, prev_consumed)
                    }
                }
            }
        
            fn insufficient(self, compound: &'a Compound, n: usize) -> ReactionResult {
                match self {
                    ReactionResult::Ok(_, _) => {
                        let mut m = CompoundStore::new();
                        m.increment(compound, n);
                        ReactionResult::InsufficientResources(m)
                    },
                    ReactionResult::InsufficientResources(mut ins) => {
                        ins.increment(compound, n);
                        ReactionResult::InsufficientResources(ins)
                    }
                }
            }
        }
        
        let result = self.reagents.iter().fold(ReactionResult::Ok(0, CompoundStore::new()), |result, reagent| {
            let av = available.get(&reagent.compound).unwrap_or(0);
            let needed: usize = reagent.n * reaction_count;
            if reagent.compound.name == "ORE" {
                // println!("{} consumes {} ORE producing {} {}", self, needed, self.product.n * reaction_count, self.product.compound);
                result.consume_ore(needed)
            } else if av >= needed {
                result.consume(&reagent.compound, needed)
            } else {
                result.insufficient(&reagent.compound, needed - av)
            }
        });

        match result {
            ReactionResult::Ok(ore, consumed) => {
                // println!("Requested: {} units of {} via {}", product_desired, self.product.compound, self);
                for (compound, c) in consumed {
                    available.decrement(compound, c);
                    // println!("Consumed {} units of {}; available {}", c, compound, available);
                }
                available.increment(&self.product.compound, reaction_count * self.product.n);
                // println!("Produced {} units of {}; available {}", reaction_count * self.product.n, self.product.compound, available);
                Ok(ore)
            },
            ReactionResult::InsufficientResources(ins) => Err(ins)
        }
    }
}


struct NanoFactory {
    reactions: BTreeMap<Compound, Reaction> // if (c -> r) in the map, then r.product.compound must equal c
}

impl NanoFactory {
    fn parse<'a, J>(lines: J) -> NanoFactory where J: Iterator<Item=String> {
        let mut reactions = BTreeMap::new();

        for line in lines {
            for reaction in Reaction::from_str(line.as_str()) {
                reactions.insert(reaction.product.compound.clone(), reaction);
            }
        }

        NanoFactory { reactions }
    }

    fn height(&self) -> BTreeMap<&Compound, usize> {
        let mut heights = BTreeMap::new();

        fn resolve_height<'b>(c: &'b Compound, reactions: &'b BTreeMap<Compound, Reaction>, mut heights: BTreeMap<&'b Compound, usize>) -> BTreeMap<&'b Compound, usize> {
            if heights.contains_key(c) {
                return heights
            } else if c.name == "ORE" {
                heights.insert(c, 0);
                return heights
            } else {
                let mut h = 0;
                for reagent in &reactions[c].reagents {
                    let next_heights = resolve_height(&reagent.compound, reactions, heights);
                    heights = next_heights;
                    h = usize::max(h, 1 + heights[&reagent.compound]);
                }
                heights.insert(c, h);
                return heights
            }
        }

        for (_, reaction) in &self.reactions {
            let next_heights = resolve_height(&reaction.product.compound, &self.reactions, heights);
            heights = next_heights;
        }

        heights
    }

    fn produce_reagent<'a>(&'a self,
                           compound: &Compound,
                           desired: usize,
                           mut available: CompoundStore<'a>,
                           heights: &BTreeMap<&Compound, usize>) -> (usize, CompoundStore<'a>) {
        let mut total_ore = 0;
        let root_reaction = &self.reactions[compound];
        let root_reaction_result = root_reaction.produce(desired, &mut available);
        match root_reaction_result {
            Ok(ore) => {
                (total_ore + ore, available)
            },
            Err(missing) => {
                // println!("INSUFFICIENT resources for {} of {} via {}", desired, compound, root_reaction);
                // println!("Missing {}, available {}", missing, available);
                for (precursor, precursor_desired) in missing.by_height(|c| heights[c]) {
                    let (precursor_ore, next_available) = self.produce_reagent(precursor, precursor_desired, available, heights);
                    total_ore += precursor_ore;
                    available = next_available;
                };
                let (root_ore, next_available) = self.produce_reagent(compound, desired, available, heights);
                (total_ore + root_ore, next_available)
            }
        }
    }

    pub fn produce_one_fuel(&self) -> usize {
        let heights = self.height();
        let compound = Compound::from_str("FUEL");
        let (ore, _unused) = self.produce_reagent(&compound, 1, CompoundStore::new(), &heights);
        ore        
    }

    pub fn consume_ore(&self, n: usize) -> usize {
        let heights = self.height();
        let fuel = Compound::from_str("FUEL");
        let mut total_ore = 0;
        let mut total_fuel = 0;
        let (ore_1, mut available) = self.produce_reagent(&fuel, 1, CompoundStore::new(), &heights);
        
        println!("{} ORE required to produce 1 FUEL.", ore_1);
        total_ore += ore_1;
        total_fuel += 1;
        fn next_target(ore_used: &usize, ore_budget: &usize, est_per_fuel: &usize) -> usize {
            usize::max(1, (ore_budget - ore_used) / est_per_fuel) 
        }

        let target = next_target(&total_ore, &n, &ore_1);
        let (ore_2, next_available) = self.produce_reagent(&fuel, target, available, &heights);
        
        if ore_2 > n {
            println!("Too much ore {} used, rethink", ore_2);
            return total_fuel
        } else {
            available = next_available;
            total_ore += ore_2;
            total_fuel += target;
        }

        println!("{} ORE required to produce {} FUEL", total_ore, total_fuel);

        loop {
            let target = next_target(&total_ore, &n, &ore_1);
            let (next_ore, next_available) = self.produce_reagent(&fuel, target, available, &heights);
            if total_ore + next_ore > n {
                break
            }
            total_ore += next_ore;
            total_fuel += target;
            available = next_available;
            println!("{} ORE needed to produce {} FUEL", total_ore, total_fuel);
        }

        total_fuel
    }

    pub fn len(&self) -> usize {
        self.reactions.len()
    }
}

fn main() {
    use std::io::BufRead;

    let stdin = std::io::stdin();
    let lines = stdin.lock().lines().flatten();
    let nanofactory = NanoFactory::parse(lines);

    println!("My nanofactory is capable of {} reactions", nanofactory.len());

    let n = nanofactory.produce_one_fuel();
    println!("{} ORE are needed for 1 FUEL", n);

    let trillion = usize::pow(10, 12);
    let fuel = nanofactory.consume_ore(trillion);
    println!("{} FUEL produced from a trillion ORE", fuel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaction_from_str_spec() {
        let line = "10 ORE => 10 A";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("ORE", 10)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("A", 10));

        let line = "1 ORE => 1 B";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("ORE", 1)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("B", 1));

        let line = "7 A, 1 B => 1 C";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("A", 7), Reagent::apply("B", 1)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("C", 1));

        let line = "7 A, 1 C => 1 D";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("A", 7), Reagent::apply("C", 1)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("D", 1));

        let line = "7 A, 1 D => 1 E";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("A", 7), Reagent::apply("D", 1)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("E", 1));

        let line = "7 A, 1 E => 1 FUEL";
        let reaction = Reaction::from_str(line).unwrap();
        let reagents: BTreeSet<Reagent> = [Reagent::apply("A", 7), Reagent::apply("E", 1)].iter().map(|r| r.clone()).collect();
        assert_eq!(reaction.reagents, reagents);
        assert_eq!(reaction.product, Reagent::apply("FUEL", 1));
    }

    #[test]
    fn nanofactory_parse_spec() {
        let puzzle = "10 ORE => 10 A
        1 ORE => 1 B
        7 A, 1 B => 1 C
        7 A, 1 C => 1 D
        7 A, 1 D => 1 E
        7 A, 1 E => 1 FUEL";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(|s| String::from(s)));
        assert_eq!(nanofactory.len(), 6);
        let reaction_ore_a = Reaction::from_str("10 ORE => 10 A").unwrap();
        let reaction_ore_b = Reaction::from_str("1 ORE => 1 B").unwrap();
        let parsed_reation_a = &nanofactory.reactions[&Compound::from_str("A")];
        assert_eq!(*parsed_reation_a, reaction_ore_a);

        let parsed_reaction_b = &nanofactory.reactions[&Compound::from_str("B")];
        assert_eq!(*parsed_reaction_b, reaction_ore_b);

        let reaction_ae_fuel = Reaction::from_str("7 A, 1 E => 1 FUEL").unwrap();
        let parsed_reaction_fuel = &nanofactory.reactions[&Compound::from_str("FUEL")];
        assert_eq!(*parsed_reaction_fuel, reaction_ae_fuel);

        let puzzle = "9 ORE => 2 A
        8 ORE => 3 B
        7 ORE => 5 C
        3 A, 4 B => 1 AB
        5 B, 7 C => 1 BC
        4 C, 1 A => 1 CA
        2 AB, 3 BC, 4 CA => 1 FUEL";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.len(), 7);

        let reaction = Reaction {
            reagents: [Reagent::apply("AB", 2), Reagent::apply("BC", 3), Reagent::apply("CA", 4)].iter().map(|r| r.clone()).collect(),
            product: Reagent::apply("FUEL", 1)
        };
        let parsed_reaction = &nanofactory.reactions[&Compound::from_str("FUEL")];
        assert_eq!(*parsed_reaction, reaction);
    }


    #[test]
    fn fuel_production_spec_1() {
        let puzzle = "10 ORE => 10 A
        1 ORE => 1 B
        7 A, 1 B => 1 C
        7 A, 1 C => 1 D
        7 A, 1 D => 1 E
        7 A, 1 E => 1 FUEL";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.produce_one_fuel(), 31)
    }

    #[test]
    fn fuel_production_spec_2() {
        let puzzle = "9 ORE => 2 A
        8 ORE => 3 B
        7 ORE => 5 C
        3 A, 4 B => 1 AB
        5 B, 7 C => 1 BC
        4 C, 1 A => 1 CA
        2 AB, 3 BC, 4 CA => 1 FUEL";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.produce_one_fuel(), 165);
    }

    #[test]
    fn fuel_production_spec_3() {
        let puzzle = "157 ORE => 5 NZVS
        165 ORE => 6 DCFZ
        44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
        12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
        179 ORE => 7 PSHF
        177 ORE => 5 HKGWZ
        7 DCFZ, 7 PSHF => 2 XJWVT
        165 ORE => 2 GPVTF
        3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.produce_one_fuel(), 13312);
    }

    #[test]
    fn fuel_production_spec_4() {
        let puzzle = "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
        17 NVRVD, 3 JNWZP => 8 VPVL
        53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
        22 VJHF, 37 MNCFX => 5 FWMGM
        139 ORE => 4 NVRVD
        144 ORE => 7 JNWZP
        5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
        5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
        145 ORE => 6 MNCFX
        1 NVRVD => 8 CXFTF
        1 VJHF, 6 MNCFX => 4 RFSQX
        176 ORE => 6 VJHF";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.produce_one_fuel(), 180697);
    }

    #[test]
    fn fuel_production_spec_5() {
        let puzzle = "171 ORE => 8 CNZTR
        7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
        114 ORE => 4 BHXH
        14 VRPVC => 6 BMBT
        6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
        6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
        15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
        13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
        5 BMBT => 4 WPTQ
        189 ORE => 9 KTJDG
        1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
        12 VRPVC, 27 CNZTR => 2 XDBXC
        15 KTJDG, 12 BHXH => 5 XCVML
        3 BHXH, 2 VRPVC => 7 MZWV
        121 ORE => 7 VRPVC
        7 XCVML => 6 RJRHP
        5 BHXH, 4 VRPVC => 5 LTCX";
        let nanofactory = NanoFactory::parse(puzzle.lines().map(String::from));
        assert_eq!(nanofactory.produce_one_fuel(), 2210736);        
    }
}