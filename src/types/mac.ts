/** Adapter MAC info reported by the backend (verified actual state) */
export interface MacInfo {
  adapterName: string
  currentMac: string
  permanentMac: string
  isModified: boolean
}
