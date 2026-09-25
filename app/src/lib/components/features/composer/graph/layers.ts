/**
 * Topological layering — a property of the graph, not of the drawing.
 *
 * Kept separate from `layout.ts` on purpose: this answers "how deep is this node",
 * layout answers "where does it go on screen". Two files, two boundaries.
 */

import type { GraphRecipeEdge } from '$lib/types';

export interface LayerInput {
  id: string;
}

/**
 * Depth of every node = longest path from any root.
 *
 * Memoised DFS with a cycle guard: a node currently being visited resolves to 0
 * rather than recursing forever. Cycles are rejected by the backend, but the canvas
 * must stay drawable while the user is halfway through making one.
 */
export function computeLayers(nodes: LayerInput[], edges: GraphRecipeEdge[]): Map<string, number> {
  const parents = new Map<string, string[]>();
  for (const node of nodes) parents.set(node.id, []);
  for (const edge of edges) {
    const list = parents.get(edge.to);
    if (list && parents.has(edge.from)) list.push(edge.from);
  }

  const depth = new Map<string, number>();
  const visiting = new Set<string>();

  function resolve(id: string): number {
    const memo = depth.get(id);
    if (memo !== undefined) return memo;
    if (visiting.has(id)) return 0;

    visiting.add(id);
    let best = 0;
    for (const parent of parents.get(id) ?? []) {
      best = Math.max(best, resolve(parent) + 1);
    }
    visiting.delete(id);

    depth.set(id, best);
    return best;
  }

  for (const node of nodes) resolve(node.id);
  return depth;
}
