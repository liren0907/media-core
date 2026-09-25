/**
 * Picker filtering — "what can go here".
 *
 * Judged on the manifest declaration, never on a method id, so a new backend step
 * shows up here with no frontend change at all.
 *
 * This is the permissive side of the pair: a method whose missing input could still
 * be supplied by filling in a parameter is offered (flagged), because the user has
 * not had a chance to type anything yet. The backend validator is the strict side
 * and has the final word. The gap is deliberate — a visible warning beats an option
 * that silently never appears.
 */

import type { GraphMethodMeta, GraphRecipeEdge, GraphSlot, GraphSourceKind } from '$lib/types';

export type InsertPoint =
  | { kind: 'root' }
  | { kind: 'after'; nodeId: string }
  | { kind: 'before'; nodeId: string }
  | { kind: 'below'; nodeId: string }
  | { kind: 'edge'; from: string; to: string };

export interface PickerNode {
  id: string;
  ref: string;
}

export interface PickerModel {
  nodes: PickerNode[];
  edges: GraphRecipeEdge[];
  methods: GraphMethodMeta[];
}

export interface PickerRow {
  meta: GraphMethodMeta;
  /** Set = cannot be placed here, with the reason. Shown disabled, never hidden. */
  blocked?: string;
  /** Set = placeable, but a parameter must be filled in for it to run. */
  needsParam?: string;
}

export interface PickerSection {
  title: string;
  rows: PickerRow[];
}

const SLOT_LABEL: Record<GraphSlot, string> = {
  source: 'a media source',
  metadata: 'metadata',
  analysis: 'an analysis report',
  extractedFrames: 'in-memory frames',
  hlsResult: 'HLS output',
  annotationResult: 'annotation output',
  videoProcessResult: 'frames on disk',
  processResult: 'file processing output',
};

/** Slots written by the given nodes and everything upstream of them. */
function slotsUpTo(
  ids: string[],
  model: PickerModel,
  metaOf: (ref: string) => GraphMethodMeta | undefined
): Set<GraphSlot> {
  const refOf = new Map(model.nodes.map((n) => [n.id, n.ref]));
  const found = new Set<GraphSlot>();
  const seen = new Set<string>();
  const stack = [...ids];

  while (stack.length) {
    const id = stack.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);

    const meta = metaOf(refOf.get(id) ?? '');
    if (meta) for (const write of meta.writes) found.add(write.slot);
    for (const edge of model.edges) if (edge.to === id) stack.push(edge.from);
  }
  return found;
}

function parentsOf(nodeId: string, model: PickerModel): string[] {
  return model.edges.filter((e) => e.to === nodeId).map((e) => e.from);
}

/** Which nodes sit immediately upstream of a new node placed at `point`. */
function upstreamAt(point: InsertPoint, model: PickerModel): string[] {
  switch (point.kind) {
    case 'root':
      return [];
    case 'after':
      return [point.nodeId];
    case 'before':
    case 'below':
      return parentsOf(point.nodeId, model);
    case 'edge':
      return [point.from];
  }
}

/** Whether a new node placed at `point` will have anything after it. */
function hasDownstreamAt(point: InsertPoint, model: PickerModel): boolean {
  switch (point.kind) {
    case 'root':
    case 'below':
      return false;
    case 'after':
      return model.edges.some((e) => e.from === point.nodeId);
    case 'before':
      return true;
    case 'edge':
      return true;
  }
}

export function optionsAt(point: InsertPoint, model: PickerModel): PickerSection[] {
  const byId = new Map(model.methods.map((m) => [m.id, m]));
  const metaOf = (ref: string) => byId.get(ref);

  const upstream = upstreamAt(point, model);
  const available = slotsUpTo(upstream, model, metaOf);
  const hasDownstream = hasDownstreamAt(point, model);
  const usedRefs = new Set(model.nodes.map((n) => n.ref));

  const sourceNode = model.nodes.find((n) => metaOf(n.ref)?.kind === 'source');
  const sourceKind = sourceNode?.ref as GraphSourceKind | undefined;

  // Slots the immediately preceding node writes, used only for ordering: the things
  // that chain directly onto what you just placed should be at the top.
  const direct = new Set<GraphSlot>();
  for (const id of upstream) {
    const meta = metaOf(model.nodes.find((n) => n.id === id)?.ref ?? '');
    if (meta) for (const write of meta.writes) direct.add(write.slot);
  }

  function classify(meta: GraphMethodMeta): PickerRow {
    if (usedRefs.has(meta.id)) {
      return { meta, blocked: 'Already in this graph' };
    }

    if (meta.kind === 'source') {
      if (sourceNode) return { meta, blocked: 'A graph can only have one source' };
      if (upstream.length > 0) return { meta, blocked: 'A source cannot have an input' };
      return { meta };
    }

    if (meta.kind === 'sink' && hasDownstream) {
      return { meta, blocked: 'Nothing can come after an output' };
    }

    let needsParam: string | undefined;

    for (const read of meta.reads) {
      if (read.optional) continue;

      if (available.has(read.slot)) {
        // The three source variants are not interchangeable.
        if (read.slot === 'source' && read.accepts?.length && sourceKind) {
          if (!read.accepts.includes(sourceKind)) {
            if (read.satisfiedByParam) {
              needsParam ??= read.satisfiedByParam;
              continue;
            }
            return { meta, blocked: `Cannot read from a ${sourceKind} source` };
          }
        }
        continue;
      }

      if (read.satisfiedByParam) {
        needsParam ??= read.satisfiedByParam;
        continue;
      }
      return { meta, blocked: `Needs ${SLOT_LABEL[read.slot]}, which nothing upstream produces` };
    }

    return { meta, needsParam };
  }

  function rank(meta: GraphMethodMeta): number {
    // Chains onto what was just placed → first. Merely runnable → next.
    return meta.reads.some((r) => direct.has(r.slot)) ? 0 : 1;
  }

  function build(title: string, kinds: GraphMethodMeta['kind'][]): PickerSection {
    const rows = model.methods
      .filter((m) => kinds.includes(m.kind))
      .map(classify)
      .sort((a, b) => {
        // Blocked rows sink to the bottom but stay visible.
        const ab = a.blocked ? 1 : 0;
        const bb = b.blocked ? 1 : 0;
        if (ab !== bb) return ab - bb;
        if (ab === 1) return 0;
        const ar = rank(a.meta);
        const br = rank(b.meta);
        if (ar !== br) return ar - br;
        const an = a.needsParam ? 1 : 0;
        const bn = b.needsParam ? 1 : 0;
        return an - bn;
      });
    return { title, rows };
  }

  const sections: PickerSection[] = [];
  if (point.kind === 'root' || !sourceNode) sections.push(build('Source', ['source']));
  sections.push(build('Method', ['method']));
  sections.push(build('Output', ['sink']));

  return sections.filter((s) => s.rows.length > 0);
}
