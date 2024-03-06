//use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use indexical::bitset::simd::SimdBitset;
use indexical::bitset::BitSet;
use std::cmp;
use std::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
use std::io::BufRead;

#[derive(Clone)]
struct Cocktail<'a> {
    ingredients: Vec<&'a str>,
    name: String,
    cost: f32,
    singular: bool,
    bitset: SimdBitset<u64, 4>,
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
) -> (SimdBitset<u64, 4>, Vec<Cocktail>) {
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

    let base_bitset = SimdBitset::<u64, 4>::empty(domain.len());
    //println!("Partial Ingredients: {:#?}", base_bitset.len());

    let mut cocktail_candidates: Vec<_> = candidates
        .iter()
        .filter(|(ingredients, _)| ingredients.len() <= max_size)
        .map(|(ingredients, name)| {
            let mut cocktail_bitset = base_bitset.clone();
            ingredients.iter().for_each(|x| {
                cocktail_bitset.insert(domain[x]);
            });

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
    //println!("{:#?}", cocktail_candidates);
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

    let mut exploration_stack: Vec<(Vec<&Cocktail>, Vec<&Cocktail>, Vec<&Cocktail>)> =
        vec![(candidates_ref, vec![], vec![])];

    while let Some((mut candidates, partial, forbidden)) = exploration_stack.pop() {
        let mut partial_ingredients = base_bitset.clone();
        partial
            .iter()
            .for_each(|x| partial_ingredients.union(&x.bitset));

        let disallowed = forbidden
            .iter()
            .any(|x| partial_ingredients.superset(&x.bitset));

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
            && singleton_bound(&candidates, max_size - partial_ingredients.len()) > threshold
        {
            if let Some(best) = candidates.pop() {
                // The branch where we exclude the current best candidate.

                exploration_stack.push((
                    candidates.clone(),
                    partial.clone(),
                    [forbidden.clone(), vec![best]].concat(),
                ));

                // The branch that includes the current best candidate.
                let mut new_partial_ingredients = partial_ingredients.clone();
                new_partial_ingredients.union(&best.bitset);

                let feasible_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| {
                        let mut extended_ingredients = new_partial_ingredients.clone();
                        extended_ingredients.union(&x.bitset);
                        extended_ingredients.len() <= max_size
                    })
                    .cloned()
                    .collect();

                exploration_stack.push((
                    feasible_candidates,
                    [partial, vec![best]].concat(),
                    forbidden,
                ));
            }
        }
    }
    highest.iter().cloned().cloned().collect()
}

fn main() {
    let cocktails = read_cocktails();
    let highest = search(&cocktails, 30);
    //println!("{:#?}", highest);
}
