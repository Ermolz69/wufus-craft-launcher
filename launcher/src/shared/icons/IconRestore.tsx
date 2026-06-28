interface IconProps {
  size?: number
  className?: string
}

export function IconRestore({ size = 16, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      className={className}
      aria-hidden="true"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Back window square */}
      <rect
        x="4.5"
        y="1.5"
        width="10"
        height="10"
        rx="1"
        stroke="currentColor"
        strokeWidth="1.5"
        fill="none"
      />
      {/* Front window square with background to cover back */}
      <rect
        x="1.5"
        y="4.5"
        width="10"
        height="10"
        rx="1"
        stroke="currentColor"
        strokeWidth="1.5"
        fill="var(--bg-base)"
      />
    </svg>
  )
}
