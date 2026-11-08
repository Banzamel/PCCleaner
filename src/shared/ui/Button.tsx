import type { ComponentProps } from 'react'

export type ButtonStyle = 'primary' | 'green' | 'secondary' | 'outline' | 'link'

interface Props extends ComponentProps<'button'> {
  look?: ButtonStyle
}

/** Buttons of the old window: primary (blue), green, secondary (light blue), outline, link. */
export function Button({ look = 'primary', className, type = 'button', ...rest }: Props) {
  return <button type={type} className={`btn btn-${look} ${className ?? ''}`} {...rest} />
}
