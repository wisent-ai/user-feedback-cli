#!/usr/bin/env node

import { readFile } from 'node:fs/promises'
import { deduplicateFeedback, normalizeFeedback, rankFeedback, summarizeFeedback } from './index.js'

function usage() {
  return `user-feedback-cli

Usage:
  user-feedback normalize --feedback <feedback.json>
  user-feedback dedupe --feedback <feedback-list.json>
  user-feedback summarize --feedback <feedback-list.json>
  user-feedback rank --feedback <feedback-list.json> [--policy <ranking-policy.json>]

The CLI preserves submitted text and ranks only explicit numeric signals.`
}

function value(args, name, required = true) {
  const index = args.indexOf(name)
  const result = index >= 0 ? args[index + 1] : null
  if (required && !result) throw new Error(`${name} is required`)
  return result
}

async function json(path) {
  return JSON.parse(await readFile(path, 'utf8'))
}

async function main() {
  const args = process.argv.slice(2)
  if (!args.length || args.includes('--help') || args.includes('-h')) {
    console.log(usage())
    return
  }

  const command = args[0]
  const feedback = await json(value(args, '--feedback'))
  let output
  if (command === 'normalize') {
    output = normalizeFeedback(feedback)
  } else if (command === 'dedupe') {
    output = deduplicateFeedback(feedback)
  } else if (command === 'summarize') {
    output = summarizeFeedback(feedback)
  } else if (command === 'rank') {
    const policyPath = value(args, '--policy', false)
    output = rankFeedback(feedback, policyPath ? await json(policyPath) : {})
  } else {
    throw new Error(`Unknown command: ${command}\n\n${usage()}`)
  }
  console.log(JSON.stringify(output, null, 2))
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error))
  process.exitCode = 1
})
