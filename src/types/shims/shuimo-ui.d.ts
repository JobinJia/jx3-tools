/**
 * Minimal type shim for shuimo-ui.
 *
 * The published shuimo-ui-nightly package ships raw .ts/.tsx source next to its
 * type declarations, and `types/components/components.d.ts` imports component
 * files by extension-less relative paths that resolve to the raw sources.
 * Type-checking those sources fails under `verbatimModuleSyntax` (and they
 * reference missing modules like `@floating-ui/dom`), so `tsconfig.app.json`
 * maps the `shuimo-ui` specifier to this shim instead. Runtime resolution
 * (Vite) is unaffected and still uses the package's dist build.
 *
 * Remove this shim (and the `paths` entry) once upstream ships clean d.ts files.
 */
import type { App } from 'vue'

export interface ShuimoUI {
  install: (app: App) => App
}

export type MWCType = 'MBorder' | 'MRicePaper'

export interface MUIOption {
  component?: string[] | {
    includes?: string[]
    excludes?: string[]
  }
  disableWebComponent?: MWCType[]
  svgInject?: 'auto' | 'wrapper' | 'nuxt'
}

export declare function createMUI(options?: MUIOption): ShuimoUI
