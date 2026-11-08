import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import { AppContext } from './app/AppContext'
import { afterStart, boot } from './app/boot'
import { App } from './app/components/App'
import './styles/index.css'

const root = createRoot(document.getElementById('root') as HTMLElement)

boot()
  .then(({ store, services }) => {
    root.render(
      <StrictMode>
        <AppContext.Provider value={{ store, services }}>
          <App />
        </AppContext.Provider>
      </StrictMode>,
    )
    afterStart(store, services)
  })
  .catch((e: unknown) => {
    root.render(<pre className="boot-error">PC Cleaner could not start: {JSON.stringify(e)}</pre>)
  })
