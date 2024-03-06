use roaring::RoaringBitmap;
use std::cmp;
use std::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
use std::io::BufRead;

#[derive(Clone, Debug)]
struct Cocktail<'a> {
    ingredients: Vec<&'a str>,
    name: String,
    cost: f32,
    singular: bool,
    bitset: RoaringBitmap,
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

fn amortized_cost(candidates: &[(HashSet<String>, String)], max_size: usize) -> Vec<Cocktail> {
    let cardinality: HashMap<_, f32> = candidates
        .iter()
        .filter(|(ingredients, _)| ingredients.len() <= max_size)
        .flat_map(|(ingredients, _)| ingredients.iter())
        .fold(HashMap::default(), |mut acc, ingredient| {
            *acc.entry(ingredient.to_string()).or_insert(0.0) += 1.0;
            acc
        });

    let domain: HashMap<_, u32> = cardinality
        .keys()
        .enumerate()
        .map(|(idx, value)| (value.clone(), idx as u32))
        .collect();

    let mut cocktail_candidates: Vec<_> = candidates
        .iter()
        .filter(|(ingredients, _)| ingredients.len() <= max_size)
        .map(|(ingredients, name)| {
            let mut cocktail_bitset = RoaringBitmap::new();
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
    cocktail_candidates
}

fn singleton_bound(candidates: &[&Cocktail], ingredient_budget: usize) -> usize {
    let n_singular_cocktails = candidates.iter().filter(|x| x.singular).count();

    candidates.len() - n_singular_cocktails + cmp::min(n_singular_cocktails, ingredient_budget)
}

fn forbidden_bound(forbidden: &[&Cocktail], partial_ingredients: &RoaringBitmap) -> bool {
    !forbidden
        .iter()
        .any(|x| x.bitset.is_subset(partial_ingredients))
}

fn search(cocktails: &[(HashSet<String>, String)], max_size: usize) -> Vec<Cocktail> {
    let mut highest_score = 0;
    let mut highest: Vec<&Cocktail> = vec![];

    let candidates = amortized_cost(cocktails, max_size);
    let candidates_ref: Vec<&Cocktail> = candidates.iter().collect();

    let mut exploration_stack: Vec<(
        Vec<&Cocktail>,
        Vec<&Cocktail>,
        Vec<&Cocktail>,
        RoaringBitmap,
    )> = vec![(candidates_ref, vec![], vec![], RoaringBitmap::new())];

    while let Some((mut candidates, partial, forbidden, partial_ingredients)) =
        exploration_stack.pop()
    {
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
            && singleton_bound(&candidates, max_size - partial_ingredients.len() as usize)
                > threshold
            && forbidden_bound(&forbidden, &partial_ingredients)
        {
            if let Some(best) = candidates.pop() {
                let new_partial_ingredients = &partial_ingredients | &best.bitset;
                let window = max_size as u64 - new_partial_ingredients.len();

                let feasible_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| {
                        x.bitset.len() <= window
                            || x.bitset.difference_len(&new_partial_ingredients) <= window
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
    //println!("{:#?}", highest);
}
