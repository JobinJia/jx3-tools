/** Generic IPC error from Tauri backend */
export interface IpcError {
  message: string
  code?: string
}

/** Loading state for async operations */
export interface AsyncState {
  loading: boolean
  error: string | null
}
