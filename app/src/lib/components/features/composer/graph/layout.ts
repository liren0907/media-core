/**
 * Coordinate engine — compute positions, then draw at those numbers.
 *
 * Nothing here measures the DOM. That is only possible because card sizes are
 * fixed, so THIS FILE IS THE ONLY SOURCE OF TRUTH for them: the component writes
 * these constants into inline styles and the CSS carries no second copy.
 *
 * Coordinates are never stored in the recipe. The same graph lays out identically
 * every time, so there is nothing to lose by recomputing on load.
 */

import type { GraphRecipeEdge } from '$lib/types';
import { computeLayers } from './layers';

export const NODE_W = 220;
/** Header only. */
export const NODE_H = 34;
/** Header plus one summary line. */
export const NODE_H_SUM = 54;
export const LAYER_GAP = 56;
export const ROW_GAP = 28;
/** The `+` buttons float outside the card; without padding they get clipped. */
export const PAD = 30;

export interface LayoutInput {
  id: string;
  /** Whether the card shows a summary line — the only thing that changes its height. */
  hasSummary: boolean;
}

export interface Box {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  cx: number;
  cy: number;
}

export interface EdgeGeom {
  from: string;
  to: string;
  path: string;
  /** Midpoint of the curve — where the on-edge `+` sits. */
  mx: number;
  my: number;
}

export interface Layout {
  boxes: Box[];
  byId: Map<string, Box>;
  edges: EdgeGeom[];
  width: number;
  height: number;
}

export function layoutGraph(nodes: LayoutInput[], edges: GraphRecipeEdge[]): Layout {
  if (nodes.length === 0) {
    return { boxes: [], byId: new Map(), edges: [], width: 0, height: 0 };
  }

  const depth = computeLayers(nodes, edges);
  const order = new Map(nodes.map((n, i) => [n.id, i]));
  const heightOf = new Map(nodes.map((n) => [n.id, n.hasSummary ? NODE_H_SUM : NODE_H]));

  const parents = new Map<string, string[]>();
  for (const node of nodes) parents.set(node.id, []);
  for (const edge of edges) {
    if (parents.has(edge.to) && parents.has(edge.from)) parents.get(edge.to)!.push(edge.from);
  }

  const byLayer = new Map<number, string[]>();
  for (const node of nodes) {
    const layer = depth.get(node.id) ?? 0;
    if (!byLayer.has(layer)) byLayer.set(layer, []);
    byLayer.get(layer)!.push(node.id);
  }

  const byId = new Map<string, Box>();
  const layerKeys = [...byLayer.keys()].sort((a, b) => a - b);

  for (const layer of layerKeys) {
    const ids = byLayer.get(layer)!;
    const x = PAD + layer * (NODE_W + LAYER_GAP);

    // `anchor` (where the upstream sits) and `want` (where this card must go for its
    // own centre to land on the anchor) must stay separate. `want` includes half the
    // card height, so siblings of different heights would sort differently from how
    // they are placed — which visibly swaps two branches over.
    const anchorOf = new Map<string, number | undefined>();
    for (const id of ids) {
      const placed = (parents.get(id) ?? [])
        .map((p) => byId.get(p))
        .filter((b): b is Box => b !== undefined);
      anchorOf.set(
        id,
        placed.length ? placed.reduce((sum, b) => sum + b.cy, 0) / placed.length : undefined
      );
    }

    const sorted = [...ids].sort((a, b) => {
      const av = anchorOf.get(a);
      const bv = anchorOf.get(b);
      if (av === undefined && bv === undefined) return (order.get(a) ?? 0) - (order.get(b) ?? 0);
      if (av === undefined) return 1;
      if (bv === undefined) return -1;
      if (av !== bv) return av - bv;
      return (order.get(a) ?? 0) - (order.get(b) ?? 0);
    });

    let cursor = PAD;
    for (const id of sorted) {
      const h = heightOf.get(id) ?? NODE_H;
      const anchor = anchorOf.get(id);
      // Align on centres, not top edges: with two card heights, aligning tops draws
      // faintly sloped connections.
      const want = anchor === undefined ? cursor : anchor - h / 2;
      const y = Math.round(Math.max(cursor, want));
      byId.set(id, { id, x, y, w: NODE_W, h, cx: x + NODE_W / 2, cy: y + h / 2 });
      cursor = y + h + ROW_GAP;
    }
  }

  const boxes = nodes.map((n) => byId.get(n.id)!).filter(Boolean);
  const width = PAD + (layerKeys.length ? layerKeys[layerKeys.length - 1] : 0) * (NODE_W + LAYER_GAP) + NODE_W + PAD;
  const height = boxes.reduce((max, b) => Math.max(max, b.y + b.h), 0) + PAD;

  return { boxes, byId, edges: routeEdges(edges, byId), width, height };
}

/**
 * Anchor points spread evenly along the facing edges, each side sorted by the centre
 * of the opposite card so that fan-out and fan-in do not cross themselves.
 */
function routeEdges(edges: GraphRecipeEdge[], byId: Map<string, Box>): EdgeGeom[] {
  const live = edges.filter((e) => byId.has(e.from) && byId.has(e.to));

  const outSlots = new Map<string, string[]>();
  const inSlots = new Map<string, string[]>();

  for (const edge of live) {
    if (!outSlots.has(edge.from)) outSlots.set(edge.from, []);
    outSlots.get(edge.from)!.push(edge.to);
    if (!inSlots.has(edge.to)) inSlots.set(edge.to, []);
    inSlots.get(edge.to)!.push(edge.from);
  }
  for (const [id, targets] of outSlots) {
    targets.sort((a, b) => (byId.get(a)!.cy - byId.get(b)!.cy) || a.localeCompare(b));
    outSlots.set(id, targets);
  }
  for (const [id, sources] of inSlots) {
    sources.sort((a, b) => (byId.get(a)!.cy - byId.get(b)!.cy) || a.localeCompare(b));
    inSlots.set(id, sources);
  }

  return live.map((edge) => {
    const from = byId.get(edge.from)!;
    const to = byId.get(edge.to)!;

    const outs = outSlots.get(edge.from)!;
    const ins = inSlots.get(edge.to)!;
    const oi = outs.indexOf(edge.to);
    const ii = ins.indexOf(edge.from);

    const x1 = from.x + from.w;
    const y1 = Math.round(from.y + (from.h * (oi + 1)) / (outs.length + 1));
    const x2 = to.x;
    const y2 = Math.round(to.y + (to.h * (ii + 1)) / (ins.length + 1));

    const dx = Math.max(10, Math.abs(x2 - x1) * 0.4);
    const c1x = x1 + dx;
    const c2x = x2 - dx;

    // Cubic Bézier midpoint at t = 0.5.
    const mx = Math.round((x1 + 3 * c1x + 3 * c2x + x2) / 8);
    const my = Math.round((y1 + 3 * y1 + 3 * y2 + y2) / 8);

    return {
      from: edge.from,
      to: edge.to,
      path: `M ${x1} ${y1} C ${c1x} ${y1}, ${c2x} ${y2}, ${x2} ${y2}`,
      mx,
      my,
    };
  });
}
