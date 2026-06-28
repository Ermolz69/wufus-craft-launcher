interface IconProps {
  size?: number
  className?: string
}

/**
 * Isometric cube — placeholder logo for Wufus Craft.
 * Three faces at different opacity give depth without extra colours.
 */
export function IconLogo({ size = 24, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      aria-hidden="true"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Top face */}
      <path d="M12 2 L22 7 L12 12 L2 7 Z" fill="var(--accent-primary)" opacity="0.95" />
      {/* Right face */}
      <path d="M22 7 L22 17 L12 22 L12 12 Z" fill="var(--accent-primary)" opacity="0.5" />
      {/* Left face */}
      <path d="M2 7 L12 12 L12 22 L2 17 Z" fill="var(--accent-primary)" opacity="0.7" />
    </svg>
  )
}
