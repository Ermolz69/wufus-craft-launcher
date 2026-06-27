const SCREENS = ['splash', 'main', 'update', 'settings', 'error'] as const

export type Screen = (typeof SCREENS)[number]

export function isScreen(s: string): s is Screen {
  return (SCREENS as readonly string[]).includes(s)
}
