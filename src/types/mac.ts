/** MAC address state */
export interface MacState {
  originalAddress: string
  currentAddress: string
  isChanged: boolean
  autoRestoreEnabled: boolean
}
