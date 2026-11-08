import { useEffect, useRef, useState } from 'react'

import { useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { useChat } from '../hooks/useChat'

/** Questions about the plan, folders, programs or games; works before a scan too (general answers). */
export function ChatPage() {
  const tr = useT()
  const info = useUi((s) => s.info)
  const busy = useUi((s) => s.busy)
  const chat = useUi((s) => s.chat)
  const { send } = useChat()
  const [q, setQ] = useState('')
  const end = useRef<HTMLDivElement>(null)
  const input = useRef<HTMLInputElement>(null)
  useEffect(() => end.current?.scrollIntoView({ block: 'end' }), [chat])
  useEffect(() => input.current?.focus(), [])
  const submit = () => {
    const text = q.trim()
    if (!text || busy || info.noAi) return
    setQ('')
    void send(text)
  }
  return (
    <div className="chat-page">
      <PageHeader
        title={tr.t('AI assistant')}
        info={info.noAi ? tr.t('The assistant is disabled (started with -BezLLM).') : tr.t('Local model {0} - runs on this computer, sends nothing to the internet and never deletes anything.', info.model)}
      />
      <Card className="chat-card">
        {!info.noAi ? (
          <p className="dim chat-welcome">
            {tr.t(
              'Ask about the plan, folders, programs or games. The assistant knows the scan results, the disk analysis and the startup list.\nBefore a scan it only answers in general terms - to ask about the files on this computer, click "Scan" first.\nThe first question starts the model (about 10-20 s).',
            )}
          </p>
        ) : null}
        {chat.map((m, i) => (
          <div key={i} className={`chat-msg chat-${m.who}`}>
            <strong>{m.who === 'you' ? tr.t('You') : 'PC Cleaner'}</strong>
            <p>{m.text}</p>
          </div>
        ))}
        <div ref={end} />
      </Card>
      <Card className="chat-input">
        <input
          ref={input}
          value={q}
          placeholder={tr.t('Type a question and press Enter...')}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && submit()}
          disabled={info.noAi}
        />
        <Button disabled={busy || info.noAi} onClick={submit}>
          {tr.t('Send')}
        </Button>
      </Card>
    </div>
  )
}
