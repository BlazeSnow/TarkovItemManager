import { describe, expect, it } from 'vitest'
import { formatQuantity } from './format'

describe('formatQuantity', () => {
  it('formats integers with thousands separators', () => {
    expect(formatQuantity(0)).toBe('0')
    expect(formatQuantity(7)).toBe('7')
    expect(formatQuantity(1000)).toBe('1,000')
    expect(formatQuantity(15517000)).toBe('15,517,000')
  })
})
