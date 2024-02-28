use std::cmp;
use std::collections::{BTreeSet, HashMap};
use std::io::BufRead;

type Cocktail = BTreeSet<String>;

struct Cost {
    cost: f32,
    singular: bool,
}

fn read_cocktails() -> HashMap<Cocktail, String> {
    std::io::stdin()
        .lock()
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            let cocktail_name = parts.next()?;
            let ingredients: Cocktail = parts.map(String::from).collect();
            Some((ingredients, cocktail_name.to_string()))
        })
        .collect()
}

fn amortized_cost<'a>(candidates: &'a [&'a Cocktail]) -> HashMap<&'a Cocktail, Cost> {
    let mut costs = HashMap::new();
    let mut cardinality = HashMap::new();

    for cocktail in candidates {
        for ingredient in cocktail.iter() {
            *cardinality.entry(ingredient).or_insert(0.0) += 1.0;
        }
    }

    for cocktail in candidates {
        let cost = Cost {
            cost: cocktail.iter().map(|x| 1.0 / cardinality[x]).sum(),
            singular: cocktail.iter().any(|x| cardinality[x] == 1.0),
        };
        costs.insert(*cocktail, cost);
    }

    costs
}

fn singleton_bound(
    candidates: &[&Cocktail],
    costs: &HashMap<&Cocktail, Cost>,
    ingredient_budget: usize,
) -> usize {
    let n_singular_cocktails = candidates.iter().filter(|x| costs[*x].singular).count();

    candidates.len() - n_singular_cocktails + cmp::min(n_singular_cocktails, ingredient_budget)
}

fn search(cocktails: &HashMap<Cocktail, String>, max_size: usize) -> Vec<&Cocktail> {
    let mut highest_score = 0;
    let mut highest: Vec<&Cocktail> = vec![];

    let original_candidates: Vec<_> = cocktails.keys().filter(|x| x.len() <= max_size).collect();

    let costs = amortized_cost(&original_candidates);

    let mut exploration_stack: Vec<(_, _, Vec<&Cocktail>)> =
        vec![(original_candidates.clone(), vec![], vec![])];

    while let Some((candidates, partial, forbidden)) = exploration_stack.pop() {
        let score = partial.len();

        if score > highest_score {
            highest_score = score;
            highest = partial.clone();
            println!("{:#?}", highest_score);
        }

        let threshold = highest_score - partial.len();
        let partial_ingredients = partial.iter().fold(BTreeSet::new(), |acc, set| &acc | *set);

        if candidates.len() > threshold
            && singleton_bound(&candidates, &costs, max_size - partial_ingredients.len())
                > threshold
        {
            if let Some(&best) = candidates
                .iter()
                .min_by(|a, b| costs[*a].cost.total_cmp(&costs[*b].cost))
            {
                // The branch where we exclude the current best candidate.

                // The filter condition is equivalent to
                // !(&partial_ingredients | x).is_superset(best) but should
                // be faster
                let remaining_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| !partial_ingredients.is_superset(&(best - x)))
                    .cloned()
                    .collect();

                exploration_stack.push((
                    remaining_candidates,
                    partial.clone(),
                    [forbidden.clone(), vec![best]].concat(),
                ));

                // The branch that includes the current best candidate.
                let new_partial_ingredients = &partial_ingredients | best;

                let covered_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| new_partial_ingredients.is_superset(x))
                    .cloned()
                    .collect();

                let feasible_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|x| {
                        let prospective_ingredients = &new_partial_ingredients | x;
                        prospective_ingredients.len() <= max_size
                            && !forbidden
                                .iter()
                                .any(|y| prospective_ingredients.is_superset(y))
                    })
                    .filter(|x| !new_partial_ingredients.is_superset(x))
                    .cloned()
                    .collect();

                exploration_stack.push((
                    feasible_candidates,
                    [partial, covered_candidates].concat(),
                    forbidden,
                ));
            }
        }
    }
    highest
}

fn main() {
    let cocktails = read_cocktails();
    let highest = search(&cocktails, 30);
    println!("{:#?}", highest.len());
}
