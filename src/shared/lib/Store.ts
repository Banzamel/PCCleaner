type Listener = () => void

/** A small observable state container; React reads it through useSyncExternalStore. */
export class Store<T extends object> {
  private listeners = new Set<Listener>()

  constructor(private state: T) {}

  get = (): T => this.state

  set = (patch: Partial<T> | ((s: T) => Partial<T>)): void => {
    const p = typeof patch === 'function' ? patch(this.state) : patch
    this.state = { ...this.state, ...p }
    this.listeners.forEach((l) => l())
  }

  subscribe = (l: Listener): (() => void) => {
    this.listeners.add(l)
    return () => this.listeners.delete(l)
  }
}
