use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::cmp;
use std::io::BufRead;

#[derive(Clone)]
struct Cocktail {
    ingredients: HashSet<String>,
    name: String,
    cost: f32,
    singular: bool,
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

fn amortized_cost(candidates: &Vec<(HashSet<String>, String)>) -> Vec<Cocktail> {
    let mut cardinality = HashMap::default();

    for (ingredients, _) in candidates {
        for ingredient in ingredients.iter() {
            *cardinality.entry(ingredient).or_insert(0.0) += 1.0;
        }
    }

    let mut cocktail_candidates: Vec<_> = candidates
        .iter()
        .map(|(ingredients, name)| Cocktail {
            ingredients: ingredients.to_owned(),
            name: name.to_owned(),
            cost: ingredients.iter().map(|x| 1.0 / cardinality[x]).sum(),
            singular: ingredients.iter().any(|x| cardinality[x] == 1.0),
        })
        .collect();
    cocktail_candidates.sort_by(|a, b| b.cost.total_cmp(&a.cost));
    cocktail_candidates
}

fn singleton_bound(candidates: &[&Cocktail], ingredient_budget: usize) -> usize {
    let n_singular_cocktails = candidates.iter().filter(|x| x.singular).count();

    candidates.len() - n_singular_cocktails + cmp::min(n_singular_cocktails, ingredient_budget)
}

fn search(cocktails: &Vec<(HashSet<String>, String)>, max_size: usize) -> Vec<Cocktail> {
    let mut highest_score = 0;
    let mut highest: Vec<&Cocktail> = vec![];

    let candidates = amortized_cost(cocktails);
    let candidates_ref: Vec<&Cocktail> = candidates.iter().collect();

    let mut exploration_stack: Vec<(Vec<&Cocktail>, Vec<&Cocktail>, Vec<&Cocktail>)> =
        vec![(candidates_ref, vec![], vec![])];

    while let Some((mut candidates, partial, forbidden)) = exploration_stack.pop() {
        let partial_ingredients: HashSet<_> = partial
            .iter()
            .flat_map(|x| x.ingredients.iter())
            .cloned()
            .collect();
        let disallowed = forbidden
            .iter()
            .any(|x| partial_ingredients.is_superset(&x.ingredients));

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
                let new_partial_ingredients = &partial_ingredients | &best.ingredients;

                let feasible_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| {
                        new_partial_ingredients.len() + x.ingredients.len() <= max_size
                            || new_partial_ingredients.union(&x.ingredients).count() <= max_size
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
    println!("{:#?}", highest.len());
}
