//use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use bitvec::prelude as bv;
use std::cmp;
use std::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
use std::io::BufRead;

#[derive(Clone, Debug)]
struct Cocktail<'a> {
    ingredients: Vec<&'a str>,
    name: String,
    cost: f32,
    singular: bool,
    bitset: bv::BitVec<u64>,
}

fn read_cocktails() -> Vec<(HashSet<String>, String)> {
    std::io::stdin()
        .lock()
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            let cocktail_name = parts.next()?;
            let ingredients: HashSet<_> = parts.map(String::from).collect();
            Some((ingredients, cocktail_name.to_string()))
        })
        .collect()
}

fn amortized_cost(
    candidates: &[(HashSet<String>, String)],
    max_size: usize,
) -> (bv::BitVec<u64>, Vec<Cocktail>) {
    let cardinality: HashMap<_, f32> = candidates
        .iter()
        .filter(|(ingredients, _)| ingredients.len() <= max_size)
        .flat_map(|(ingredients, _)| ingredients.iter())
        .fold(HashMap::default(), |mut acc, ingredient| {
            *acc.entry(ingredient.to_string()).or_insert(0.0) += 1.0;
            acc
        });

    let domain: HashMap<_, _> = cardinality
        .keys()
        .enumerate()
        .map(|(idx, value)| (value.clone(), idx))
        .collect();

    let base_bitset = bv::bitvec![u64, bv::Lsb0; 0; domain.len()];
    println!("Partial Ingredients: {:#?}", base_bitset);

    let mut cocktail_candidates: Vec<_> = candidates
        .iter()
        .filter(|(ingredients, _)| ingredients.len() <= max_size)
        .map(|(ingredients, name)| {
            let mut cocktail_bitset = base_bitset.clone();
            ingredients
                .iter()
                .for_each(|x| cocktail_bitset.set(domain[x], true));

            Cocktail {
                ingredients: ingredients.iter().map(|x| x.as_str()).collect(),
                name: name.to_owned(),
                cost: ingredients.iter().map(|x| 1.0 / cardinality[x]).sum(),
                singular: ingredients.iter().any(|x| cardinality[x] == 1.0),
                bitset: cocktail_bitset,
            }
        })
        .collect();
    cocktail_candidates.sort_by(|a, b| b.cost.total_cmp(&a.cost));
    println!("{:#?}", cocktail_candidates);
    (base_bitset, cocktail_candidates)
}

fn singleton_bound(candidates: &[&Cocktail], ingredient_budget: usize) -> usize {
    let n_singular_cocktails = candidates.iter().filter(|x| x.singular).count();

    candidates.len() - n_singular_cocktails + cmp::min(n_singular_cocktails, ingredient_budget)
}

fn search(cocktails: &[(HashSet<String>, String)], max_size: usize) -> Vec<Cocktail> {
    let mut highest_score = 0;
    let mut highest: Vec<&Cocktail> = vec![];

    let (base_bitset, candidates) = amortized_cost(cocktails, max_size);
    let candidates_ref: Vec<&Cocktail> = candidates.iter().collect();

    let mut exploration_stack: Vec<(
        Vec<&Cocktail>,
        Vec<&Cocktail>,
        Vec<&Cocktail>,
        bv::BitVec<u64>,
    )> = vec![(candidates_ref, vec![], vec![], base_bitset.clone())];

    while let Some((mut candidates, partial, forbidden, partial_ingredients)) =
        exploration_stack.pop()
    {
        let inverted_ingredients = !partial_ingredients.clone();
        let disallowed = forbidden.iter().any(|x| {
            let mut difference = x.bitset.clone();
            difference &= &inverted_ingredients;
            difference.count_ones() == 0
        });

        if disallowed {
            continue;
        }

        let score = partial.len();

        if score > highest_score {
            highest_score = score;
            highest = partial.clone();
            println!(
                "{:#?} cocktails found for {:#?} ingredients",
                highest_score, max_size
            );
        }

        let threshold = highest_score - score;

        if candidates.len() > threshold
            && singleton_bound(&candidates, max_size - partial_ingredients.count_ones()) > threshold
        {
            if let Some(best) = candidates.pop() {
                let mut new_partial_ingredients = partial_ingredients.clone();
                new_partial_ingredients |= &best.bitset;

                let feasible_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| {
                        let mut union = new_partial_ingredients.clone();
                        union |= &x.bitset;
                        union.count_ones() <= max_size
                    })
                    .cloned()
                    .collect();

                let mut new_partial = partial.clone();
                new_partial.push(best);

                let mut new_forbidden = forbidden.clone();
                new_forbidden.push(best);

                // The branch where we exclude the current best candidate.

                exploration_stack.push((candidates, partial, new_forbidden, partial_ingredients));

                // The branch that includes the current best candidate.
                exploration_stack.push((
                    feasible_candidates,
                    new_partial,
                    forbidden,
                    new_partial_ingredients,
                ));
            }
        }
    }
    highest.iter().cloned().cloned().collect()
}

fn main() {
    let cocktails = read_cocktails();
    let highest = search(&cocktails, 30);
    println!("{:#?}", highest);
}
