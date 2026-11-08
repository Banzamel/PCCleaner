// JSON schemas for structured answers (Ollama "format").

export function leftoverSchema(): object {
  return {
    type: 'object',
    properties: {
      explanations: {
        type: 'array',
        items: { type: 'object', properties: { id: { type: 'string' }, description: { type: 'string' } }, required: ['id', 'description'] },
      },
    },
    required: ['explanations'],
  }
}

/** Items with a description and advice from a fixed list. */
export function adviceSchema(advice: string[]): object {
  return {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        items: {
          type: 'object',
          properties: { id: { type: 'string' }, description: { type: 'string' }, advice: { type: 'string', enum: advice } },
          required: ['id', 'description', 'advice'],
        },
      },
    },
    required: ['items'],
  }
}

export interface DescribedItem {
  id: string
  description: string
  advice?: string
}

/** Parses a structured answer; items without an id or a description are dropped. */
export function parseItems(answer: string, field: 'explanations' | 'items'): DescribedItem[] {
  const data: unknown = JSON.parse(answer)
  if (typeof data !== 'object' || data === null) return []
  const list = (data as Record<string, unknown>)[field]
  if (!Array.isArray(list)) return []
  return list.flatMap((x: unknown) => {
    if (typeof x !== 'object' || x === null) return []
    const o = x as Record<string, unknown>
    if (typeof o.id !== 'string' || typeof o.description !== 'string' || !o.description.trim()) return []
    return [{ id: o.id, description: o.description, advice: typeof o.advice === 'string' ? o.advice : undefined }]
  })
}

/** "Description. Advice: ...." as shown in the lists. */
export function withAdvice(description: string, adviceLabel: string, advice: string): string {
  return `${description.trim().replace(/[.;]+$/, '')}. ${adviceLabel}: ${advice}.`
}
