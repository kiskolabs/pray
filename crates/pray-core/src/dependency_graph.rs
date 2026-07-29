use std::collections::{BTreeMap, BTreeSet};

/// Returns one cycle path `A -> B -> ... -> A` when the directed dependency graph
/// among known package names contains a cycle. Edges to unknown packages are ignored.
pub fn find_dependency_cycle(edges: &BTreeMap<String, Vec<String>>) -> Option<Vec<String>> {
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();

    for name in edges.keys() {
        if visited.contains(name) {
            continue;
        }
        if let Some(cycle) =
            depth_first_search(name, edges, &mut visiting, &mut visited, &mut stack)
        {
            return Some(cycle);
        }
    }
    None
}

fn depth_first_search(
    name: &str,
    edges: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visited.contains(name) {
        return None;
    }
    if !visiting.insert(name.to_string()) {
        let cycle_start = stack.iter().position(|entry| entry == name)?;
        let mut cycle = stack[cycle_start..].to_vec();
        cycle.push(name.to_string());
        return Some(cycle);
    }

    stack.push(name.to_string());
    if let Some(dependencies) = edges.get(name) {
        for dependency in dependencies {
            if !edges.contains_key(dependency) {
                continue;
            }
            if let Some(cycle) = depth_first_search(dependency, edges, visiting, visited, stack) {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    visiting.remove(name);
    visited.insert(name.to_string());
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(name, deps)| {
                (
                    (*name).to_string(),
                    deps.iter().map(|dep| (*dep).to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn detects_two_node_cycle() {
        let edges = graph(&[("a", &["b"]), ("b", &["a"])]);
        let cycle = find_dependency_cycle(&edges).expect("cycle");
        assert!(cycle.len() >= 3);
        assert_eq!(cycle.first(), cycle.last());
    }

    #[test]
    fn accepts_dag() {
        let edges = graph(&[("a", &["b"]), ("b", &["c"]), ("c", &[])]);
        assert!(find_dependency_cycle(&edges).is_none());
    }

    #[test]
    fn ignores_edges_to_unknown_packages() {
        let edges = graph(&[("a", &["missing"]), ("b", &[])]);
        assert!(find_dependency_cycle(&edges).is_none());
    }

    #[test]
    fn detects_self_cycle() {
        let edges = graph(&[("a", &["a"])]);
        let cycle = find_dependency_cycle(&edges).expect("self cycle");
        assert_eq!(cycle, vec!["a".to_string(), "a".to_string()]);
    }
}
