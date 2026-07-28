const integerFormatter = new Intl.NumberFormat('en-US')

export function formatQuantity(value: number) {
  return integerFormatter.format(value)
}
