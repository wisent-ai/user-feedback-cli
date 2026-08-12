export type FeedbackStatus = 'new' | 'reviewed' | 'planned' | 'shipped' | 'declined'

export interface FeedbackSignals {
  votes?: number
  affectedAccounts?: number
  severity?: number
  revenueImpact?: number
}

export interface Feedback {
  id: string
  submittedAt: string | Date
  source: string
  text: string
  status?: FeedbackStatus
  tags?: string[]
  productArea?: string | null
  accountId?: string | null
  userId?: string | null
  evidenceUrl?: string | null
  externalId?: string | null
  signals?: FeedbackSignals
}

export interface NormalizedFeedback extends Omit<Feedback, 'submittedAt' | 'signals'> {
  submittedAt: string
  status: FeedbackStatus
  tags: string[]
  signals: Required<FeedbackSignals>
}

export interface FeedbackGroup {
  key: string
  canonical: NormalizedFeedback
  duplicates: NormalizedFeedback[]
  recordIds: string[]
}

export interface RankingPolicy {
  weights?: Partial<Record<'records' | 'votes' | 'affectedAccounts' | 'severity' | 'revenueImpact', number>>
}

export function normalizeFeedback(input: Feedback): NormalizedFeedback
export function deduplicateFeedback(inputs: Feedback[]): FeedbackGroup[]
export function summarizeFeedback(inputs: Feedback[]): {
  records: number
  duplicateGroups: number
  uniqueAccounts: number
  uniqueUsers: number
  sources: Array<{ key: string; count: number }>
  productAreas: Array<{ key: string; count: number }>
  tags: Array<{ key: string; count: number }>
  statuses: Array<{ key: string; count: number }>
}
export function normalizeRankingPolicy(input?: RankingPolicy): { weights: Required<NonNullable<RankingPolicy['weights']>> }
export function rankFeedback(inputs: Feedback[], policy?: RankingPolicy): Array<{
  key: string
  canonical: NormalizedFeedback
  recordIds: string[]
  components: Record<string, number>
  contributions: Record<string, number>
  score: number
}>
