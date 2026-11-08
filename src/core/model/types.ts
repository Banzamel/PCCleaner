// Data shapes returned by the Rust commands (camelCase serde). Kept in one place so the UI and the
// pure logic agree with the IPC contract.

export type Arg = string | { size: number } | { text: Text }

/** A user-facing text: the English sentence is the key; args fill {0}, {1}... */
export interface Text {
  key: string
  args?: Arg[]
}

export type Tone = 'gray' | 'darkGray' | 'white' | 'green' | 'yellow' | 'cyan'
export type StepState = 'waiting' | 'running' | 'ok' | 'partial' | 'skipped' | 'fail'

/** Thrown by every command (AppError in Rust). */
export interface CommandError {
  message: Text
  detail: string
}

export type * from './scan'
export type * from './system'
