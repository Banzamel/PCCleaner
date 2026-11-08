import type { HTMLAttributes } from 'react'

/** Rounded card without a border - it stands out by colour alone. */
export function Card({ className, ...rest }: HTMLAttributes<HTMLDivElement>) {
  return <div className={`card ${className ?? ''}`} {...rest} />
}
