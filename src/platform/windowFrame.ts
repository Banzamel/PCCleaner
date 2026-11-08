// The current window for the shared title bar (src/shared/ui/titlebar): its buttons, and the sizes
// that tell whether it is maximized and whether Windows draws its border.

import { getCurrentWindow } from '@tauri-apps/api/window'

export const windowFrame = {
  minimize: () => getCurrentWindow().minimize(),
  toggleMaximize: () => getCurrentWindow().toggleMaximize(),
  /** A close request: the X keeps its behaviour (hiding to the tray, blocked while cleaning). */
  close: () => getCurrentWindow().close(),
  async measure() {
    const win = getCurrentWindow()
    const [maximized, outer, inner] = await Promise.all([win.isMaximized(), win.outerSize(), win.innerSize()])
    return { maximized, outerWidth: outer.width, innerWidth: inner.width }
  },
  onResized: (handler: () => void) => getCurrentWindow().onResized(() => handler()),
}
