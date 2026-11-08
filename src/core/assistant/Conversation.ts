export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

/** The model gets the report and only the last exchanges - a long chat must not overflow its context. */
export const CHAT_TURNS = 6

/** Chat history: the system message with the report, then questions and answers. */
export class Conversation {
  private messages: ChatMessage[] = []

  constructor(system: string) {
    this.messages = [{ role: 'system', content: system }]
  }

  /** Messages for the next request with the new question appended. */
  request(question: string): ChatMessage[] {
    const history = this.messages.slice(1).slice(-2 * CHAT_TURNS)
    return [this.messages[0], ...history, { role: 'user', content: question }]
  }

  /** Records a finished exchange (a failed question is not kept). */
  record(question: string, answer: string): void {
    this.messages.push({ role: 'user', content: question }, { role: 'assistant', content: answer })
  }

  get length(): number {
    return this.messages.length
  }
}
