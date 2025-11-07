// Viewport and safe area utilities

export interface ViewportDimensions {
  width: number
  height: number
  safeAreaTop: number
  safeAreaBottom: number
  headerHeight: number
}

/**
 * Get viewport dimensions with safe areas and header height
 */
export function getViewportDimensions(): ViewportDimensions {
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight - 60 // Subtract tab bar height

  // Get safe-area-insets from CSS variables
  const safeAreaTop = parseFloat(
    getComputedStyle(document.documentElement).getPropertyValue(
      '--safe-area-inset-top',
    ) || '0',
  )
  const safeAreaBottom = parseFloat(
    getComputedStyle(document.documentElement).getPropertyValue(
      '--safe-area-inset-bottom',
    ) || '0',
  )

  // Get header height from UI store
  const { headerHeight } = useUiStore()

  return {
    width: viewportWidth,
    height: viewportHeight,
    safeAreaTop,
    safeAreaBottom,
    headerHeight,
  }
}

/**
 * Calculate available content height (viewport height minus safe areas)
 * Note: viewport height already excludes header, so we only subtract safe areas
 */
export function getAvailableContentHeight(): number {
  const dimensions = getViewportDimensions()
  return (
    dimensions.height -
    dimensions.safeAreaTop -
    dimensions.safeAreaBottom
  )
}

/**
 * Calculate fullscreen window dimensions (for small screens)
 */
export function getFullscreenDimensions() {
  const dimensions = getViewportDimensions()

  return {
    x: 0,
    y: 0,
    width: dimensions.width,
    height: getAvailableContentHeight(),
  }
}
