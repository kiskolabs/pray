export function findDependencyCycle(
  edges: Map<string, string[]>,
): string[] | undefined {
  const visiting = new Set<string>();
  const visited = new Set<string>();
  const stack: string[] = [];
  for (const name of [...edges.keys()].sort()) {
    if (visited.has(name)) {
      continue;
    }
    const cycle = depthFirstSearch(name, edges, visiting, visited, stack);
    if (cycle) {
      return cycle;
    }
  }
  return undefined;
}

function depthFirstSearch(
  name: string,
  edges: Map<string, string[]>,
  visiting: Set<string>,
  visited: Set<string>,
  stack: string[],
): string[] | undefined {
  if (visited.has(name)) {
    return undefined;
  }
  if (visiting.has(name)) {
    const start = stack.indexOf(name);
    if (start < 0) {
      return undefined;
    }
    return [...stack.slice(start), name];
  }
  visiting.add(name);
  stack.push(name);
  for (const dependency of edges.get(name) ?? []) {
    if (!edges.has(dependency)) {
      continue;
    }
    const cycle = depthFirstSearch(dependency, edges, visiting, visited, stack);
    if (cycle) {
      return cycle;
    }
  }
  stack.pop();
  visiting.delete(name);
  visited.add(name);
  return undefined;
}
