import { useServices } from '../../../app/AppContext'
import { tx } from '../../../core/text/format'

/** Sends a question and appends the question and the answer to the chat. */
export function useChat(): { send: (q: string) => Promise<void> } {
  const { ui, assistant } = useServices()
  const add = (who: 'you' | 'ai' | 'note', text: string) => ui.store.set((s) => ({ chat: [...s.chat, { who, text }] }))
  return {
    send: async (q: string) => {
      add('you', q)
      await ui.busy(async () => {
        ui.status(tx('The assistant is thinking...'))
        const a = await assistant.ask(q)
        if (a) {
          add('ai', a)
          ui.status(tx(''))
        } else add('note', ui.tr.t('(no answer - see the Log)'))
      })
    },
  }
}
