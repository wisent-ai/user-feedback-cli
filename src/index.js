const STATUSES = ['new', 'reviewed', 'planned', 'shipped', 'declined']

function requiredString(value, label) {
  const normalized = String(value ?? '').replace(/\s+/gu, ' ').trim()
  if (!normalized) throw new TypeError(`${label} must be a non-empty string`)
  return normalized
}

function optionalString(value) {
  const normalized = String(value ?? '').trim()
  return normalized || null
}

function isoDate(value, label) {
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) throw new TypeError(`${label} must be a valid date`)
  return date.toISOString()
}

function nonNegative(value, label) {
  const number = Number(value ?? 0)
  if (!Number.isFinite(number) || number < 0) throw new TypeError(`${label} must be a non-negative number`)
  return number
}

function boundedSeverity(value, label) {
  const number = nonNegative(value, label)
  if (number > 5) throw new TypeError(`${label} must be between 0 and 5`)
  return number
}

function normalizeTags(value) {
  if (!Array.isArray(value)) return []
  return [...new Set(value.map((tag) => requiredString(tag, 'tag').toLowerCase()))].sort()
}

function canonicalText(value) {
  return requiredString(value, 'feedback.text').toLocaleLowerCase('en-US')
}

export function normalizeFeedback(input) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) throw new TypeError('feedback must be an object')
  const status = requiredString(input.status ?? 'new', 'feedback.status').toLowerCase()
  if (!STATUSES.includes(status)) throw new TypeError(`unsupported feedback status: ${status}`)
  const signals = input.signals && typeof input.signals === 'object' && !Array.isArray(input.signals) ? input.signals : {}

  return {
    id: requiredString(input.id, 'feedback.id'),
    submittedAt: isoDate(input.submittedAt, 'feedback.submittedAt'),
    source: requiredString(input.source, 'feedback.source').toLowerCase(),
    text: requiredString(input.text, 'feedback.text'),
    status,
    tags: normalizeTags(input.tags),
    productArea: optionalString(input.productArea),
    accountId: optionalString(input.accountId),
    userId: optionalString(input.userId),
    evidenceUrl: optionalString(input.evidenceUrl),
    externalId: optionalString(input.externalId),
    signals: {
      votes: nonNegative(signals.votes, 'feedback.signals.votes'),
      affectedAccounts: nonNegative(signals.affectedAccounts, 'feedback.signals.affectedAccounts'),
      severity: boundedSeverity(signals.severity, 'feedback.signals.severity'),
      revenueImpact: nonNegative(signals.revenueImpact, 'feedback.signals.revenueImpact'),
    },
  }
}

export function deduplicateFeedback(inputs) {
  if (!Array.isArray(inputs)) throw new TypeError('feedback must be an array')
  const groups = new Map()
  for (const input of inputs) {
    const feedback = normalizeFeedback(input)
    const key = feedback.externalId
      ? `external:${feedback.source}:${feedback.externalId}`
      : `text:${canonicalText(feedback.text)}`
    const group = groups.get(key)
    if (group) group.push(feedback)
    else groups.set(key, [feedback])
  }

  return [...groups.entries()].map(([key, records]) => ({
    key,
    canonical: records[0],
    duplicates: records.slice(1),
    recordIds: records.map((record) => record.id).sort(),
  })).sort((left, right) => left.canonical.submittedAt.localeCompare(right.canonical.submittedAt) || left.key.localeCompare(right.key))
}

function increment(map, key, amount = 1) {
  if (key) map.set(key, (map.get(key) ?? 0) + amount)
}

function entries(map) {
  return [...map.entries()].map(([key, count]) => ({ key, count })).sort((left, right) => right.count - left.count || left.key.localeCompare(right.key))
}

export function summarizeFeedback(inputs) {
  if (!Array.isArray(inputs)) throw new TypeError('feedback must be an array')
  const records = inputs.map(normalizeFeedback)
  const sources = new Map()
  const productAreas = new Map()
  const tags = new Map()
  const statuses = new Map()
  const accounts = new Set()
  const users = new Set()

  for (const record of records) {
    increment(sources, record.source)
    increment(productAreas, record.productArea)
    increment(statuses, record.status)
    for (const tag of record.tags) increment(tags, tag)
    if (record.accountId) accounts.add(record.accountId)
    if (record.userId) users.add(record.userId)
  }

  return {
    records: records.length,
    duplicateGroups: deduplicateFeedback(records).filter((group) => group.duplicates.length > 0).length,
    uniqueAccounts: accounts.size,
    uniqueUsers: users.size,
    sources: entries(sources),
    productAreas: entries(productAreas),
    tags: entries(tags),
    statuses: entries(statuses),
  }
}

export function normalizeRankingPolicy(input = {}) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) throw new TypeError('policy must be an object')
  const weights = input.weights && typeof input.weights === 'object' && !Array.isArray(input.weights) ? input.weights : {}
  return {
    weights: {
      records: nonNegative(weights.records ?? 1, 'policy.weights.records'),
      votes: nonNegative(weights.votes ?? 1, 'policy.weights.votes'),
      affectedAccounts: nonNegative(weights.affectedAccounts ?? 1, 'policy.weights.affectedAccounts'),
      severity: nonNegative(weights.severity ?? 1, 'policy.weights.severity'),
      revenueImpact: nonNegative(weights.revenueImpact ?? 0, 'policy.weights.revenueImpact'),
    },
  }
}

export function rankFeedback(inputs, policyInput = {}) {
  const policy = normalizeRankingPolicy(policyInput)
  const ranked = deduplicateFeedback(inputs).map((group) => {
    const records = [group.canonical, ...group.duplicates]
    const uniqueAccounts = new Set(records.map((record) => record.accountId).filter(Boolean)).size
    const components = {
      records: records.length,
      votes: records.reduce((sum, record) => sum + record.signals.votes, 0),
      affectedAccounts: Math.max(uniqueAccounts, ...records.map((record) => record.signals.affectedAccounts)),
      severity: Math.max(...records.map((record) => record.signals.severity)),
      revenueImpact: records.reduce((sum, record) => sum + record.signals.revenueImpact, 0),
    }
    const contributions = Object.fromEntries(Object.entries(components).map(([key, value]) => [key, value * policy.weights[key]]))
    return {
      key: group.key,
      canonical: group.canonical,
      recordIds: group.recordIds,
      components,
      contributions,
      score: Object.values(contributions).reduce((sum, value) => sum + value, 0),
    }
  })

  return ranked.sort((left, right) => right.score - left.score || left.canonical.submittedAt.localeCompare(right.canonical.submittedAt) || left.key.localeCompare(right.key))
}
