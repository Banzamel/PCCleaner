// Events the Rust side sends while it works (progress lines, cleanup steps, driver states).

import { listen } from '@tauri-apps/api/event'

import type { StepState, Text, Tone } from '../core/model/types'

export interface EventMap {
  'pcc://status': Text
  'pcc://line': { text: Text; tone: Tone }
  'pcc://step': { id: string; state: StepState; bytes: number; note: Text | null }
  'pcc://driver': { id: string; state: string; error: string }
  'pcc://ai-install': { text: Text; fraction: number }
  'pcc://close-blocked': 'cleaning' | 'drivers'
  'pcc://tray-hint': null
}

/** Subscribes to an event; returns the unsubscribe function. */
export function on<K extends keyof EventMap>(name: K, handler: (payload: EventMap[K]) => void): () => void {
  let stop: (() => void) | null = null
  let cancelled = false
  void listen<EventMap[K]>(name, (e) => handler(e.payload)).then((un) => {
    if (cancelled) un()
    else stop = un
  })
  return () => {
    cancelled = true
    stop?.()
  }
}
